use criterion::{criterion_group, criterion_main};
use geo::frechet_distance::FrechetDistance;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("frechet distance f32", |bencher| {
        let ls_a = geo_test_fixtures::vw_orig::<f32>();
        let ls_b = geo_test_fixtures::vw_simplified::<f32>();

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls_a).frechet_distance(criterion::black_box(&ls_b)),
            );
        });
    });

    c.bench_function("frechet distance f64", |bencher| {
        let ls_a = geo_test_fixtures::vw_orig::<f64>();
        let ls_b = geo_test_fixtures::vw_simplified::<f64>();

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls_a).frechet_distance(criterion::black_box(&ls_b)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
