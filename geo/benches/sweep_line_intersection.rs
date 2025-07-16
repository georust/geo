//! Benchmarks for the Bentley-Ottmann sweep line intersection algorithm
//!
//! This streamlined benchmark suite compares the sweep line implementation against
//! a brute force O(n²) approach across key scenarios:
//!
//! - Performance comparison across different dataset sizes
//! - Dense intersections (grid patterns)
//! - Sparse large datasets (few intersections, many segments)
//! - Essential edge cases (collinear segments, numerical precision)
//! - Real-world patterns (road networks, polygon boundaries)
//!
//! To run specific benchmark groups:
//! ```
//! cargo bench --bench sweep_line_intersection "Performance Comparison"
//! cargo bench --bench sweep_line_intersection "Dense Line Intersections"
//! cargo bench --bench sweep_line_intersection "Sparse Large Dataset"
//! cargo bench --bench sweep_line_intersection "Essential Edge Cases"
//! cargo bench --bench sweep_line_intersection "Realistic Patterns"
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo::algorithm::line_intersection::line_intersection;
use geo::algorithm::sweep::Intersections as NewSweepIntersections;
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

// Benchmark performance comparison across different dataset sizes
fn bench_performance_comparison(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);

    // Test key sizes: crossover point, medium, and large datasets
    for (n, sample_size) in [
        (10, None),
        (100, None),
        (1_000, Some(100)),
        (10_000, Some(10)),
    ] {
        let mut group = c.benchmark_group(&format!("Performance Comparison ({n} lines)"));
        if let Some(sample_size) = sample_size {
            group.sample_size(sample_size);
        }

        let lines = generate_random_lines(n, &mut rng);

        // Brute force approach
        group.bench_function("brute_force", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });

        // Sweep line algorithm
        group.bench_function("sweep", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });

        group.finish();
    }
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

    group.bench_function("sweep_dense", |b| {
        b.iter(|| {
            let intersections: Vec<_> =
                NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
            black_box(intersections);
        });
    });

    group.finish();
}

// Benchmark with sparse large dataset: should be faster
fn bench_sparse_large_dataset(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sparse Large Dataset");
    group.sample_size(10);

    let mut rng = StdRng::seed_from_u64(42);

    // Generate 10,000 lines with few intersections
    let n = 10_000;
    let mut lines = Vec::with_capacity(n);

    // Create mostly parallel lines with small variations
    for i in 0..n {
        let stripe = i / 100;
        let base_x = (stripe % 20) as f64 * 10.0;
        let base_y = (stripe / 20) as f64 * 10.0;
        let angle = (i % 100) as f64 * 0.005;

        let x1 = base_x + rng.gen_range(-1.0..1.0);
        let y1 = base_y + rng.gen_range(-1.0..1.0);
        let x2 = x1 + 5.0 * angle.cos();
        let y2 = y1 + 5.0 * angle.sin();

        lines.push(Line::from([(x1, y1), (x2, y2)]));
    }

    group.bench_function("sparse_sweep_10000", |b| {
        b.iter(|| {
            let intersections: Vec<_> =
                NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
            black_box(intersections);
        });
    });

    group.bench_function("sparse_brute_force_10000", |b| {
        b.iter(|| {
            black_box(brute_force_intersections(&lines));
        });
    });

    group.finish();
}

fn generate_random_line(rng: &mut impl Rng) -> Line<f64> {
    let x1 = rng.gen_range(-100.0..100.0);
    let y1 = rng.gen_range(-100.0..100.0);
    let x2 = rng.gen_range(-100.0..100.0);
    let y2 = rng.gen_range(-100.0..100.0);
    Line::from([(x1, y1), (x2, y2)])
}

