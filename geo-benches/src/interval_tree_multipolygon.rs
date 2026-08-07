//! Benchmarks for point-in-polygon queries via `IntervalTreeMultiPolygon`.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use geo::indexed::IntervalTreeMultiPolygon;
use geo::{BoundingRect, Contains, MultiPolygon, Point, Polygon};

/// A deterministic n x n grid of query points across the geometry's bounding
/// rectangle, giving a stable mix of hits, misses, and near-boundary queries.
fn query_grid(mp: &MultiPolygon, n: usize) -> Vec<Point> {
    let rect = mp.bounding_rect().unwrap();
    let (w, h) = (rect.width(), rect.height());
    let mut points = Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            points.push(Point::new(
                rect.min().x + w * (i as f64 + 0.5) / n as f64,
                rect.min().y + h * (j as f64 + 0.5) / n as f64,
            ));
        }
    }
    points
}

fn criterion_benchmark(c: &mut Criterion) {
    // Many small polygons: exercises candidate filtering across parcels.
    let plots: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
    // One large, fjord-heavy ring: exercises long interval-tree walks, the
    // worst case for per-comparison overhead.
    let norway = MultiPolygon(vec![Polygon::new(
        geo_test_fixtures::norway_main::<f64>(),
        vec![],
    )]);

    c.bench_function("build IntervalTreeMultiPolygon (nl plots)", |bencher| {
        bencher.iter(|| {
            black_box(IntervalTreeMultiPolygon::new(&plots));
        });
    });

    for (label, mp) in [("nl plots", &plots), ("norway", &norway)] {
        let index = IntervalTreeMultiPolygon::new(mp);
        let points = query_grid(mp, 32);

        c.bench_function(
            &format!("IntervalTreeMultiPolygon contains, 1024 points ({label})"),
            |bencher| {
                bencher.iter(|| {
                    let mut hits = 0;
                    for point in &points {
                        if index.contains(black_box(point)) {
                            hits += 1;
                        }
                    }
                    black_box(hits);
                });
            },
        );

        c.bench_function(
            &format!("MultiPolygon contains unindexed, 1024 points ({label})"),
            |bencher| {
                bencher.iter(|| {
                    let mut hits = 0;
                    for point in &points {
                        if mp.contains(black_box(point)) {
                            hits += 1;
                        }
                    }
                    black_box(hits);
                });
            },
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
