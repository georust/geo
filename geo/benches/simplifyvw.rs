#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::prelude::*;
use geo::simplify_vw::SimplifyVwPreserve;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simplify vw simple f32", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw(criterion::black_box(&0.0005)),
            );
        });
    });

    c.bench_function("simplify vw simple f64", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw(criterion::black_box(&0.0005)),
            );
        });
    });

    c.bench_function("simplify vwp f32", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw_preserve(criterion::black_box(&0.0005)),
            );
        });
    });

    c.bench_function("simplify vwp f64", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls).simplify_vw_preserve(criterion::black_box(&0.0005)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
