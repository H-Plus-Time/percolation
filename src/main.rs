#![feature(link_args)]
#[link_args = "-s ALLOW_MEMORY_GROWTH=1"]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate stdweb;
extern crate serde_json;
use serde_json::to_string_pretty;
use std::os::raw::c_char;
use std::ffi::CString;
#[macro_use]
extern crate clap;
use clap::App;

mod geometric_uf;
use geometric_uf::{GeometricUF, GeomBounds};
use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use stdweb::serde::Serde;

#[cfg(target_os = "emscripten")]
fn main() {
    stdweb::initialize();
    println!("Hello, world!");
    stdweb::event_loop();
}


#[no_mangle]
pub fn percolate(fw: f32, fh: f32, wt: f32, cs: i32, width: f32, rot: f32) -> *mut c_char {
    let mut simulation = GeometricUF::new(fw,fh,wt);
    simulation.percolate(cs, GeomBounds{width, rot: rot*PI});
    let res = to_string_pretty(&simulation);
    match res {
        Ok(msg) => {
                return CString::new(msg).unwrap().into_raw();
        },
        Err(_) => {
            return CString::new("oh no!").unwrap().into_raw();
        }
    }
}

#[no_mangle]
pub fn generate(fw: f32, fh: f32, wt: f32, cs: i32, width: f32, rot: f32, num_iter: i32) -> *mut c_char {
    let mut simulation = GeometricUF::new(fw,fh,wt);
    simulation.generate(cs, GeomBounds{width, rot: rot*PI}, num_iter);
    let res = to_string_pretty(&simulation);
    match res {
        Ok(msg) => {
                return CString::new(msg).unwrap().into_raw();
        },
        Err(_) => {
            return CString::new("oh no!").unwrap().into_raw();
        }
    }
}

#[cfg(not(target_os = "emscripten"))]
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
