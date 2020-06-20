#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::algorithm::area::Area;
use geo::{LineString, Polygon};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("area", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let polygon = Polygon::new(LineString::<f32>::from(points), vec![]);

        bencher.iter(|| {
            criterion::black_box(|| {
                polygon.signed_area();
            });
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
