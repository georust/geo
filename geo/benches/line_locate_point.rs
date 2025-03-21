use criterion::{criterion_group, criterion_main, Criterion};
use geo::{InterpolatableLine, Euclidean, LineLocatePoint};
use geo::LineString;

fn criterion_benchmark(c: &mut Criterion) {
    let test_line: LineString = geo_test_fixtures::norway_main();

    let percentage = 0.45;

    c.bench_function("LineString line_locate_point", |bencher| {
        bencher.iter(|| {
            let point = test_line.point_at_ratio_from_start(&Euclidean, percentage).expect("Did not interpolate out point");
            let percentage_calced = criterion::black_box(test_line.line_locate_point(&point)).expect("Didn't interpolate point back to percentage");

            approx::assert_abs_diff_eq!(percentage, percentage_calced, epsilon = 0.0000001);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);