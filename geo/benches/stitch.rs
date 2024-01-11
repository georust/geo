use criterion::{criterion_group, criterion_main, Criterion};
use geo::stitch::Stitch;
use geo::TriangulateSpade;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("stitch", |bencher| {
        let p = geo_test_fixtures::east_baton_rouge::<f32>();
        let tris = p.unconstrained_triangulation().unwrap();

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&tris).stitch_together().unwrap());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
