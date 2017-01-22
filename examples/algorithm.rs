extern crate geo;

use geo::{Point, LineString, Coordinate, LineStringTrait};

fn main() {    
    let mut vec = Vec::new();
    vec.push(Point(Coordinate{
        x: 40.02f64,
        y: 116.34
    }));
    vec.push(Point(Coordinate{
        x: 41.02f64,
        y: 116.34
    }));
    let linestring = LineString(vec);
    println!("Centroid {:?}", linestring.centroid());
}
