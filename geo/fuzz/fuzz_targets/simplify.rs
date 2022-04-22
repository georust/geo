#![no_main]

use geo::algorithm::euclidean_length::EuclideanLength;
use geo::algorithm::simplify::Simplify;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo_types::Polygon<f32>, f32)| {
    let (polygon, epsilon) = tuple;

    let simplified = polygon.simplify(&epsilon);

    check_result(polygon, simplified);
});

fn check_result(original: geo_types::Polygon<f32>, simplified: geo_types::Polygon<f32>) {
    assert!(simplified.exterior().inner().len() <= original.exterior().inner().len());
    assert!(simplified.exterior().euclidean_length() <= original.exterior().euclidean_length());

    for interior in simplified.interiors() {
        assert!(simplified.exterior().inner().len() <= interior.0.len());
        assert!(simplified.exterior().euclidean_length() <= interior.euclidean_length());
    }
}
