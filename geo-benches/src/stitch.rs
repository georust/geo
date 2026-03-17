use criterion::{Criterion, criterion_group, criterion_main};
use geo::algorithm::bool_ops::unary_union;
#[allow(deprecated)]
use geo::algorithm::{StitchTriangles, TriangulateEarcut};

fn criterion_benchmark(c: &mut Criterion) {
    let p = geo_test_fixtures::east_baton_rouge::<f32>();
    let tris = p.earcut_triangles();
    c.bench_function("stitch triangles", |bencher| {
        bencher.iter(|| {
            #[allow(deprecated)]
            criterion::black_box(criterion::black_box(&tris).stitch_triangulation().unwrap());
        });
    });
    c.bench_function("union triangles via boolean ops", |bencher| {
        bencher.iter(|| {
            let polys: Vec<_> = tris.iter().map(|tri| tri.to_polygon()).collect();
            criterion::black_box(unary_union(criterion::black_box(&polys)))
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
