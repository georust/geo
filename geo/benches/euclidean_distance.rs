#[macro_use]
extern crate criterion;
extern crate geo;
use geo::convex_hull::ConvexHull;
use geo::euclidean_distance::EuclideanDistance;
use geo::{polygon, Polygon};

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("Polygon Euclidean distance RTree f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_nonconvex_hull.rs");
        let poly1 = Polygon::new(points.into(), vec![]);
        let poly2 = polygon![
            (x: -6.064453, y: 68.49604),
            (x: -7.426758, y: 68.544315),
            (x: -9.272461, y: 68.431513),
            (x: -10.063477, y: 68.285651),
            (x: -10.898438, y: 68.056889),
            (x: -11.337891, y: 67.792641),
            (x: -11.601563, y: 67.458082),
            (x: -11.601563, y: 66.964476),
            (x: -11.425781, y: 66.583217),
            (x: -11.25, y: 66.319861),
            (x: -10.898438, y: 65.964377),
            (x: -10.678711, y: 65.802776),
            (x: -10.283203, y: 65.549367),
            (x: -9.975586, y: 65.385147),
            (x: -9.448242, y: 65.127638),
            (x: -8.701172, y: 64.848937),
            (x: -7.910156, y: 64.642704),
            (x: -7.075195, y: 64.642704),
            (x: -6.416016, y: 64.75539),
            (x: -5.361328, y: 64.979359),
            (x: -4.790039, y: 65.366837),
            (x: -4.526367, y: 65.676381),
            (x: -4.482422, y: 66.196009),
            (x: -4.614258, y: 66.705169),
            (x: -4.87793, y: 67.118748),
            (x: -4.174805, y: 67.474922),
            (x: -4.833984, y: 67.958148),
            (x: -6.064453, y: 68.49604),
        ];
        bencher.iter(|| {
            let _ = poly1.euclidean_distance(&poly2);
        });
    });

    c.bench_function(
        "Polygon Euclidean distance rotating calipers f64",
        |bencher| {
            let points = include!("../src/algorithm/test_fixtures/norway_convex_hull.rs");
            let poly1 = Polygon::new(points.into(), vec![]).convex_hull();
            let poly2 = polygon![
                (x: -6.064453, y: 68.49604),
                (x: -7.426758, y: 68.688521),
                (x: -8.569336, y: 68.544315),
                (x: -9.272461, y: 68.431513),
                (x: -10.063477, y: 68.285651),
                (x: -10.898438, y: 68.056889),
                (x: -11.337891, y: 67.792641),
                (x: -11.601563, y: 67.458082),
                (x: -11.601563, y: 66.964476),
                (x: -11.425781, y: 66.583217),
                (x: -11.25, y: 66.319861),
                (x: -10.898438, y: 65.964377),
                (x: -10.678711, y: 65.802776),
                (x: -10.283203, y: 65.549367),
                (x: -9.975586, y: 65.385147),
                (x: -9.448242, y: 65.127638),
                (x: -8.701172, y: 64.848937),
                (x: -7.910156, y: 64.642704),
                (x: -7.075195, y: 64.642704),
                (x: -6.416016, y: 64.75539),
                (x: -5.361328, y: 64.979359),
                (x: -4.790039, y: 65.366837),
                (x: -4.526367, y: 65.676381),
                (x: -4.482422, y: 66.196009),
                (x: -4.614258, y: 66.705169),
                (x: -4.87793, y: 67.118748),
                (x: -4.174805, y: 67.474922),
                (x: -4.833984, y: 67.958148),
                (x: -6.064453, y: 68.49604),
            ]
            .convex_hull();
            bencher.iter(|| {
                let _ = poly1.euclidean_distance(&poly2);
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
