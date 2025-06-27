use std::fmt::Display;
use std::panic::catch_unwind;

use criterion::measurement::Measurement;
use geo::coordinate_position::CoordPos;
use geo::monotone::monotone_subdivision;
use geo::{CoordinatePosition, MapCoords, Polygon};

use criterion::{
    criterion_group, criterion_main, BatchSize, BenchmarkGroup, BenchmarkId, Criterion,
};
use geo::{Coord, Rect};
use wkt::ToWkt;

#[path = "utils/random.rs"]
mod random;
use rand::thread_rng;
use random::*;

fn criterion_benchmark_pt_in_poly(c: &mut Criterion) {
    let pt_samples = Samples::from_fn(512, || {
        uniform_point(&mut thread_rng(), Rect::new((-1., -1.), (1., 1.)))
    });

    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly steppy-polygon (worst case)".to_string());
        let poly = steppy_polygon(&mut thread_rng(), size);
        bench_pt_in_poly(&mut grp, poly, size, &pt_samples)
    }
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly steppy-polygon (best case)".to_string());
        let poly = steppy_polygon(&mut thread_rng(), size).map_coords(|c| (c.y, c.x).into());
        bench_pt_in_poly(&mut grp, poly, size, &pt_samples)
    }
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly circular-polygon".to_string());
        let poly = circular_polygon(&mut thread_rng(), size);
        bench_pt_in_poly(&mut grp, poly, size, &pt_samples)
    }
}

fn bench_pt_in_poly<T, I>(
    g: &mut BenchmarkGroup<T>,
    polygon: Polygon<f64>,
    param: I,
    samples: &Samples<Coord<f64>>,
) where
    T: Measurement,
    I: Display + Copy,
{
    let mon = match catch_unwind(|| monotone_subdivision([polygon.clone()])) {
        Ok(m) => m,
        Err(_) => {
            panic!(
                "Monotone subdivision failed for polygon: {}",
                polygon.to_wkt()
            );
        }
    };

    g.bench_with_input(
        BenchmarkId::new("Simple point-in-poly", param),
        &(),
        |b, _| {
            b.iter_batched(
                samples.sampler(),
                |pt| {
                    polygon.coordinate_position(pt);
                },
                BatchSize::SmallInput,
            );
        },
    );
    g.bench_with_input(
        BenchmarkId::new("Pre-processed point-in-poly", param),
        &(),
        |b, _| {
            b.iter_batched(
                samples.sampler(),
                |pt| {
                    mon.iter()
                        .filter(|mp| mp.coordinate_position(pt) == CoordPos::Inside)
                        .count();
                },
                BatchSize::SmallInput,
            );
        },
    );
}

fn criterion_benchmark_monotone_subdiv(c: &mut Criterion) {
    for size in [16, 64, 2048, 32000] {
        let mut grp = c.benchmark_group("monotone_subdiv steppy-polygon (worst case)".to_string());
        let poly_fn = |size| steppy_polygon(&mut thread_rng(), size);
        bench_monotone_subdiv(&mut grp, poly_fn, size)
    }
    for size in [16, 64, 2048, 32000] {
        let mut grp = c.benchmark_group("monotone_subdiv steppy-polygon (best case)".to_string());
        let poly_fn =
            |size| steppy_polygon(&mut thread_rng(), size).map_coords(|c| (c.y, c.x).into());
        bench_monotone_subdiv(&mut grp, poly_fn, size)
    }
    for size in [16, 64, 2048, 32000] {
        let mut grp = c.benchmark_group("monotone_subdiv circular-polygon".to_string());
        let poly_fn = |size| circular_polygon(&mut thread_rng(), size);
        bench_monotone_subdiv(&mut grp, poly_fn, size)
    }
}

fn bench_monotone_subdiv<T, F>(g: &mut BenchmarkGroup<T>, mut f: F, param: usize)
where
    T: Measurement,
    F: FnMut(usize) -> Polygon<f64>,
{
    let samples = Samples::from_fn(16, || f(param));
    g.bench_with_input(
        BenchmarkId::new("Montone subdivision", param),
        &(),
        |b, _| {
            b.iter_batched(
                samples.sampler(),
                |pt| {
                    let mon = monotone_subdivision([pt.clone()]);
                    mon.len();
                },
                BatchSize::SmallInput,
            );
        },
    );
}

criterion_group!(
    benches,
    criterion_benchmark_pt_in_poly,
    criterion_benchmark_monotone_subdiv
);
criterion_main!(benches);
