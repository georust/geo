use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo::algorithm::line_intersection::line_intersection;
use geo::algorithm::new_sweep::Intersections as NewSweepIntersections;
use geo::Line;
use rand::prelude::*;
use std::iter::FromIterator;

// Generate a set of random lines
fn generate_random_lines(count: usize, rng: &mut impl Rng) -> Vec<Line<f64>> {
    let mut lines = Vec::with_capacity(count);
    for _ in 0..count {
        // Generate random coordinates within a bounded area
        // Using a bounded area increases the likelihood of intersections
        let x1 = rng.gen_range(-100.0..100.0);
        let y1 = rng.gen_range(-100.0..100.0);
        let x2 = rng.gen_range(-100.0..100.0);
        let y2 = rng.gen_range(-100.0..100.0);

        lines.push(Line::from([(x1, y1), (x2, y2)]));
    }
    lines
}

// Benchmark the brute force approach (O(n²))
fn brute_force_intersections(lines: &[Line<f64>]) -> Vec<(Line<f64>, Line<f64>)> {
    let mut result = Vec::new();
    for i in 0..lines.len() {
        for j in (i + 1)..lines.len() {
            if line_intersection(lines[i], lines[j]).is_some() {
                result.push((lines[i], lines[j]));
            }
        }
    }
    result
}

// Benchmark finding intersections between lines
// the new algorithm is consistently 1.3x slower than brute force for these workloads
fn bench_line_intersections(c: &mut Criterion) {
    let mut group = c.benchmark_group("Line Intersections");
    group.sample_size(10); // Fewer samples due to longer running time

    // Use a consistent seed for reproducibility
    let mut rng = StdRng::seed_from_u64(42);

    // Test with different numbers of input lines
    for &n in &[5000, 10000] {
        // n=1000 gives ~500k comparisons (n * (n-1) / 2)

        let lines = generate_random_lines(n, &mut rng);

        // Clone lines once to avoid re-generation overhead
        let lines_for_bench = lines.clone();

        // Brute force approach
        group.bench_function(format!("brute_force_n{}", n), |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines_for_bench));
            });
        });

        // New sweep line algorithm
        group.bench_function(format!("new_sweep_n{}", n), |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines_for_bench.iter().cloned())
                        .collect();
                black_box(intersections);
            });
        });
    }

    group.finish();
}

// Benchmark with "dense" case - many lines in small area
// The new algorithm should be faster here
fn bench_dense_line_intersections(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dense Line Intersections");
    group.sample_size(10);

    // Create a grid of lines that will have many intersections
    // expected intersections = (n / 2) * (n / 2);
    let n = 1000;
    let mut lines = Vec::with_capacity(n);

    // Add horizontal and vertical lines (grid-like pattern guarantees many intersections)
    for i in 0..n / 2 {
        let pos = (i as f64) * 2.0 - (n as f64 / 2.0);
        // Horizontal line
        lines.push(Line::from([(-50.0, pos), (50.0, pos)]));
        // Vertical line
        lines.push(Line::from([(pos, -50.0), (pos, 50.0)]));
    }

    // Brute force approach
    group.bench_function("brute_force_dense", |b| {
        b.iter(|| {
            black_box(brute_force_intersections(&lines));
        });
    });

    group.bench_function("new_sweep_dense", |b| {
        b.iter(|| {
            let intersections: Vec<_> =
                NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
            black_box(intersections);
        });
    });

    group.finish();
}

