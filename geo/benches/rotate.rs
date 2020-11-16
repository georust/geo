#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::prelude::*;
use geo::LineString;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("rotate f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f32>::from(points);

        bencher.iter(|| {
            line_string.rotate(180.);
        });
    });

    c.bench_function("rotate f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f64>::from(points);

        bencher.iter(|| {
            line_string.rotate(180.);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
