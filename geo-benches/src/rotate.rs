use criterion::{Criterion, criterion_group, criterion_main};
use geo::prelude::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("rotate f32", |bencher| {
        let line_string = geo_test_fixtures::norway_main::<f32>();

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&line_string)
                    .rotate_around_centroid(criterion::black_box(180.)),
            );
        });
    });

    c.bench_function("rotate f64", |bencher| {
        let line_string = geo_test_fixtures::norway_main::<f64>();

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&line_string)
                    .rotate_around_centroid(criterion::black_box(180.)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
