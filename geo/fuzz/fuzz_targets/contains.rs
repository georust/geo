#![no_main]

use geo::algorithm::contains::Contains;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo::Geometry<f32>, geo::Point<f32>)| {
    tuple.0.contains(&tuple.1);
});
