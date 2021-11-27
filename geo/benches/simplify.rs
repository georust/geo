#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::simplify::Simplify;
use geo::LineString;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simplify simple f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/louisiana.rs");
        let ls: LineString<f32> = points.into();
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&ls).simplify(criterion::black_box(&0.01)));
        });
    });

    c.bench_function("simplify simple f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/louisiana.rs");
        let ls: LineString<f64> = points.into();
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&ls).simplify(criterion::black_box(&0.01)));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
