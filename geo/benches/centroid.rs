use criterion::{criterion_group, criterion_main, Criterion};
use geo::Centroid;
use geo::Polygon;
use geo_generic_tests::wkb::reader::read_wkb;
use geo_traits::to_geo::ToGeoGeometry;

#[path = "utils/wkb.rs"]
mod wkb;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("centroid_f32", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).centroid());
        });
    });

    c.bench_function("centroid", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).centroid());
        });
    });

    c.bench_function("centroid_wkb_conv", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let polygon = Polygon::new(norway, vec![]);
        let wkb_bytes = wkb::geo_to_wkb(&polygon);

        bencher.iter(|| {
            let wkb_geom = read_wkb(&wkb_bytes).unwrap();
            let geom = wkb_geom.to_geometry();
            criterion::black_box(geom.centroid());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
