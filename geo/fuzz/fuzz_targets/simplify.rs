#![no_main]

use geo::algorithm::simplify::Simplify;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo_types::Polygon<f32>, f32)| {
    tuple.0.simplify(&tuple.1);
});
