#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::prelude::*;
use geo::{Coordinate, CoordinateType, LineString};

use num_traits::Signed;
use rand::distributions::uniform::SampleUniform;
use rand::Rng;
pub fn uniform_points_in_range<S: CoordinateType + SampleUniform + Signed, R: Rng>(
    range: S,
    size: usize,
    rng: &mut R,
) -> Vec<Coordinate<S>> {
    (0..size)
        .map(|_| (rng.gen_range(-range, range), rng.gen_range(-range, range)).into())
        .collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("convex hull f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f32>::from(points);

        bencher.iter(|| {
            line_string.convex_hull();
        });
    });

    c.bench_function("convex hull f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f64>::from(points);

        bencher.iter(|| {
            line_string.convex_hull();
        });
    });

    c.bench_function("convex hull with collinear random i64", |bencher| {
        let mut points = uniform_points_in_range(10_000_i64, 1_000_000, &mut rand::thread_rng());
        use geo::algorithm::convex_hull::graham_hull;
        bencher.iter(|| {
            graham_hull(&mut points, true);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
