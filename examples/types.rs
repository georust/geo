extern crate geo;

use geo::*;

fn main() {
    let p = Point::new(40.02, 116.34);
    println!("Point at ({}, {})", p.x(), p.y());
}
