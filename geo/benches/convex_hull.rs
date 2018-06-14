#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::LineString;
use geo::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("convex hull f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f32>::from(points);

        bencher.iter(|| {
            line_string.convex_hull();
        });
    });

    c.bench_function("convex hull f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f64>::from(points);

        bencher.iter(|| {
            line_string.convex_hull();
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
