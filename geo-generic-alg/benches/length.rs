use criterion::{criterion_group, criterion_main, Criterion};
use geo_generic_alg::algorithm::line_measures::{Euclidean, LengthMeasurableExt};
use geo_traits::to_geo::ToGeoGeometry;

#[path = "utils/wkb.rs"]
mod wkb;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("length_f32", |bencher| {
        let linestring = geo_test_fixtures::norway_main::<f32>();

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&linestring).length_ext(&Euclidean));
        });
    });

    c.bench_function("length", |bencher| {
        let linestring = geo_test_fixtures::norway_main::<f64>();

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&linestring).length_ext(&Euclidean));
        });
    });

    c.bench_function("length_wkb", |bencher| {
        let linestring = geo_test_fixtures::norway_main::<f64>();
        let wkb_bytes = wkb::geo_to_wkb(&linestring);

        bencher.iter(|| {
            let wkb_geom = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes).unwrap();
            criterion::black_box(wkb_geom.length_ext(&Euclidean));
        });
    });

    c.bench_function("length_wkb_convert", |bencher| {
        let linestring = geo_test_fixtures::norway_main::<f64>();
        let wkb_bytes = wkb::geo_to_wkb(&linestring);

        bencher.iter(|| {
            let wkb_geom = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes).unwrap();
            let geom = wkb_geom.to_geometry();
            criterion::black_box(geom.length_ext(&Euclidean));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
