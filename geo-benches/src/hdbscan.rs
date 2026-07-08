use criterion::{Criterion, criterion_group, criterion_main};
use geo::Hdbscan;
use geo_benches::utils::random::make_moons;

fn hdbscan_benchmarks(c: &mut Criterion) {
    let points_1k = make_moons(1_000, 0.1, 42);
    let points_10k = make_moons(10_000, 0.1, 42);
    let points_100k = make_moons(100_000, 0.1, 42);

    // HDBSCAN parameters: min_cluster_size=15, min_samples=5 are typical
    // starting values for this kind of dataset.
    let min_cluster_size = 15;
    let min_samples = 5;

    let mut group = c.benchmark_group("hdbscan");

    group.bench_function("1k_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_1k)
                .hdbscan_with_min_samples(min_cluster_size, min_samples)
                .unwrap();
            criterion::black_box(result);
        });
    });

    group.bench_function("10k_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_10k)
                .hdbscan_with_min_samples(min_cluster_size, min_samples)
                .unwrap();
            criterion::black_box(result);
        });
    });

    group.sample_size(10);
    group.bench_function("100k_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_100k)
                .hdbscan_with_min_samples(min_cluster_size, min_samples)
                .unwrap();
            criterion::black_box(result);
        });
    });

    let points_500k = make_moons(500_000, 0.1, 42);
    group.bench_function("500k_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_500k)
                .hdbscan_with_min_samples(min_cluster_size, min_samples)
                .unwrap();
            criterion::black_box(result);
        });
    });

    let points_1e6 = make_moons(1_000_000, 0.1, 42);
    group.bench_function("1e6_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_1e6)
                .hdbscan_with_min_samples(min_cluster_size, min_samples)
                .unwrap();
            criterion::black_box(result);
        });
    });

    let points_3e6 = make_moons(3_000_000, 0.1, 42);
    group.bench_function("3e6_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_3e6)
                .hdbscan_with_min_samples(min_cluster_size, min_samples)
                .unwrap();
            criterion::black_box(result);
        });
    });

    group.finish();
}

criterion_group!(benches, hdbscan_benchmarks);
criterion_main!(benches);
