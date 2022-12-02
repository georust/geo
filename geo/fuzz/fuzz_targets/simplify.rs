#![no_main]

use geo::CoordsIter;
use geo::EuclideanLength;
use geo::Simplify;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|tuple: (geo_types::Polygon<f32>, f32)| {
    let (polygon, epsilon) = tuple;

    if contains_nan(&polygon) {
        return;
    }

    if !is_valid(&polygon) {
        return;
    }

    let simplified = polygon.simplify(&epsilon);

    check_polygon(polygon, simplified);
});

fn contains_nan(polygon: &geo_types::Polygon<f32>) -> bool {
    polygon
        .coords_iter()
        .any(|coord| coord.x.is_nan() || coord.y.is_nan())
}

fn is_valid(polygon: &geo_types::Polygon<f32>) -> bool {
    polygon.exterior().0.len() > 3
        && polygon
            .interiors()
            .iter()
            .all(|interior| interior.0.len() > 3)
}

fn check_polygon(original: geo_types::Polygon<f32>, simplified: geo_types::Polygon<f32>) {
    check_ring(original.exterior(), simplified.exterior());

    for (simplified, original) in simplified
        .interiors()
        .iter()
        .zip(original.interiors().iter())
    {
        check_ring(original, simplified);
    }
}

fn check_ring(original: &geo_types::LineString<f32>, simplified: &geo_types::LineString<f32>) {
    assert!(
        simplified.0.len() <= original.0.len(),
        "The simplified result has {} coords, which is not less than the original's {} coords",
        simplified.0.len(),
        original.0.len()
    );
}
