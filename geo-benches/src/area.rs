use criterion::{Criterion, criterion_group, criterion_main};
use geo::Area;
use geo::Polygon;
use geo::TriangulateEarcut;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("area", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).signed_area());
        });
    });

    c.bench_function("triangle area", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);
        let triangles = polygon.earcut_triangles();

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&triangles)
                    .iter()
                    .fold(0.0, |area, tri| area + tri.signed_area()),
            );
        })
    });

    c.bench_function("multipolygon area", |bencher| {
        let nl_plots = geo_test_fixtures::nl_plots_wgs84::<f32>();

        bencher.iter(|| criterion::black_box(criterion::black_box(&nl_plots).signed_area()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
