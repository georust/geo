#[macro_use]
extern crate criterion;
extern crate geo;

use geo::prelude::*;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("geodesic distance f64", |bencher| {
        let a = geo::Point::new(17.107558, 48.148636);
        let b = geo::Point::new(16.372477, 48.208810);

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&a).geodesic_distance(criterion::black_box(&b)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
