#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::prelude::*;
use geo::Polygon;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("extremes f32", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).extremes());
        });
    });

    c.bench_function("extremes f64", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).extremes());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
