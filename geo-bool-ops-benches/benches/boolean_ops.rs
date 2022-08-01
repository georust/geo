use std::f64::consts::PI;

use criterion::{measurement::Measurement, *};
use geo::{
    algorithm::{BooleanOps, Rotate},
    Relate,
};

use geo_booleanop::boolean::BooleanOp as OtherBooleanOp;
use rand::{thread_rng, Rng};
use rand_distr::Standard;

#[path = "../../geo/benches/utils/random.rs"]
mod random;
use random::Samples;

#[path = "utils/bops.rs"]
mod bops;
use bops::convert_poly;

fn run_complex<T: Measurement>(c: &mut Criterion<T>) {
    const SAMPLE_SIZE: usize = 16;
    let mut group = c.benchmark_group("Circular polygon boolean-ops");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    (8..14).for_each(|scale| {
        let steps = 1 << scale;
        let polys = Samples::from_fn(SAMPLE_SIZE, || {
            let poly1 = random::steppy_polygon(thread_rng(), steps);
            let angle: f64 = thread_rng().sample::<f64, _>(Standard) * PI * 2.0;
            let poly1 = poly1.rotate_around_point(angle, poly1.exterior().0[0].into());

            let poly2 = random::circular_polygon(thread_rng(), steps);
            let angle: f64 = thread_rng().sample::<f64, _>(Standard) * PI * 2.0;
            let poly2 = poly2.rotate_around_point(angle, poly2.exterior().0[0].into());

            let cp1 = convert_poly(&poly1);
            let cp2 = convert_poly(&poly2);
            (poly1, poly2, cp1, cp2)
        });

        group.sample_size(10);
        group.bench_with_input(BenchmarkId::new("bops::union", steps), &(), |b, _| {
            b.iter_batched(
                polys.sampler(),
                |&(ref poly, ref poly2, _, _)| poly.intersection(poly2),
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("bops::intersection", steps),
            &(),
            |b, _| {
                b.iter_batched(
                    polys.sampler(),
                    |&(ref poly, ref poly2, _, _)| poly.union(poly2),
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(BenchmarkId::new("rgbops::union", steps), &(), |b, _| {
            b.iter_batched(
                polys.sampler(),
                |&(_, _, ref poly, ref poly2)| poly.intersection(poly2),
                BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("rgbops::intersection", steps),
            &(),
            |b, _| {
                b.iter_batched(
                    polys.sampler(),
                    |&(_, _, ref poly, ref poly2)| poly.union(poly2),
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(BenchmarkId::new("geo::relate", steps), &(), |b, _| {
            b.iter_batched(
                polys.sampler(),
                |&(ref poly, ref poly2, _, _)| poly.relate(poly2).is_intersects(),
                BatchSize::SmallInput,
            );
        });
    });
}

criterion_group!(verts_vs_time, run_complex);
criterion_main!(verts_vs_time);
