use criterion::{criterion_group, criterion_main, Criterion};
use geo_generic_alg::algorithm::line_measures::{Euclidean, LengthMeasurableExt};
use geo_generic_alg::Polygon;
use geo_traits::to_geo::ToGeoGeometry;

#[path = "utils/wkb.rs"]
mod wkb;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("perimeter_f32", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).perimeter_ext(&Euclidean));
        });
    });

    c.bench_function("perimeter", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let polygon = Polygon::new(norway, vec![]);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&polygon).perimeter_ext(&Euclidean));
        });
    });

    c.bench_function("perimeter_wkb", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let polygon = Polygon::new(norway, vec![]);
        let wkb_bytes = wkb::geo_to_wkb(&polygon);

        bencher.iter(|| {
            let wkb_geom = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes).unwrap();
            criterion::black_box(wkb_geom.perimeter_ext(&Euclidean));
        });
    });

    c.bench_function("perimeter_wkb_convert", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let polygon = Polygon::new(norway, vec![]);
        let wkb_bytes = wkb::geo_to_wkb(&polygon);

        bencher.iter(|| {
            let wkb_geom = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes).unwrap();
            let geom = wkb_geom.to_geometry();
            criterion::black_box(geom.perimeter_ext(&Euclidean));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
