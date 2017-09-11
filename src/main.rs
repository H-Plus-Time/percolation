#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
use serde_json::to_string_pretty;

#[macro_use]
extern crate clap;
use clap::App;

mod thing;
use thing::{GeometricUF, GeomBounds};
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let fw = value_t!(matches, "field_width", f32).unwrap_or(1.);
    let fh = value_t!(matches, "field_height", f32).unwrap_or(1.);
    let wt = value_t!(matches, "wire_thickness", f32).unwrap_or(0.01);
    let cs = value_t!(matches, "chunk_size", i32).unwrap_or(100);
    let width = value_t!(matches, "width", f32).unwrap_or(0.1);
    let rot = value_t!(matches, "rotation", f32).unwrap_or(0.25);
    let mut simulation = GeometricUF::new(fw,fh, wt);
    simulation.percolate(cs, GeomBounds{width: width, rot: rot*PI});
    // dump the results of the simulation to disk
    let res = to_string_pretty(&simulation);
    let mut f = File::create("/home/nicholas/percolation/test.json").expect("Unable to create file");
    match res {
        Ok(x) => {
            f.write_all(x.as_bytes());
            println!("Success")
        },
        Err(why) => println!("Empty UnionFind")
    }
}
