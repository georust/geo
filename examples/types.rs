extern crate geo;

use geo::*;

fn main() {
    let c = Coordinate {
        x: 40.02f64,
        y: 116.34
    };

    let p = Point {
        coordinate: c
    };

    println!("Point at ({}, {})", p.coordinate.x, p.coordinate.y);
}
