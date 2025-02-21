use criterion::{criterion_group, criterion_main};
use geo::{line_measures::FrechetDistance, Euclidean};

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("frechet distance f32", |bencher| {
        let ls_a = geo_test_fixtures::vw_orig::<f32>();
        let ls_b = geo_test_fixtures::vw_simplified::<f32>();

        bencher.iter(|| {
            criterion::black_box(
                Euclidean
                    .frechet_distance(criterion::black_box(&ls_a), criterion::black_box(&ls_b)),
            );
        });
    });

    c.bench_function("frechet distance f64", |bencher| {
        let ls_a = geo_test_fixtures::vw_orig::<f64>();
        let ls_b = geo_test_fixtures::vw_simplified::<f64>();

        bencher.iter(|| {
            criterion::black_box(
                Euclidean
                    .frechet_distance(criterion::black_box(&ls_a), criterion::black_box(&ls_b)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
