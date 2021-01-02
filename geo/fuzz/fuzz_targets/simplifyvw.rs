#![no_main]

use geo::algorithm::simplifyvw::SimplifyVW;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo::Polygon<f32>, f32)| {
    if tuple.1 < 0. {
        return;
    }
    tuple.0.simplifyvw(&tuple.1);
});
