use criterion::{Criterion, criterion_group, criterion_main};
use geo::{KMeans, Point, point};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};

/// Generate clustered data with multiple well-separated clusters
fn make_blobs(num_points: usize, num_clusters: usize, seed: u64) -> Vec<Point<f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut points = Vec::with_capacity(num_points);

    let points_per_cluster = num_points / num_clusters;
    let cluster_std = 1.5;
    let cluster_separation = 15.0;

    // Generate cluster centres in a grid pattern
    let grid_size = (num_clusters as f64).sqrt().ceil() as usize;

    for cluster_idx in 0..num_clusters {
        let cluster_x = (cluster_idx % grid_size) as f64 * cluster_separation;
        let cluster_y = (cluster_idx / grid_size) as f64 * cluster_separation;

        let normal = Normal::new(0.0, cluster_std).unwrap();

        let num_points_in_cluster = if cluster_idx == num_clusters - 1 {
            // Last cluster gets any remaining points
            num_points - (cluster_idx * points_per_cluster)
        } else {
            points_per_cluster
        };

        for _ in 0..num_points_in_cluster {
            let x = cluster_x + normal.sample(&mut rng);
            let y = cluster_y + normal.sample(&mut rng);
            points.push(point!(x: x, y: y));
        }
    }

    // Shuffle points
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

fn kmeans_benchmarks(c: &mut Criterion) {
    // Generate datasets with well-separated clusters for k-means
    let mut points_1k = make_blobs(1_000, 5, 42);
    let mut points_10k = make_blobs(10_000, 5, 42);
    let mut points_100k = make_blobs(100_000, 5, 42);

    // Standardise data
    standardise_points(&mut points_1k);
    standardise_points(&mut points_10k);
    standardise_points(&mut points_100k);

    // K-means parameters
    let k = 5;

    let mut group = c.benchmark_group("kmeans");

    group.bench_function("1k_points_k5", |bencher| {
        bencher.iter(|| {
            let labels = criterion::black_box(&points_1k).kmeans(k).unwrap();
            criterion::black_box(labels);
        });
    });

    group.bench_function("10k_points_k5", |bencher| {
        bencher.iter(|| {
            let labels = criterion::black_box(&points_10k).kmeans(k).unwrap();
            criterion::black_box(labels);
        });
    });

    group.sample_size(10);
    group.bench_function("100k_points_k5", |bencher| {
        bencher.iter(|| {
            let labels = criterion::black_box(&points_100k).kmeans(k).unwrap();
            criterion::black_box(labels);
        });
    });

    group.finish();

    // Benchmark with different k values on medium-sized dataset
    let mut group = c.benchmark_group("kmeans_varying_k");
    let mut points_5k = make_blobs(5_000, 3, 42);
    standardise_points(&mut points_5k);

    for k in [2, 5, 10, 20] {
        group.bench_function(format!("5k_points_k{}", k), |bencher| {
            bencher.iter(|| {
                let labels = criterion::black_box(&points_5k).kmeans(k).unwrap();
                criterion::black_box(labels);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, kmeans_benchmarks);
criterion_main!(benches);
