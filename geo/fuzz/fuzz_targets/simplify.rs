#![no_main]

use geo::algorithm::simplify::Simplify;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo::Polygon<f32>, f32)| {
    if tuple.1 < 0. {
        return;
    }
    tuple.0.simplify(&tuple.1);
});
