use criterion::{Criterion, criterion_group, criterion_main};
use geo::{Dbscan, Point, point};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use std::f64::consts::PI;

/// Generate test data similar to sklearn's make_moons
/// Creates two interleaving half circles
fn make_moons(num_points: usize, noise: f64, seed: u64) -> Vec<Point<f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut points = Vec::with_capacity(num_points);

    // Split points between outer and inner semicircles
    let n_samples_out = num_points / 2;
    let n_samples_in = num_points - n_samples_out;

    // Outer semicircle: parametric form with θ from 0 to π
    for i in 0..n_samples_out {
        let theta = (i as f64) * PI / (n_samples_out - 1) as f64;
        let x = theta.cos();
        let y = theta.sin();
        points.push(point!(x: x, y: y));
    }

    // Inner semicircle: offset and flipped to interleave
    for i in 0..n_samples_in {
        let theta = (i as f64) * PI / (n_samples_in - 1) as f64;
        let x = 1.0 - theta.cos();
        let y = 1.0 - theta.sin() - 0.5;
        points.push(point!(x: x, y: y));
    }

    // Add Gaussian noise if specified
    if noise > 0.0 {
        let normal = Normal::new(0.0, noise).unwrap();
        for point in &mut points {
            let noise_x = normal.sample(&mut rng);
            let noise_y = normal.sample(&mut rng);
            *point = point!(x: point.x() + noise_x, y: point.y() + noise_y);
        }
    }

    // Shuffle points to mix the two semicircles
    use rand::seq::SliceRandom;
    points.shuffle(&mut rng);

    points
}

/// Standardise points to have zero mean and unit variance (like sklearn's StandardScaler)
fn standardise_points(points: &mut [Point<f64>]) {
    let n = points.len() as f64;

    // Calculate means
    let (sum_x, sum_y) = points
        .iter()
        .fold((0.0, 0.0), |(sx, sy), p| (sx + p.x(), sy + p.y()));
    let mean_x = sum_x / n;
    let mean_y = sum_y / n;

    // Calculate standard deviations
    let (sum_sq_x, sum_sq_y) = points.iter().fold((0.0, 0.0), |(sx, sy), p| {
        let dx = p.x() - mean_x;
        let dy = p.y() - mean_y;
        (sx + dx * dx, sy + dy * dy)
    });
    let std_x = (sum_sq_x / n).sqrt();
    let std_y = (sum_sq_y / n).sqrt();

    // Standardise each point
    for point in points {
        let x = (point.x() - mean_x) / std_x;
        let y = (point.y() - mean_y) / std_y;
        *point = point!(x: x, y: y);
    }
}

fn dbscan_benchmarks(c: &mut Criterion) {
    // Generate make_moons datasets with noise=0.1 (typical value)
    let mut points_1k = make_moons(1_000, 0.1, 42);
    let mut points_10k = make_moons(10_000, 0.1, 42);
    let mut points_100k = make_moons(100_000, 0.1, 42);

    // Standardise data like sklearn does (zero mean, unit variance)
    standardise_points(&mut points_1k);
    standardise_points(&mut points_10k);
    standardise_points(&mut points_100k);

    // DBSCAN parameters from sklearn examples (for standardised data)
    let eps = 0.3;
    let min_samples = 5;

    let mut group = c.benchmark_group("dbscan");

    group.bench_function("1k_points", |bencher| {
        bencher.iter(|| {
            let labels = criterion::black_box(&points_1k).dbscan(eps, min_samples);
            criterion::black_box(labels);
        });
    });

    group.bench_function("10k_points", |bencher| {
        bencher.iter(|| {
            let labels = criterion::black_box(&points_10k).dbscan(eps, min_samples);
            criterion::black_box(labels);
        });
    });

    group.sample_size(10);
    group.bench_function("100k_points", |bencher| {
        bencher.iter(|| {
            let labels = criterion::black_box(&points_100k).dbscan(eps, min_samples);
            criterion::black_box(labels);
        });
    });

    group.finish();
}

criterion_group!(benches, dbscan_benchmarks);
criterion_main!(benches);
