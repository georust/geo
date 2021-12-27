#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::{intersects::Intersects, Coordinate, Line, LineString, Polygon};


fn criterion_benchmark(c: &mut Criterion) {
    let big: Vec<[f64;2 ]> = include!("../src/algorithm/test_fixtures/big.rs");
    let norway: Vec<[f64;2 ]> = include!("../src/algorithm/test_fixtures/norway_main.rs");
    c.bench_function("Large Polygon-Line intersection", |bencher| {
        let polygon = {
            let exterior = LineString::<f64>::from(big.clone());
            Polygon::new(exterior, vec![])
        };
        let line = Line::new(
            Coordinate { x: 49.92187499999999, y: 70.95969716686398 },
            Coordinate { x: 55.1953125, y: 62.59334083012024 }
        );

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&polygon).intersects(criterion::black_box(&line)),
            );
        });
    });

    c.bench_function("Polygon-Polygon intersection", |bencher| {

        let polygon_big = {
            let exterior = LineString::<f64>::from(big.clone());
            Polygon::new(exterior, vec![])
        };
        let polygon_norway = {
            let exterior = LineString::<f64>::from(norway.clone());
            Polygon::new(exterior, vec![])
        };

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&polygon_big).intersects(criterion::black_box(&polygon_norway)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
