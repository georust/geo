use criterion::{Criterion, criterion_group, criterion_main};
use geo::{BuildBallTree, Point, point};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, Uniform};

/// Generate uniformly distributed random 2D points in [0, 1) x [0, 1).
fn random_points(n: usize, seed: u64) -> Vec<Point<f64>> {
    let mut rng = StdRng::seed_from_u64(seed);
    let dist = Uniform::new(0.0, 1.0).unwrap();
    (0..n)
        .map(|_| point!(x: dist.sample(&mut rng), y: dist.sample(&mut rng)))
        .collect()
}

fn ball_tree_benchmarks(c: &mut Criterion) {
    let points_1k = random_points(1_000, 42);
    let points_10k = random_points(10_000, 42);
    let points_100k = random_points(100_000, 42);

    // Fixed query point near the centre of the distribution.
    let query = point!(x: 0.5, y: 0.5);

    // Radius chosen so that roughly 1% of the unit square's area is covered:
    // pi * r^2 ~ 0.01  =>  r ~ 0.056
    let radius = 0.056;

    // -- construction --------------------------------------------------------
    {
        let mut group = c.benchmark_group("ball_tree_construction");

        group.bench_function("1k_points", |b| {
            b.iter(|| {
                let tree = criterion::black_box(&points_1k).build_ball_tree();
                criterion::black_box(tree);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let tree = criterion::black_box(&points_10k).build_ball_tree();
                criterion::black_box(tree);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let tree = criterion::black_box(&points_100k).build_ball_tree();
                criterion::black_box(tree);
            });
        });

        group.finish();
    }

    // Pre-build trees for the query benchmarks.
    let tree_1k = points_1k.build_ball_tree();
    let tree_10k = points_10k.build_ball_tree();
    let tree_100k = points_100k.build_ball_tree();

    // -- nearest neighbour ---------------------------------------------------
    {
        let mut group = c.benchmark_group("ball_tree_nearest_neighbour");

        group.bench_function("1k_points", |b| {
            b.iter(|| {
                let result =
                    criterion::black_box(&tree_1k).nearest_neighbour(criterion::black_box(&query));
                criterion::black_box(result);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let result =
                    criterion::black_box(&tree_10k).nearest_neighbour(criterion::black_box(&query));
                criterion::black_box(result);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let result = criterion::black_box(&tree_100k)
                    .nearest_neighbour(criterion::black_box(&query));
                criterion::black_box(result);
            });
        });

        group.finish();
    }

    // -- k-NN (k=10) ---------------------------------------------------------
    {
        let mut group = c.benchmark_group("ball_tree_knn_k10");

        group.bench_function("1k_points", |b| {
            b.iter(|| {
                let result = criterion::black_box(&tree_1k)
                    .nearest_neighbours(criterion::black_box(&query), 10);
                criterion::black_box(result);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let result = criterion::black_box(&tree_10k)
                    .nearest_neighbours(criterion::black_box(&query), 10);
                criterion::black_box(result);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let result = criterion::black_box(&tree_100k)
                    .nearest_neighbours(criterion::black_box(&query), 10);
                criterion::black_box(result);
            });
        });

        group.finish();
    }

    // -- within_radius -------------------------------------------------------
    {
        let mut group = c.benchmark_group("ball_tree_within_radius");

        group.bench_function("1k_points", |b| {
            b.iter(|| {
                let result = criterion::black_box(&tree_1k)
                    .within_radius(criterion::black_box(&query), radius);
                criterion::black_box(result);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let result = criterion::black_box(&tree_10k)
                    .within_radius(criterion::black_box(&query), radius);
                criterion::black_box(result);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let result = criterion::black_box(&tree_100k)
                    .within_radius(criterion::black_box(&query), radius);
                criterion::black_box(result);
            });
        });

        group.finish();
    }
}

criterion_group!(benches, ball_tree_benchmarks);
criterion_main!(benches);
