extern crate geo;
use self::geo::{Point, LineString, Polygon};
use self::geo::algorithm::intersects::Intersects;
extern crate itertools;
use self::itertools::Itertools;
extern crate rand;
use self::rand::distributions::{IndependentSample, Range};
use self::rand::distributions::normal::Normal;
extern crate lyon_bezier;
use self::lyon_bezier::{QuadraticBezierSegment, QuadraticFlatteningIter};
use self::lyon_bezier::Point as eucPoint;
pub struct GeomBounds {
    pub width: f32,
    pub rot: f32
}

#[derive(Debug, Serialize)]
pub struct UF {
    count: i32,
    parent: Vec<i32>,
    rank: Vec<i32>
}

#[derive(Debug, Serialize)]
pub struct GeometricUF {
    count: i32,
    parent: Vec<i32>,
    rank: Vec<i32>,
    objects: Vec<Polygon<f32>>,
    fieldheight: f32,
    fieldwidth: f32,
    wire_thickness: f32
}

pub trait UnionFind {
    fn find(&mut self, p: i32) -> i32;
    fn union(&mut self, p: i32, q: i32);
    fn connected(&mut self, p: i32, q: i32) -> bool {
        return self.find(p) == self.find(q);
    }
}

impl UnionFind for UF {
    fn find(&mut self, mut p: i32) -> i32 {
        while p != self.parent[p as usize] {
            self.parent[p as usize] = self.parent[self.parent[p as usize] as usize];
            p = self.parent[p as usize];
        }
        return p;
    }
    fn union(&mut self, p: i32, q: i32) {
        let root_p = self.find(p);
        let root_q = self.find(q);
        if root_p == root_q {
            return;
        }

        // make root of smaller rank point to root of larger rank
        if self.rank[root_p as usize] < self.rank[root_q as usize] {
            self.parent[root_p as usize] = root_q;
        } 
        else if self.rank[root_p as usize] > self.rank[root_q as usize] {
            self.parent[root_q as usize] = root_p;
        } 
        else {
            self.parent[root_q as usize] = root_p;
            self.rank[root_p as usize] += 1;
        }
        self.count -= 1;
    }
}

