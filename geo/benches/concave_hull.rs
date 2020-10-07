#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::algorithm::concave_hull::ConcaveHull;
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
    c.bench_function("concave hull f32", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f32>::from(points);

        bencher.iter(|| {
            line_string.concave_hull(2.0);
        });
    });

    c.bench_function("concave hull f64", |bencher| {
        let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
        let line_string = LineString::<f64>::from(points);

        bencher.iter(|| {
            line_string.concave_hull(2.0);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
