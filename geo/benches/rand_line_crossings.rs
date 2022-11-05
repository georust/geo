use std::fmt::Display;

use criterion::{measurement::Measurement, *};
use geo::{Coord, Line};

const BBOX: Coord<f64> = Coord { x: 1024., y: 1024. };

#[path = "utils/random.rs"]
mod random;

#[path = "utils/crossings.rs"]
mod crossings;
use crossings::*;
use random::*;

fn bench_algos<T, F, I>(g: &mut BenchmarkGroup<T>, mut gen: F, sample_size: usize, param: I)
where
    T: Measurement,
    F: FnMut() -> Vec<Line<f64>>,
    I: Display + Copy,
{
    let samples = Samples::from_fn(sample_size, || {
        let lines = gen();
        let expected = count_brute(&lines);
        (lines, expected)
    });

    g.bench_with_input(BenchmarkId::new("Bentley-Ottman", param), &(), |b, _| {
        b.iter_batched(
            samples.sampler(),
            |lines| {
                assert_eq!(count_bo(&lines.0), lines.1);
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_with_input(BenchmarkId::new("Brute-Force", param), &(), |b, _| {
        b.iter_batched(
            samples.sampler(),
            |lines| {
                assert_eq!(count_brute(&lines.0), lines.1);
            },
            BatchSize::SmallInput,
        );
    });
    g.bench_with_input(BenchmarkId::new("R-Tree", param), &(), |b, _| {
        b.iter_batched(
            samples.sampler(),
            |lines| {
                assert_eq!(count_rtree(&lines.0), lines.1);
            },
            BatchSize::SmallInput,
        );
    });
}

fn short<T: Measurement>(c: &mut Criterion<T>) {
    const NUM_LINES: usize = 4096;
    const SAMPLE_SIZE: usize = 10;

    let mut group = c.benchmark_group("Short lines");
    group.sample_size(10);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    (0..10).for_each(|scale| {
        let line_gen = scaled_generator(BBOX, scale);
        let scaling: f64 = (1 << scale) as f64;

        bench_algos(
            &mut group,
            || (0..NUM_LINES).map(|_| line_gen()).collect(),
            SAMPLE_SIZE,
            1. / scaling,
        );
    });
}

fn uniform<T: Measurement>(c: &mut Criterion<T>) {
    const SAMPLE_SIZE: usize = 16;
    const SCALE: usize = 4;

    let mut group = c.benchmark_group("Random lines");
    group.sample_size(2 * SAMPLE_SIZE);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    let line_gen = scaled_generator(BBOX, SCALE);

    (3..12).step_by(2).for_each(|log_num_lines| {
        let num_lines = 1 << log_num_lines;
        bench_algos(
            &mut group,
            || (0..num_lines).map(|_| line_gen()).collect(),
            SAMPLE_SIZE,
            num_lines,
        );
    });
}

fn mixed<T: Measurement>(c: &mut Criterion<T>) {
    const SAMPLE_SIZE: usize = 16;

    let mut group = c.benchmark_group("Mixed");
    group.sample_size(2 * SAMPLE_SIZE);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    (3..12).step_by(2).for_each(|log_num_lines| {
        let num_lines = 1 << log_num_lines;
        bench_algos(
            &mut group,
            || {
                (0..8)
                    .flat_map(|scale| {
                        let line_gen = scaled_generator(BBOX, scale);
                        (0..num_lines / 8).map(move |_| line_gen())
                    })
                    .collect()
            },
            SAMPLE_SIZE,
            num_lines,
        );
    });
}

criterion_group!(random, uniform, short, mixed);
criterion_main!(random);
