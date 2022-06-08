#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::Area;
use geo::Polygon;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("area", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).signed_area());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
