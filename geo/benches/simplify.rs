#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::LineString;
use geo::simplifyvw::SimplifyVWPreserve;
use geo::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simplify vw simple f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let ls: LineString<f32> = points.into();
        bencher.iter(|| {
            let _ = ls.simplifyvw(&0.0005);
        });
    });

    c.bench_function("simplify vwp f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let ls: LineString<f32> = points.into();
        bencher.iter(|| {
            let _ = ls.simplifyvw_preserve(&0.0005);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