// Benchmark edge cases
fn bench_essential_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("Essential Edge Cases");
    group.sample_size(10);

    let mut rng = StdRng::seed_from_u64(42);

    // Test 1: Collinear segments
    {
        let n = 1000;
        let mut lines = Vec::with_capacity(n);

        // Create many collinear segments on the same line
        for i in 0..n / 4 {
            let start = i as f64 * 2.0;
            let end = start + 1.5;
            lines.push(Line::from([(start, start), (end, end)])); // Diagonal line y=x
        }

        // Add more collinear segments on different lines
        for i in 0..n / 4 {
            let start = i as f64 * 2.0;
            let end = start + 1.5;
            lines.push(Line::from([(start, 0.0), (end, 0.0)])); // Horizontal
            lines.push(Line::from([(0.0, start), (0.0, end)])); // Vertical
            lines.push(Line::from([(start, -start), (end, -end)])); // Diagonal y=-x
        }

        group.bench_function("collinear_segments_sweep", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });

        group.bench_function("collinear_segments_brute_force", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });
    }

    // Test 2: Numerical precision edge cases
    {
        let n = 500;
        let mut lines = Vec::with_capacity(n);

        // Very small coordinate values
        for i in 0..n / 5 {
            let epsilon = 1e-10;
            let offset = i as f64 * epsilon;
            lines.push(Line::from([
                (offset, offset),
                (offset + epsilon, offset + epsilon),
            ]));
            lines.push(Line::from([
                (offset, offset + epsilon),
                (offset + epsilon, offset),
            ]));
        }

        // Very large coordinate values
        for i in 0..n / 5 {
            let large = 1e15;
            let offset = i as f64;
            lines.push(Line::from([
                (large + offset, large),
                (large + offset + 10.0, large + 10.0),
            ]));
            lines.push(Line::from([
                (large + offset, large + 10.0),
                (large + offset + 10.0, large),
            ]));
        }

        // Near-parallel lines with tiny angle differences
        for i in 0..n / 5 {
            let base_x = i as f64 * 10.0;
            let tiny_angle = 1e-8;
            lines.push(Line::from([(base_x, 0.0), (base_x + 100.0, 1.0)]));
            lines.push(Line::from([
                (base_x, 0.0),
                (base_x + 100.0, 1.0 + tiny_angle),
            ]));
        }

        // Power of 2 boundaries
        for i in 0..n / 5 {
            let power = 10 + i % 5; // 2^10 to 2^14
            let boundary = (1 << power) as f64;
            lines.push(Line::from([
                (boundary - 1.0, boundary - 1.0),
                (boundary + 1.0, boundary + 1.0),
            ]));
            lines.push(Line::from([
                (boundary - 1.0, boundary + 1.0),
                (boundary + 1.0, boundary - 1.0),
            ]));
        }

        // Fill remaining with normal lines
        while lines.len() < n {
            lines.push(generate_random_line(&mut rng));
        }

        group.bench_function("numerical_precision_sweep", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });

        group.bench_function("numerical_precision_brute_force", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });
    }

    group.finish();
}

// Additional benchmark for real-world-like patterns
fn bench_realistic_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Realistic Patterns");
    group.sample_size(10);

    let mut rng = StdRng::seed_from_u64(42);

    // Pattern 1: Road network simulation (mostly non-intersecting with some crossings)
    {
        let n = 5000;
        let mut lines = Vec::with_capacity(n);

        // Create a grid-like road network with some irregularity
        let grid_size = 50;
        let cell_size = 10.0;

        // Add "streets" (horizontal and vertical)
        for i in 0..grid_size {
            let pos = i as f64 * cell_size;
            let wiggle = rng.gen_range(-1.0..1.0);

            // Horizontal streets with some "curves"
            for j in 0..grid_size - 1 {
                let x1 = j as f64 * cell_size;
                let x2 = (j + 1) as f64 * cell_size;
                let y1 = pos + wiggle + rng.gen_range(-0.5..0.5);
                let y2 = pos + wiggle + rng.gen_range(-0.5..0.5);
                lines.push(Line::from([(x1, y1), (x2, y2)]));
            }

            // Vertical streets with some curves
            for j in 0..grid_size - 1 {
                let y1 = j as f64 * cell_size;
                let y2 = (j + 1) as f64 * cell_size;
                let x1 = pos + wiggle + rng.gen_range(-0.5..0.5);
                let x2 = pos + wiggle + rng.gen_range(-0.5..0.5);
                lines.push(Line::from([(x1, y1), (x2, y2)]));
            }
        }

        // Trim to exactly n lines
        lines.truncate(n);

        group.bench_function("road_network_sweep", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });

        group.bench_function("road_network_brute_force", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });
    }

    // Pattern 2: Polygon boundaries (building footprints simulation)
    {
        let num_polygons = 200;
        let sides_per_polygon = 8;
        let mut lines = Vec::with_capacity(num_polygons * sides_per_polygon);

        for i in 0..num_polygons {
            let center_x = (i % 20) as f64 * 25.0 + rng.gen_range(-5.0..5.0);
            let center_y = (i / 20) as f64 * 25.0 + rng.gen_range(-5.0..5.0);
            let radius = rng.gen_range(5.0..15.0);

            // Create polygon edges
            for j in 0..sides_per_polygon {
                let angle1 = (j as f64) * 2.0 * std::f64::consts::PI / (sides_per_polygon as f64);
                let angle2 =
                    ((j + 1) as f64) * 2.0 * std::f64::consts::PI / (sides_per_polygon as f64);

                let x1 = center_x + radius * angle1.cos();
                let y1 = center_y + radius * angle1.sin();
                let x2 = center_x + radius * angle2.cos();
                let y2 = center_y + radius * angle2.sin();

                lines.push(Line::from([(x1, y1), (x2, y2)]));
            }
        }

        group.bench_function("polygon_boundaries_sweep", |b| {
            b.iter(|| {
                let intersections: Vec<_> =
                    NewSweepIntersections::<_>::from_iter(lines.iter().cloned()).collect();
                black_box(intersections);
            });
        });

        group.bench_function("polygon_boundaries_brute_force", |b| {
            b.iter(|| {
                black_box(brute_force_intersections(&lines));
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_performance_comparison,
    bench_dense_line_intersections,
    bench_sparse_large_dataset,
    bench_essential_edge_cases,
    bench_realistic_patterns
);
criterion_main!(benches);
