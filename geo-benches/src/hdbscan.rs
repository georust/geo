use criterion::{Criterion, criterion_group, criterion_main};
use geo::{Hdbscan, Point, point};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;

/// Generate test data similar to sklearn's make_moons.
///
/// Creates two interleaving half circles with optional Gaussian noise,
/// reusing the same generator as the DBSCAN benchmark for comparability.
fn make_moons(num_points: usize, noise: f64, seed: u64) -> Vec<Point<f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut points = Vec::with_capacity(num_points);

    let n_samples_out = num_points / 2;
    let n_samples_in = num_points - n_samples_out;

    // Outer semicircle
    for i in 0..n_samples_out {
        let theta = (i as f64) * PI / (n_samples_out - 1) as f64;
        let x = theta.cos();
        let y = theta.sin();
        points.push(point!(x: x, y: y));
    }

    // Inner semicircle (offset and flipped)
    for i in 0..n_samples_in {
        let theta = (i as f64) * PI / (n_samples_in - 1) as f64;
        let x = 1.0 - theta.cos();
        let y = 1.0 - theta.sin() - 0.5;
        points.push(point!(x: x, y: y));
    }

    if noise > 0.0 {
        let normal = Normal::new(0.0, noise).unwrap();
        for point in &mut points {
            let noise_x = normal.sample(&mut rng);
            let noise_y = normal.sample(&mut rng);
            *point = point!(x: point.x() + noise_x, y: point.y() + noise_y);
        }
    }

    use rand::seq::SliceRandom;
    points.shuffle(&mut rng);

    points
}

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
                .hdbscan_with_min_samples(min_cluster_size, min_samples);
            criterion::black_box(result);
        });
    });

    group.bench_function("10k_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_10k)
                .hdbscan_with_min_samples(min_cluster_size, min_samples);
            criterion::black_box(result);
        });
    });

    group.sample_size(10);
    group.bench_function("100k_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_100k)
                .hdbscan_with_min_samples(min_cluster_size, min_samples);
            criterion::black_box(result);
        });
    });

    let points_500k = make_moons(500_000, 0.1, 42);
    group.bench_function("500k_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_500k)
                .hdbscan_with_min_samples(min_cluster_size, min_samples);
            criterion::black_box(result);
        });
    });

    let points_1e6 = make_moons(1_000_000, 0.1, 42);
    group.bench_function("1e6_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_1e6)
                .hdbscan_with_min_samples(min_cluster_size, min_samples);
            criterion::black_box(result);
        });
    });

    let points_3e6 = make_moons(3_000_000, 0.1, 42);
    group.bench_function("3e6_points", |bencher| {
        bencher.iter(|| {
            let result = criterion::black_box(&points_3e6)
                .hdbscan_with_min_samples(min_cluster_size, min_samples);
            criterion::black_box(result);
        });
    });

    group.finish();
}

criterion_group!(benches, hdbscan_benchmarks);
criterion_main!(benches);
