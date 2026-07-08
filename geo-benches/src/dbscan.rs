use criterion::{Criterion, criterion_group, criterion_main};
use geo::{Dbscan, Point, point};
use geo_benches::utils::random::make_moons;

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
