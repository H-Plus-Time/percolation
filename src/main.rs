#[macro_use]
extern crate serde_derive;

extern crate serde_json;
use serde_json::to_string_pretty;

#[macro_use]
extern crate clap;
use clap::App;

mod geometric_uf;
use geometric_uf::{GeometricUF, GeomBounds};
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let fw = value_t!(matches, "field_width", f32).unwrap_or(100000.);
    let fh = value_t!(matches, "field_height", f32).unwrap_or(100000.);
    let wt = value_t!(matches, "wire_thickness", f32).unwrap_or(10.);
    let cs = value_t!(matches, "chunk_size", i32).unwrap_or(100);
    let width = value_t!(matches, "width", f32).unwrap_or(500.);
    let rot = value_t!(matches, "rotation", f32).unwrap_or(0.75);
    let mut simulation = GeometricUF::new(fw,fh, wt);
    simulation.percolate(cs, GeomBounds{width: width, rot: rot*PI});
    // dump the results of the simulation to disk
    let res = to_string_pretty(&simulation);
    let mut f = File::create("/home/nicholas/percolation/test.json").expect("Unable to create file");
    match res {
        Ok(x) => {
            match f.write_all(x.as_bytes()) {
                Ok(_) => println!("Written to disk"),
                Err(_) => println!("Something appears to have gone wrong")
            }
        },
        Err(_) => println!("Empty UnionFind")
    }
}
