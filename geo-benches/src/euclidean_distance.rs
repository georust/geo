use criterion::{criterion_group, criterion_main};
use geo::algorithm::{ConvexHull, Distance, Euclidean};
use geo::{LineString, Polygon, polygon};

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("Polygon Euclidean distance RTree f64", |bencher| {
        let ls = geo_test_fixtures::norway_nonconvex_hull::<f64>();
        let poly1 = Polygon::new(ls, vec![]);
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
            criterion::black_box(Euclidean.distance(&poly1, &poly2));
        });
    });

    c.bench_function(
        "Polygon Euclidean distance rotating calipers f64",
        |bencher| {
            let ls = geo_test_fixtures::norway_convex_hull::<f64>();
            let poly1 = Polygon::new(ls, vec![]).convex_hull();
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
                criterion::black_box(Euclidean.distance(&poly1, &poly2));
            });
        },
    );

    c.bench_function(
        "MultiPolygon Euclidean distance bbox pruning f64",
        |bencher| {
            // A 10x10 grid of 64-gon polygons; the target polygon sits to the
            // left of the grid, so all but the nearest members can be skipped
            // using the distance between bounding rectangles as a lower bound.
            let n_sides = 64;
            let circle = |cx: f64, cy: f64, r: f64| -> Polygon<f64> {
                let ring: LineString<f64> = (0..=n_sides)
                    .map(|i| {
                        let theta = std::f64::consts::TAU * (i % n_sides) as f64 / n_sides as f64;
                        (cx + r * theta.cos(), cy + r * theta.sin())
                    })
                    .collect::<Vec<_>>()
                    .into();
                Polygon::new(ring, vec![])
            };
            let members: Vec<Polygon<f64>> = (0..10)
                .flat_map(|gx| (0..10).map(move |gy| ((gx * 10 + 20) as f64, (gy * 10) as f64)))
                .map(|(cx, cy)| circle(cx, cy, 3.0))
                .collect();
            let multi_polygon = geo::MultiPolygon::new(members);
            let target = circle(0.0, 0.0, 3.0);
            bencher.iter(|| {
                criterion::black_box(Euclidean.distance(&multi_polygon, &target));
            });
        },
    );

    c.bench_function(
        "LineString Euclidean distance separable overlapping projections f64",
        |bencher| {
            // Two dense zigzag linestrings, separated along the x axis but offset
            // along y so that their projections onto the axis connecting the two
            // bounding-box centroids overlap over half of each geometry. This
            // exercises the prefix-pruning step of the separable fast path, which
            // would otherwise scan the overlapping region quadratically.
            let n = 1_000;
            let a: LineString<f64> = (0..n)
                .map(|i| (0.5 * (i % 2) as f64, i as f64 * 0.02))
                .collect::<Vec<_>>()
                .into();
            let y_offset = n as f64 * 0.02 * 0.5;
            let b: LineString<f64> = (0..n)
                .map(|i| (2.0 + 0.5 * (i % 2) as f64, y_offset + i as f64 * 0.02))
                .collect::<Vec<_>>()
                .into();
            bencher.iter(|| {
                criterion::black_box(Euclidean.distance(&a, &b));
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
