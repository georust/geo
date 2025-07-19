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
//! cargo bench --bench sweep_line_intersection "Random Dense Lines"
//! cargo bench --bench sweep_line_intersection "Random Sparse Lines"
//! cargo bench --bench sweep_line_intersection "Essential Edge Cases"
//! cargo bench --bench sweep_line_intersection "Realistic Patterns"
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo::algorithm::line_intersection::line_intersection;
use geo::algorithm::sweep::Intersections;
use geo::{Destination, Euclidean, Line};
use geo_types::Point;
use rand::prelude::*;

/// Generate a set of random lines
///
/// The units of `density` aren't very meaningful, but a higher value will increase the likelihood of
/// intersections between lines.
/// For example
///   - 0.1 will create relatively short lines, leading to fewer intersections
///   - 4.0 will create relatively long lines, leading to more intersections
fn generate_random_lines(count: usize, density: f64, rng: &mut impl Rng) -> Vec<Line<f64>> {
    let mut lines = Vec::with_capacity(count);
    for _ in 0..count {
        // Generate random coordinates within a bounded area
        // Using a bounded area increases the likelihood of intersections
        let x1 = rng.gen_range(-100.0..100.0);
        let y1 = rng.gen_range(-100.0..100.0);
        let p1 = Point::new(x1, y1);

        let length = 200.0 * density * rng.gen_range(0.0..1.0);
        let p2 = Euclidean.destination(p1, rng.gen_range(0.0..360.0), length);

        lines.push(Line::new(p1, p2))
    }
    lines
}

// Benchmark the brute force approach (O(n²))
fn brute_force_intersections(
    lines: &[Line<f64>],
) -> impl Iterator<Item = (Line<f64>, Line<f64>)> + '_ {
    (0..lines.len()).flat_map(move |i| {
        ((i + 1)..lines.len())
            .flat_map(move |j| line_intersection(lines[i], lines[j]).map(|_| (lines[i], lines[j])))
    })
}

// Benchmark with "dense" case - lines with many intersections.
// When intersections are dense, the sweep algorithm has less of an advantage vs. brute force.
fn bench_dense_line_intersections(c: &mut Criterion) {
    for (n, sample_size, expected_intersections) in [
        (10, None, 7),
        (100, None, 827),
        (1_000, Some(50), 90_438),
        (10_000, Some(10), 8_604_894),
    ] {
        let mut rng = StdRng::seed_from_u64(42);
        let mut group = c.benchmark_group(format!("Random Dense Lines ({n} lines)"));
        if let Some(sample_size) = sample_size {
            group.sample_size(sample_size);
        }
        let lines = generate_random_lines(n, 4.0, &mut rng);

        // Brute force approach
        group.bench_function("brute_force", |b| {
            b.iter(|| {
                let intersections = black_box(brute_force_intersections(&lines)).count();
                assert_eq!(intersections, expected_intersections);
            });
        });

        // Sweep line algorithm
        group.bench_function("sweep", |b| {
            b.iter(|| {
                let intersections = Intersections::from_iter(&lines).count();
                assert_eq!(intersections, expected_intersections);
            });
        });

        group.finish();
    }
}

// Benchmark with "sparse" case - lines with few intersections.
// When intersections are sparse, the sweep algorithm tends to perform much better than brute force.
fn bench_sparse_line_intersections(c: &mut Criterion) {
    for (n, sample_size, expected_intersections) in [
        (10, None, 0),
        (100, None, 0),
        (1_000, Some(50), 11),
        (10_000, Some(10), 798),
    ] {
        let mut rng = StdRng::seed_from_u64(42);
        let mut group = c.benchmark_group(format!("Random Sparse Lines ({n} lines)"));
        if let Some(sample_size) = sample_size {
            group.sample_size(sample_size);
        }
        let lines = generate_random_lines(n, 0.01, &mut rng);

        // Brute force approach
        group.bench_function("brute_force", |b| {
            b.iter(|| {
                let intersections = black_box(brute_force_intersections(&lines)).count();
                assert_eq!(intersections, expected_intersections);
            });
        });

        // Sweep line algorithm
        group.bench_function("sweep", |b| {
            b.iter(|| {
                let intersections = Intersections::from_iter(&lines).count();
                assert_eq!(intersections, expected_intersections);
            });
        });

        group.finish();
    }
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
                let intersections = Intersections::from_iter(&lines).count();
                black_box(intersections);
            });
        });

        group.bench_function("collinear_segments_brute_force", |b| {
            b.iter(|| {
                let intersections = brute_force_intersections(&lines).count();
                black_box(intersections);
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
                let intersections = Intersections::from_iter(&lines).count();
                black_box(intersections);
            });
        });

        group.bench_function("numerical_precision_brute_force", |b| {
            b.iter(|| {
                let intersections = brute_force_intersections(&lines).count();
                black_box(intersections);
            });
        });
    }

    group.finish();
}

// Additional benchmark for real-world-like patterns
fn bench_realistic_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("Realistic Patterns");
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
                let intersections = Intersections::from_iter(&lines).count();
                black_box(intersections);
            });
        });

        group.bench_function("road_network_brute_force", |b| {
            b.iter(|| {
                let intersections = brute_force_intersections(&lines).count();
                black_box(intersections);
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
                let intersections = Intersections::from_iter(&lines).count();
                black_box(intersections);
            });
        });

        group.bench_function("polygon_boundaries_brute_force", |b| {
            b.iter(|| {
                let intersections = brute_force_intersections(&lines).count();
                black_box(intersections);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_dense_line_intersections,
    bench_sparse_line_intersections,
    bench_essential_edge_cases,
    bench_realistic_patterns
);
criterion_main!(benches);
