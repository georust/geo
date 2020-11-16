#[macro_use]
extern crate criterion;
extern crate geo;

use geo::prelude::*;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("vincenty distance f32", |bencher| {
        let a = geo::Point::<f32>::new(17.107558, 48.148636);
        let b = geo::Point::<f32>::new(16.372477, 48.208810);

        bencher.iter(|| {
            let _ = a.vincenty_distance(&b);
        });
    });

    c.bench_function("vincenty distance f64", |bencher| {
        let a = geo::Point::<f64>::new(17.107558, 48.148636);
        let b = geo::Point::<f64>::new(16.372477, 48.208810);

        bencher.iter(|| {
            let _ = a.vincenty_distance(&b);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
