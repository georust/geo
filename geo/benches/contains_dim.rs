use criterion::{criterion_group, criterion_main, Criterion};
use geo::algorithm::{Contains, Convert, Relate};
use geo::geometry::*;
use geo::{coord, point, polygon};

fn different_dimension_benchmark(c: &mut Criterion) {
    c.bench_function("polygon not in point (Contains Trait)", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(0.5, 0.1);
        bencher.iter(|| {
            assert!(!criterion::black_box(&point).contains(criterion::black_box(&polygon)));
        });
    });

    c.bench_function("polygon not in point (Relates Trait)", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(0.5, 0.1);
        bencher.iter(|| {
            assert!(
                !(criterion::black_box(&point)
                    .relate(criterion::black_box(&polygon))
                    .is_contains())
            );
        });
    });

    c.bench_function("degenerate polygon in point (Contains Trait)", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 0.0, y: 0.0),
            (x: 0.0, y: 0.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(0.0, 0.0);
        bencher.iter(|| {
            assert!((criterion::black_box(&point).contains(criterion::black_box(&polygon))));
        });
    });

    c.bench_function(
        "degenerate polygon <not> in point (Relates Trait)",
        // Relates trait returns false for degenerate polygons in point
        |bencher| {
            let polygon = polygon![
                (x: 0.0f64, y: 0.0),
                (x: 0.0, y: 0.0),
                (x: 0.0, y: 0.0),
                (x: 0.0, y: 0.0),
            ];
            let point = Point::new(0.0, 0.0);
            bencher.iter(|| {
                assert!(
                    !(criterion::black_box(&point)
                        .relate(criterion::black_box(&polygon))
                        .is_contains())
                );
            });
        },
    );
}

fn lower_dimension_in_higher_dimension_benchmark(c: &mut Criterion) {
    c.bench_function("point in simple polygon", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(0.5, 0.1);
        bencher.iter(|| {
            assert!(criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });
}

criterion_group!(
    benches,
    different_dimension_benchmark,
    lower_dimension_in_higher_dimension_benchmark
);
criterion_main!(benches);
