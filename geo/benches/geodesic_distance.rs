use criterion::{criterion_group, criterion_main};
use geo::{Distance, Geodesic};

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("geodesic distance f64", |bencher| {
        let a = geo::Point::new(17.107558, 48.148636);
        let b = geo::Point::new(16.372477, 48.208810);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(Geodesic::distance(a, b)));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
