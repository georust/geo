use criterion::{criterion_group, criterion_main, Criterion};
use geo::Area;
use geo::Polygon;
use geo_generic_tests::wkb::reader::read_wkb;
use geo_traits::to_geo::ToGeoGeometry;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("area", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).signed_area());
        });
    });

    c.bench_function("area_wkb_conv", |bencher| {
        let wkb_bytes = geo_test_fixtures::norway_main_wkb();
        bencher.iter(|| {
            let wkb_geom = read_wkb(&wkb_bytes).unwrap();
            let geom = wkb_geom.to_geometry();
            criterion::black_box(geom.signed_area());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
