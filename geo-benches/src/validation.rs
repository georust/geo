//! Benchmarks for polygon validation, focusing on:
//! - Checkerboard patterns (stress test simply-connected interior detection)
//! - Complex geometries from real-world datasets

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use geo::algorithm::Validation;
use geo::coord;
use geo::geometry::{Geometry, LineString, Polygon};
use geo_test_fixtures::checkerboard::create_checkerboard_polygon;
use geojson::GeoJson;
use std::convert::TryInto;
use std::fs;
use std::path::PathBuf;

/// Load benchmark geometries from the validate.geojson fixture.
/// Returns a vector of (name, polygon) tuples.
fn load_benchmark_geometries() -> Vec<(String, Polygon<f64>)> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/rust-geo-booleanop-fixtures/benchmarks/validate.geojson");

    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "Warning: Could not read {}: {}, skipping validation benchmarks",
                path.display(),
                e
            );
            return vec![];
        }
    };

    let geojson: GeoJson = match data.parse() {
        Ok(g) => g,
        Err(e) => {
            eprintln!(
                "Warning: Could not parse GeoJSON: {}, skipping validation benchmarks",
                e
            );
            return vec![];
        }
    };

    let mut results = Vec::new();

    if let GeoJson::FeatureCollection(fc) = geojson {
        for (idx, feature) in fc.features.into_iter().enumerate().take(20) {
            if let Some(geom) = feature.geometry {
                let geo_geom: Result<Geometry<f64>, _> = geom.try_into();
                if let Ok(Geometry::MultiPolygon(mp)) = geo_geom {
                    // Extract individual polygons from the MultiPolygon
                    for (poly_idx, poly) in mp.0.into_iter().enumerate() {
                        let num_coords: usize = poly.exterior().coords().count()
                            + poly
                                .interiors()
                                .iter()
                                .map(|r| r.coords().count())
                                .sum::<usize>();
                        let name = format!("feat_{}_poly_{}_{}_coords", idx, poly_idx, num_coords);
                        results.push((name, poly));
                    }
                } else if let Ok(Geometry::Polygon(poly)) = geo_geom {
                    let num_coords: usize = poly.exterior().coords().count()
                        + poly
                            .interiors()
                            .iter()
                            .map(|r| r.coords().count())
                            .sum::<usize>();
                    let name = format!("feat_{}_{}_coords", idx, num_coords);
                    results.push((name, poly));
                }
            }
        }
    }

    results
}

/// Create a simple valid polygon for baseline comparison.
fn create_simple_polygon(size: usize) -> Polygon<f64> {
    let n = size;
    let mut coords = Vec::with_capacity(n + 1);
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        coords.push(coord! { x: angle.cos() * 100.0, y: angle.sin() * 100.0 });
    }
    coords.push(coords[0]); // Close the ring
    Polygon::new(LineString::new(coords), vec![])
}

fn validation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("polygon_validation");

    // Benchmark simple polygons (baseline - no holes)
    for size in [10, 100, 1000] {
        let simple = create_simple_polygon(size);
        group.bench_with_input(
            BenchmarkId::new("simple_polygon", size),
            &simple,
            |b, poly| b.iter(|| criterion::black_box(poly.is_valid())),
        );
    }

    // Benchmark checkerboard patterns (many holes touching at vertices)
    // These stress-test the simply-connected interior detection
    for level in 0..=3 {
        let checkerboard = create_checkerboard_polygon(level);
        let num_holes = checkerboard.interiors().len();
        let label = format!("level_{}_{}holes", level, num_holes);
        group.bench_with_input(
            BenchmarkId::new("checkerboard", &label),
            &checkerboard,
            |b, poly| b.iter(|| criterion::black_box(poly.is_valid())),
        );
    }

    // Benchmark complex geometries from fixture file
    let fixture_polys = load_benchmark_geometries();
    for (name, poly) in fixture_polys.iter().take(15) {
        group.bench_with_input(BenchmarkId::new("fixture", name), poly, |b, p| {
            b.iter(|| criterion::black_box(p.is_valid()))
        });
    }

    group.finish();
}

criterion_group!(benches, validation_benchmark);
criterion_main!(benches);
