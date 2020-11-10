#![no_main]

use geo::algorithm::bounding_rect::BoundingRect;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|geometry: geo::Geometry<f32>| {
    geometry.bounding_rect();
});
