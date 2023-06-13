#[macro_use]
extern crate criterion;

extern crate geo;

use std::fmt::Display;
use std::panic::catch_unwind;

use criterion::measurement::Measurement;
use geo::monotone::monotone_subdivision;
use geo::{CoordinatePosition, MapCoords, Polygon};

use criterion::{BatchSize, BenchmarkGroup, BenchmarkId, Criterion};
use geo_types::Rect;
use wkt::ToWkt;

#[path = "utils/random.rs"]
mod random;
use rand::thread_rng;
use random::*;

fn criterion_benchmark(c: &mut Criterion) {
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly steppy-polygon (worst case)".to_string());
        let poly = steppy_polygon(&mut thread_rng(), size);
        bench_algos(&mut grp, poly, 512, size)
    }
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly steppy-polygon (best case)".to_string());
        let poly = steppy_polygon(&mut thread_rng(), size).map_coords(|c| (c.y, c.x).into());
        bench_algos(&mut grp, poly, 512, size)
    }
    for size in [16, 64, 512, 1024, 2048] {
        let mut grp = c.benchmark_group("rand pt-in-poly circular-polygon".to_string());
        let poly = circular_polygon(&mut thread_rng(), size);
        bench_algos(&mut grp, poly, 512, size)
    }
}

fn bench_algos<T, I>(g: &mut BenchmarkGroup<T>, polygon: Polygon<f64>, sample_size: usize, param: I)
where
    T: Measurement,
    I: Display + Copy,
{
    let samples = Samples::from_fn(sample_size, || {
        uniform_point(&mut thread_rng(), Rect::new((-1., -1.), (1., 1.)))
    });

    let mon = match catch_unwind(|| monotone_subdivision(polygon.clone())) {
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
                    mon.iter().map(|mp| mp.coordinate_position(pt)).count();
                },
                BatchSize::SmallInput,
            );
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
