extern crate geo;

use geo::{Point, Coordinate};

fn main() {
    let c = Coordinate {
        x: 40.02f64,
        y: 116.34
    };

    let p = Point(c);

    let Point(coord) = p;
    println!("Point at ({}, {})", coord.x, coord.y);
}
