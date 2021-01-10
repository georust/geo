#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::prelude::*;
use geo::{LineString, Polygon};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("extremes f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let polygon = Polygon::new(LineString::<f32>::from(points), vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).extremes());
        });
    });

    c.bench_function("extremes f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let polygon = Polygon::new(LineString::<f32>::from(points), vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).extremes());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
