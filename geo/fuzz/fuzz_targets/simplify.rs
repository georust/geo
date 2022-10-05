#![no_main]

use geo::EuclideanLength;
use geo::Simplify;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo_types::Polygon<f32>, f32)| {
    let (polygon, epsilon) = tuple;

    let simplified = polygon.simplify(&epsilon);

    check_result(polygon, simplified);
});

fn check_result(original: geo_types::Polygon<f32>, simplified: geo_types::Polygon<f32>) {
    check_ring(original.exterior(), simplified.exterior());

    for (original_interior, simplified_interior) in
        original.interiors().iter().zip(simplified.interiors())
    {
        check_ring(original_interior, simplified_interior);
    }
}

fn check_ring(original: &geo_types::LineString<f32>, simplified: &geo_types::LineString<f32>) {
    assert!(
        simplified.0.len() <= original.0.len(),
        "Simplified ring has more coordinates than the original exterior ring ({:?} <= {:?})",
        simplified.0.len(),
        original.0.len(),
    );

    let simplified_length = simplified.euclidean_length();
    let original_length = simplified.euclidean_length();

    if simplified_length.is_nan() || original_length.is_nan() {
        return;
    }

    assert!(
        simplified_length <= original_length,
        "Simplified ring is longer than the original exterior ring ({:?} <= {:?})",
        simplified_length,
        original_length,
    );
}