// Benchmark with large sets of line segments that have a moderate number of intersections
// new algorithm is around 1.3x slower here
fn bench_large_datasets(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Datasets");
    group.sample_size(10); // Fewer samples due to longer running time

    // Use a consistent seed for reproducibility
    let mut rng = StdRng::seed_from_u64(42);

    // First test case: 2000 random lines with many potential intersections
    // Sweep line should perform better here as n grows large
    {
        let n = 2000;

        // Random lines across a bounded area - should have many intersections
        let lines = generate_random_lines(n, &mut rng);

        // Brute force approach
        group.bench_function("random_brute_force_2000", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });

        // New sweep line algorithm
        group.bench_function("random_new_sweep_2000", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });
    }

    // Second test case: 2000 mostly-vertical lines with fewer intersections
    // new algorithm should be faster here
    {
        let n = 2000;

        // Generate lines with some structure to limit the number of intersections
        let mut lines = Vec::with_capacity(n);

        // Create mostly-vertical lines with similar slopes to create a realistic scenario
        // with O(n) intersections instead of O(n²)
        for i in 0..n {
            let base_x = (i % 100) as f64 * 2.0;
            let base_y = (i / 100) as f64 * 2.0;

            // Add some randomness
            let x1 = base_x + rng.gen_range(-0.5..0.5);
            let y1 = base_y + rng.gen_range(-20.0..0.0);
            let x2 = base_x + rng.gen_range(-0.5..0.5);
            let y2 = base_y + rng.gen_range(0.0..20.0);

            lines.push(Line::from([(x1, y1), (x2, y2)]));
        }

        // Brute force approach
        group.bench_function("vertical_brute_force_2000", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });

        // New sweep line algorithm
        group.bench_function("vertical_new_sweep_2000", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });
    }

    // Third test case: 10,000 "sparse" lines that don't have many intersections
    // This should be much more favourable to the sweep line algorithms
    {
        let n = 10_000;

        // Generate lines with limited intersection opportunities
        let mut lines = Vec::with_capacity(n);

        // Create parallel-ish lines with small random variations to keep intersection count low
        for i in 0..n {
            let stripe = i / 100; // Each stripe of 100 lines will have similar position/angle
            let base_x = (stripe % 20) as f64 * 10.0;
            let base_y = (stripe / 20) as f64 * 10.0;
            let angle = (i % 100) as f64 * 0.005; // Very small angle variation within stripe

            // Generate line with consistent orientation
            let x1 = base_x + rng.gen_range(-1.0..1.0);
            let y1 = base_y + rng.gen_range(-1.0..1.0);
            let x2 = x1 + 5.0 * angle.cos();
            let y2 = y1 + 5.0 * angle.sin();

            lines.push(Line::from([(x1, y1), (x2, y2)]));
        }

        // Brute force approach
        group.bench_function("sparse_brute_force_10000", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });

        // New sweep line algorithm
        group.bench_function("sparse_new_sweep_10000", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });
    }

    // Fourth test case: 20,000 lines with extremely few intersections
    // Each line is in its own isolated grid cell to minimize intersection opportunities
    // new algorithm should be faster here
    {
        let n = 20_000;

        // Generate lines with minimal intersection opportunities
        let mut lines = Vec::with_capacity(n);

        // We'll put each line in its own 1x1 cell in a large grid
        let grid_size = 200; // 200x200 grid = 40,000 cells

        for i in 0..n {
            let cell_x = i % grid_size;
            let cell_y = i / grid_size;

            // Base coordinates for this cell
            let base_x = cell_x as f64;
            let base_y = cell_y as f64;

            // Generate a short line segment mostly contained in this cell
            // with random orientation to minimize chances of intersection
            let angle = rng.gen_range(0.0..std::f64::consts::PI);
            let length = 0.2; // very short lines

            let x1 = base_x + 0.5 + rng.gen_range(-0.1..0.1);
            let y1 = base_y + 0.5 + rng.gen_range(-0.1..0.1);
            let x2 = x1 + length * angle.cos();
            let y2 = y1 + length * angle.sin();

            lines.push(Line::from([(x1, y1), (x2, y2)]));
        }

        // Brute force approach
        group.bench_function("isolated_brute_force_20000", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });

        // New sweep line algorithm
        group.bench_function("isolated_new_sweep_20000", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_line_intersections,
    bench_dense_line_intersections,
    bench_large_datasets
);
criterion_main!(benches);
