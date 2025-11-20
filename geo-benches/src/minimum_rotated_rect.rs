use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use geo::{
    Area, BoundingRect, Centroid, ConvexHull, GeoFloat, GeoNum, LinesIter, MinimumRotatedRect,
    Rotate,
};

use geo_types::{coord, CoordFloat, LineString, Polygon};
use num_traits::Bounded;

pub fn regular_unit_polygon(num_vertices: usize) -> Polygon<f64> {
    let angle_inc = 2.0 * std::f64::consts::PI / num_vertices as f64;
    Polygon::new(
        (0..num_vertices)
            .map(|i| {
                let angle = i as f64 * angle_inc;
                coord! {
                    x: angle.cos(),
                    y: angle.sin(),
                }
            })
            .collect::<Vec<_>>()
            .into(),
        Vec::new(),
    )
}

fn criterion_benchmark(c: &mut Criterion) {
    for i in [10, 100, 1000] {
        let polygon = regular_unit_polygon(i);
        c.bench_with_input(
            BenchmarkId::new("minimum rotated rect", i),
            &polygon,
            |b, polygon| b.iter(|| polygon.minimum_rotated_rect()),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
