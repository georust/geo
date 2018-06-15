#[macro_use]
extern crate criterion;
extern crate geo;
use geo::convexhull::ConvexHull;
use geo::euclidean_distance::EuclideanDistance;
use geo::Polygon;

use criterion::Criterion;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("Polygon Euclidean distance RTree f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_nonconvex_hull.rs");
        let poly1 = Polygon::new(points.into(), vec![]);
        let vec2 = vec![
            (-6.064453, 68.49604),
            (-7.426758, 68.688521),
            (-8.569336, 68.544315),
            (-9.272461, 68.431513),
            (-10.063477, 68.285651),
            (-10.898438, 68.056889),
            (-11.337891, 67.792641),
            (-11.601563, 67.458082),
            (-11.601563, 66.964476),
            (-11.425781, 66.583217),
            (-11.25, 66.319861),
            (-10.898438, 65.964377),
            (-10.678711, 65.802776),
            (-10.283203, 65.549367),
            (-9.975586, 65.385147),
            (-9.448242, 65.127638),
            (-8.701172, 64.848937),
            (-7.910156, 64.642704),
            (-7.075195, 64.642704),
            (-6.416016, 64.75539),
            (-5.361328, 64.979359),
            (-4.790039, 65.366837),
            (-4.526367, 65.676381),
            (-4.482422, 66.196009),
            (-4.614258, 66.705169),
            (-4.87793, 67.118748),
            (-4.174805, 67.474922),
            (-4.833984, 67.958148),
            (-6.064453, 68.49604),
        ];
        let poly2 = Polygon::new(vec2.into(), vec![]);
        bencher.iter(|| {
            let _ = poly1.euclidean_distance(&poly2);
        });
    });

    c.bench_function("Polygon Euclidean distance rotating calipers f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_convex_hull.rs");
        let poly1 = Polygon::new(points.into(), vec![]).convex_hull();
        let vec2 = vec![
            (-6.064453, 68.49604),
            (-7.426758, 68.688521),
            (-8.569336, 68.544315),
            (-9.272461, 68.431513),
            (-10.063477, 68.285651),
            (-10.898438, 68.056889),
            (-11.337891, 67.792641),
            (-11.601563, 67.458082),
            (-11.601563, 66.964476),
            (-11.425781, 66.583217),
            (-11.25, 66.319861),
            (-10.898438, 65.964377),
            (-10.678711, 65.802776),
            (-10.283203, 65.549367),
            (-9.975586, 65.385147),
            (-9.448242, 65.127638),
            (-8.701172, 64.848937),
            (-7.910156, 64.642704),
            (-7.075195, 64.642704),
            (-6.416016, 64.75539),
            (-5.361328, 64.979359),
            (-4.790039, 65.366837),
            (-4.526367, 65.676381),
            (-4.482422, 66.196009),
            (-4.614258, 66.705169),
            (-4.87793, 67.118748),
            (-4.174805, 67.474922),
            (-4.833984, 67.958148),
            (-6.064453, 68.49604),
        ];
        let poly2 = Polygon::new(vec2.into(), vec![]).convex_hull();
        bencher.iter(|| {
            let _ = poly1.euclidean_distance(&poly2);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
