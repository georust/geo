use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use geo::{BallTree, BallTreeBuilder, Point, point};
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
                let tree = BallTree::new(black_box(points_1k.clone()));
                black_box(tree);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let tree = BallTree::new(black_box(points_10k.clone()));
                black_box(tree);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let tree = BallTree::new(black_box(points_100k.clone()));
                black_box(tree);
            });
        });

        group.finish();
    }

    // Pre-build trees for the query benchmarks.
    let tree_1k = BallTree::new(points_1k.clone());
    let tree_10k = BallTree::new(points_10k.clone());
    let tree_100k = BallTree::new(points_100k.clone());

    // -- nearest neighbour ---------------------------------------------------
    {
        let mut group = c.benchmark_group("ball_tree_nearest_neighbour");

        group.bench_function("1k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_1k).nearest_neighbour(black_box(&query));
                black_box(result);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_10k).nearest_neighbour(black_box(&query));
                black_box(result);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_100k).nearest_neighbour(black_box(&query));
                black_box(result);
            });
        });

        group.finish();
    }

    // -- k-NN (k=10) ---------------------------------------------------------
    {
        let mut group = c.benchmark_group("ball_tree_knn_k10");

        group.bench_function("1k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_1k).nearest_neighbours(black_box(&query), 10);
                black_box(result);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_10k).nearest_neighbours(black_box(&query), 10);
                black_box(result);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_100k).nearest_neighbours(black_box(&query), 10);
                black_box(result);
            });
        });

        group.finish();
    }

    // -- within_radius -------------------------------------------------------
    {
        let mut group = c.benchmark_group("ball_tree_within_radius");

        group.bench_function("1k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_1k).within_radius(black_box(&query), radius);
                black_box(result);
            });
        });

        group.bench_function("10k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_10k).within_radius(black_box(&query), radius);
                black_box(result);
            });
        });

        group.sample_size(10);
        group.bench_function("100k_points", |b| {
            b.iter(|| {
                let result = black_box(&tree_100k).within_radius(black_box(&query), radius);
                black_box(result);
            });
        });

        group.finish();
    }

    // -- leaf size sweep at 10k points --------------------------------------
    //
    // Sweep `leaf_size` across a representative range to inform the default
    // for 2-D geospatial workloads. We focus on 10k points (large enough to
    // matter, small enough to iterate on quickly) and on query operations
    // relevant to HDBSCAN: k-NN with k=5 (a typical `min_samples`) and
    // fixed-radius neighbour search.
    {
        let leaf_sizes = [2usize, 4, 8, 16, 32, 64];
        let trees: Vec<BallTree<f64>> = leaf_sizes
            .iter()
            .map(|&ls| BallTreeBuilder::with_leaf_size(ls).build(points_10k.clone()))
            .collect();

        // Construction
        {
            let mut group = c.benchmark_group("ball_tree_leaf_size_build_10k");
            for &ls in &leaf_sizes {
                group.bench_with_input(BenchmarkId::from_parameter(ls), &ls, |b, &ls| {
                    b.iter(|| {
                        let tree = BallTreeBuilder::with_leaf_size(ls)
                            .build(black_box(points_10k.clone()));
                        black_box(tree);
                    });
                });
            }
            group.finish();
        }

        // NN
        {
            let mut group = c.benchmark_group("ball_tree_leaf_size_nn_10k");
            for (ls, tree) in leaf_sizes.iter().zip(trees.iter()) {
                group.bench_with_input(BenchmarkId::from_parameter(ls), tree, |b, tree| {
                    b.iter(|| {
                        black_box(tree.nearest_neighbour(black_box(&query)));
                    });
                });
            }
            group.finish();
        }

        // k-NN at k=5 (HDBSCAN-typical min_samples)
        {
            let mut group = c.benchmark_group("ball_tree_leaf_size_knn5_10k");
            for (ls, tree) in leaf_sizes.iter().zip(trees.iter()) {
                group.bench_with_input(BenchmarkId::from_parameter(ls), tree, |b, tree| {
                    b.iter(|| {
                        black_box(tree.nearest_neighbours(black_box(&query), 5));
                    });
                });
            }
            group.finish();
        }

        // Fixed-radius search
        {
            let mut group = c.benchmark_group("ball_tree_leaf_size_radius_10k");
            for (ls, tree) in leaf_sizes.iter().zip(trees.iter()) {
                group.bench_with_input(BenchmarkId::from_parameter(ls), tree, |b, tree| {
                    b.iter(|| {
                        black_box(tree.within_radius(black_box(&query), radius));
                    });
                });
            }
            group.finish();
        }
    }
}

criterion_group!(benches, ball_tree_benchmarks);
criterion_main!(benches);
