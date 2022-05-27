#[macro_use]
extern crate criterion;
#[macro_use]
extern crate geo;

use geo::contains::Contains;
use geo::{polygon, Line, Point, Polygon};

use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
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

    c.bench_function("point outside simple polygon", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(2.0, 2.0);
        bencher.iter(|| {
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function("point inside complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        let point = geo_test_fixtures::baton_rouge();
        bencher.iter(|| {
            assert!(criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function("point outside complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        // lake borgne - near and mostly surrounded by, but not inside of, Louisiana
        let point = point!(x: -89.641854, y: 30.026283);
        bencher.iter(|| {
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function("line across complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        // crossing part of, but not contained by Louisiana
        let line = Line::new(
            geo_test_fixtures::baton_rouge(),
            point!(x: -89.641854, y: 30.026283),
        );
        bencher.iter(|| {
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&line)));
        });
    });

    c.bench_function("complex polygon contains polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        let contained_polygon = geo_test_fixtures::east_baton_rouge();

        bencher.iter(|| {
            assert!(
                criterion::black_box(&polygon).contains(criterion::black_box(&contained_polygon))
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
