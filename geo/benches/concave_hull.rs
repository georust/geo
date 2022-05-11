#[macro_use]
extern crate criterion;
extern crate geo;

use criterion::Criterion;
use geo::ConcaveHull;
use geo::{CoordNum, Coordinate};

use num_traits::Signed;
use rand::distributions::uniform::SampleUniform;
use rand::Rng;

pub fn uniform_points_in_range<S: CoordNum + SampleUniform + Signed, R: Rng>(
    range: S,
    size: usize,
    rng: &mut R,
) -> Vec<Coordinate<S>> {
    (0..size)
        .map(|_| (rng.gen_range(-range..=range), rng.gen_range(-range..=range)).into())
        .collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("concave hull f32", |bencher| {
        let line_string = geo_test_fixtures::norway_main::<f32>();

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&line_string).concave_hull(criterion::black_box(2.0)),
            );
        });
    });

    c.bench_function("concave hull f64", |bencher| {
        let line_string = geo_test_fixtures::norway_main::<f64>();

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&line_string).concave_hull(criterion::black_box(2.0)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
