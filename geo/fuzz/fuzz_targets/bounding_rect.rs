#![no_main]

use geo::algorithm::bounding_rect::BoundingRect;
use geo::algorithm::contains::Contains;
use geo::algorithm::intersects::Intersects;
use geo::algorithm::coords_iter::CoordsIter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|geometry: geo::Geometry<f32>| {
    let rect = match geometry.bounding_rect() {
        Some(r) => r,
        None => return,
    };

    for coord in geometry.coords_iter() {
        assert!(rect.contains(&coord) || rect.intersects(&coord));
    }
});
