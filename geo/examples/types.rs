extern crate geo;

use geo::{coord, Point};

fn main() {
    let c = coord! {
        x: 40.02f64,
        y: 116.34,
    };

    let p = Point(c);

    let Point(coord) = p;
    println!("Point at ({}, {})", coord.x, coord.y);
}
