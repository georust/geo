use criterion::{criterion_group, criterion_main, Criterion};
use geo_generic_alg::Area;
use geo_generic_alg::Polygon;
use geo_traits::to_geo::ToGeoGeometry;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("area", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).signed_area());
        });
    });

    c.bench_function("area_wkb", |bencher| {
        let wkb_bytes = geo_test_fixtures::norway_main_wkb();
        bencher.iter(|| {
            let wkb_geom = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes).unwrap();
            criterion::black_box(wkb_geom.signed_area());
        });
    });

    c.bench_function("area_wkb_convert", |bencher| {
        let wkb_bytes = geo_test_fixtures::norway_main_wkb();
        bencher.iter(|| {
            let wkb_geom = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes).unwrap();
            let geom = wkb_geom.to_geometry();
            criterion::black_box(geom.signed_area());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
