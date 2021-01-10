#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::prelude::*;
use geo::simplifyvw::SimplifyVWPreserve;
use geo::LineString;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simplify vw simple f32", |bencher| {
        let points = include!("../src/test_fixtures/louisiana.rs");
        let ls: LineString<f32> = points.into();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplifyvw(criterion::black_box(&0.0005)),
            );
        });
    });

    c.bench_function("simplify vw simple f64", |bencher| {
        let points = include!("../src/test_fixtures/louisiana.rs");
        let ls: LineString<f64> = points.into();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplifyvw(criterion::black_box(&0.0005)),
            );
        });
    });

    c.bench_function("simplify vwp f32", |bencher| {
        let points = include!("../src/test_fixtures/louisiana.rs");
        let ls: LineString<f32> = points.into();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplifyvw_preserve(criterion::black_box(&0.0005)),
            );
        });
    });

    c.bench_function("simplify vwp f64", |bencher| {
        let points = include!("../src/test_fixtures/louisiana.rs");
        let ls: LineString<f32> = points.into();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplifyvw_preserve(criterion::black_box(&0.0005)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
