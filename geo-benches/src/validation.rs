//! Benchmarks for polygon validation, focusing on:
//! - Checkerboard patterns (stress test simply-connected interior detection)
//! - Simple and real-world polygon geometries

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use geo::algorithm::Validation;
use geo::coord;
use geo::geometry::{LineString, Polygon};
use geo_test_fixtures::checkerboard::create_checkerboard_polygon;

fn create_simple_polygon(size: usize) -> Polygon<f64> {
    let n = size;
    let mut coords = Vec::with_capacity(n + 1);
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        coords.push(coord! { x: angle.cos() * 100.0, y: angle.sin() * 100.0 });
    }
    coords.push(coords[0]);
    Polygon::new(LineString::new(coords), vec![])
}

fn validation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("polygon_validation");

    for size in [10, 100, 1000] {
        let simple = create_simple_polygon(size);
        group.bench_with_input(
            BenchmarkId::new("simple_polygon", size),
            &simple,
            |b, poly| b.iter(|| criterion::black_box(poly.is_valid())),
        );
    }

    for level in 0..=3 {
        let checkerboard = create_checkerboard_polygon(level);
        let num_holes = checkerboard.interiors().len();
        let label = format!("level_{level}_{num_holes}holes");
        group.bench_with_input(
            BenchmarkId::new("checkerboard", &label),
            &checkerboard,
            |b, poly| b.iter(|| criterion::black_box(poly.is_valid())),
        );
    }

    let east_baton_rouge: Polygon<f64> = geo_test_fixtures::east_baton_rouge();
    group.bench_with_input(
        BenchmarkId::new("fixture", "east_baton_rouge"),
        &east_baton_rouge,
        |b, poly| b.iter(|| criterion::black_box(poly.is_valid())),
    );

    let nl_zones = geo_test_fixtures::nl_zones::<f64>();
    group.bench_with_input(BenchmarkId::new("fixture", "nl_zones"), &nl_zones, |b, mp| {
        b.iter(|| criterion::black_box(mp.is_valid()))
    });

    let nl_plots = geo_test_fixtures::nl_plots_wgs84::<f64>();
    group.bench_with_input(BenchmarkId::new("fixture", "nl_plots"), &nl_plots, |b, mp| {
        b.iter(|| criterion::black_box(mp.is_valid()))
    });

    group.finish();
}

criterion_group!(benches, validation_benchmark);
criterion_main!(benches);
