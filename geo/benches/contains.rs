#[macro_use]
extern crate criterion;
#[macro_use]
extern crate geo;

use geo::contains::Contains;

use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("point in polygon", |bencher| {
        let polygon = geo::polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let in_candidate = point!(0.5, 0.1);
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&polygon).contains(criterion::black_box(&in_candidate)),
            );
        });
    });

    c.bench_function("point outside polygon", |bencher| {
        let polygon = geo::polygon![
            (x: 0.0, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let out_candidate = point!(2.0, 2.0);
        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&polygon).contains(criterion::black_box(&out_candidate)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