impl UnionFind for GeometricUF {
    fn find(&mut self, mut p: i32) -> i32 {
        while p != self.parent[p as usize] {
            self.parent[p as usize] = self.parent[self.parent[p as usize] as usize];
            p = self.parent[p as usize];
        }
        return p;
    }
    fn union(&mut self, p: i32, q: i32) {
        let root_p = self.find(p);
        let root_q = self.find(q);
        if root_p == root_q {
            return;
        }

        // make root of smaller rank point to root of larger rank
        if self.rank[root_p as usize] < self.rank[root_q as usize] {
            self.parent[root_p as usize] = root_q;
        } 
        else if self.rank[root_p as usize] > self.rank[root_q as usize] {
            self.parent[root_q as usize] = root_p;
        } 
        else {
            self.parent[root_q as usize] = root_p;
            self.rank[root_p as usize] += 1;
        }
        self.count -= 1;
    }
}
impl GeometricUF {
    pub fn new(fieldwidth: f32, fieldheight: f32, wire_thickness: f32) -> GeometricUF {
        let mut objects = Vec::new();
        let exterior_top = LineString(vec![Point::new(0., 0.), Point::new(fieldwidth, 0.),
                                    Point::new(fieldwidth, wire_thickness), Point::new(0., wire_thickness), Point::new(0., 0.)]);
        let exterior_bottom = LineString(vec![Point::new(0., fieldheight), Point::new(fieldwidth, fieldheight),
                                    Point::new(fieldwidth, (fieldheight+wire_thickness)), Point::new(0., (fieldheight+wire_thickness)), Point::new(0., 0.)]);
        let top = Polygon::new(exterior_top.clone(), Vec::new());
        let bottom = Polygon::new(exterior_bottom.clone(), Vec::new());
        objects.push(top);
        objects.push(bottom);
        let forest = GeometricUF{
            count: 2, parent: (0..2).collect(),
            rank: vec![0; 2],
            objects: objects,
            fieldwidth, fieldheight, wire_thickness
        };
        return forest;
    }
    fn percolates(&mut self) -> bool {
        return self.connected(0, 1);
    }
    fn add_elements(&mut self, elems: Vec<Polygon<f32>>) {
        let len: i32 = self.parent.len() as i32;
        let elem_len: i32 = elems.len() as i32;
        self.count += elem_len;
        let vals: Vec<i32> = (len..elem_len+len).collect();
        self.parent.extend(vals.iter().cloned());
        self.rank.extend(vec![0; elem_len as usize]);
        self.objects.extend(elems);
    }
    fn connect(&mut self, offset: Option<i32>) {
        let offset = offset.unwrap_or(0);
        let mut changes: Vec<(i32, i32)> = Vec::new();
        for (i, elem) in self.objects.iter().enumerate().skip(offset as usize) {
            // get the bounding box of the ith geometric object, intersect with
            // all preceding
            for (j, other) in self.objects.iter().skip(i as usize).enumerate() {
                // actually do the bounding box intersection
                if elem.intersects(other) {
                    changes.push((i as i32, j as i32));
                }
            }
        }
        changes.iter().foreach(|&(i,j)| { self.union(i,j) })
    }
    // TODO implement de Casteljau, polygonization
    fn generate_wire(&self, cx: f32, cy: f32, rot: f32, r: f32, ox: f32, oy: f32) -> Polygon<f32> {
        // calculate start and end from centre point, rotation and width
        let start = eucPoint::new(
            cx-r*rot.sin(),
            cy-r*rot.cos()
        );
        let end = eucPoint::new(
            cx+r*rot.sin(),
            cy+r*rot.cos()
        );
        let ctrl = eucPoint::new(
            cx+ox*rot.sin()*r,
            cy+oy*rot.cos()*r
        );
        let bez = QuadraticBezierSegment{
            from: start,
            to: end,
            ctrl: ctrl
        };
        let mut curve1 = Vec::new();
        let mut curve2 = Vec::new();
        // need to adjust the proportion of x and y shift
        curve1.push(Point::new(bez.from.x+r*self.wire_thickness, bez.from.y+r*self.wire_thickness));
        curve2.push(Point::new(bez.from.x-r*self.wire_thickness, bez.from.y-r*self.wire_thickness));
        for p in bez.flattening_iter(0.001) {
            curve1.push(Point::new(p.x+r*self.wire_thickness, p.y+r*self.wire_thickness));
            curve2.push(Point::new(p.x-r*self.wire_thickness, p.y+r*self.wire_thickness));
        }
        curve2.reverse();
        curve1.extend(curve2);
        
        curve1.push(Point::new(bez.from.x+r*0.05, bez.from.y+r*0.05));
        let exteriors = LineString(curve1);
        
        // return implicitly
        return Polygon::new(exteriors.clone(), Vec::new());
    }

    pub fn percolate(&mut self, chunk_size: i32, geom_bounds: GeomBounds) {
        let uniform = Range::new(0.0f32, 1.0f32);
        let mut rng = rand::thread_rng();
        let std_norm = Normal::new(0.5, 0.1666);
        let signed_norm = Normal::new(0.0, 0.3333);
        loop {
            let current_length: i32 = self.parent.len() as i32;
            let geoms = (0..chunk_size).map(|_| {
                let cx = uniform.ind_sample(&mut rng)*self.fieldwidth;
                let cy = uniform.ind_sample(&mut rng)*self.fieldheight;
                let rot = (signed_norm.ind_sample(&mut rng) as f32)*geom_bounds.rot;
                let r = (std_norm.ind_sample(&mut rng) as f32)*geom_bounds.width/2.0;
                let hypot = (std_norm.ind_sample(&mut rng) as f32)*geom_bounds.width/2.0;
                let theta_o = signed_norm.ind_sample(&mut rng) as f32;
                let ox = theta_o.sin()*hypot;
                let oy = theta_o.cos()*hypot;
                return (cx,cy,rot,r,ox,oy);
            }).filter(|&(_,_,_,r,_,_)| { r > 0.0f32 }).map(|(cx,cy,rot,r,ox,oy)| {
                self.generate_wire(cx,cy,rot,r,ox,oy)
            }).collect();
            self.add_elements(geoms);
            self.connect(Some(current_length));
            if self.percolates() {
                println!("Number of elements: {:?}", self.parent.len());
                break;
            }
        }
        
    }
}
