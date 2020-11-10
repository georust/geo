#![no_main]

use geo::algorithm::intersects::Intersects;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|geometries: [geo::Geometry<f32>; 2]| {
    geometries[0].intersects(&geometries[1]);
});
