extern crate geo;

use geo::prelude::*;
use std::error::Error;

const MEAN_EARTH_RADIUS: f64 = 6_371_000.0;

fn main() -> Result<(), Box<Error>> {
    let reykjavik = geo::Point(geo::Coordinate {
        y: 64.1265,
        x: -21.8174,
    });
    let nyc = geo::Point(geo::Coordinate {
        y: 40.7128,
        x: -74.0060,
    });

    println!(
        "old euclidean distance: {}",
        reykjavik.euclidean_distance(&nyc) * MEAN_EARTH_RADIUS
    );
    println!(
        "new euclidean distance: {}",
        new_euclidean_distance(&reykjavik, &nyc)
    );
    println!("haversine distance: {}", reykjavik.haversine_distance(&nyc));
    println!("vincenty distance: {}", reykjavik.vincenty_distance(&nyc)?);

    Ok(())
}

fn new_euclidean_distance(p1: &geo::Point<f64>, p2: &geo::Point<f64>) -> f64 {
    let x1 = p1.x().to_radians();
    let y1 = p1.y().to_radians();
    let x2 = p2.x().to_radians();
    let y2 = p2.y().to_radians();

    let x = (x2 - x1) * ((y1 + y2) / 2.).cos();
    let y = y2 - y1;
    x.hypot(y) * MEAN_EARTH_RADIUS
}
