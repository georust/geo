#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::simplify::Simplify;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simplify simple f32", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&ls).simplify(criterion::black_box(&0.01)));
        });
    });

    c.bench_function("simplify simple f64", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();
        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&ls).simplify(criterion::black_box(&0.01)));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
