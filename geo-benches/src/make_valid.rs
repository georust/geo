//! Benchmarks for `MakeValid::make_valid`.
//!
//! The snap radius depends on the scale of the input, so the same plots are
//! repaired in geographic (WGS84) and projected (EPSG:28992) coordinates.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use geo::algorithm::MakeValid;
use geo::coord;
use geo::geometry::{LineString, MultiPolygon, Polygon};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use std::hint::black_box;

/// Make a self-intersecting polygon with `n` pseudo-random vertices in a 1000 x 1000 box.
///
/// The fixed seed gives the same polygon on each run, so results can be
/// compared with a saved baseline. `StdRng` output can change between `rand`
/// versions: compare only against a baseline from the same `rand` version.
fn random_self_intersecting(n: usize) -> Polygon<f64> {
    let mut rng = StdRng::seed_from_u64(42);
    let coords: Vec<_> = (0..n)
        .map(|_| coord! { x: rng.random_range(0.0..1000.0), y: rng.random_range(0.0..1000.0) })
        .collect();
    Polygon::new(LineString::new(coords), vec![])
}

fn make_valid_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("make_valid");

    let east_baton_rouge: Polygon<f64> = geo_test_fixtures::east_baton_rouge();
    group.bench_with_input(
        BenchmarkId::new("polygon", "east_baton_rouge_wgs84"),
        &east_baton_rouge,
        |b, p| b.iter(|| black_box(p.make_valid().unwrap())),
    );

    let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
    group.bench_with_input(
        BenchmarkId::new("polygon", "norway_main_wgs84"),
        &norway,
        |b, p| b.iter(|| black_box(p.make_valid().unwrap())),
    );

    let nl_zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    group.bench_with_input(
        BenchmarkId::new("multi_polygon", "nl_zones_wgs84"),
        &nl_zones,
        |b, mp| b.iter(|| black_box(mp.make_valid().unwrap())),
    );

    for (name, plots) in [
        ("nl_plots_wgs84", geo_test_fixtures::nl_plots_wgs84::<f64>()),
        (
            "nl_plots_epsg_28992",
            geo_test_fixtures::nl_plots_epsg_28992::<f64>(),
        ),
    ] {
        group.bench_with_input(BenchmarkId::new("multi_polygon", name), &plots, |b, mp| {
            b.iter(|| black_box(mp.make_valid().unwrap()))
        });
        // Repair each plot separately, as for a parcel layer.
        group.bench_with_input(BenchmarkId::new("each_polygon", name), &plots, |b, mp| {
            b.iter(|| {
                for p in mp {
                    black_box(p.make_valid().unwrap());
                }
            })
        });
    }

    for n in [100, 1000] {
        let p = random_self_intersecting(n);
        group.bench_with_input(
            BenchmarkId::new("random_self_intersecting", n),
            &p,
            |b, p| b.iter(|| black_box(p.make_valid().unwrap())),
        );
    }

    group.finish();
}

criterion_group!(benches, make_valid_benchmark);
criterion_main!(benches);
