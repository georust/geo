#![no_main]

use geo::algorithm::geodesic_distance::GeodesicDistance;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo::Point<f64>, geo::Point<f64>)| {
    tuple.0.geodesic_distance(&tuple.1);
});
