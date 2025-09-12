use criterion::{criterion_group, criterion_main, Criterion};
use geo_generic_alg::algorithm::line_measures::{Distance, DistanceExt, Euclidean};
use geo_generic_alg::{coord, LineString, MultiPolygon, Point, Polygon};

#[path = "utils/wkb.rs"]
mod wkb;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("distance_point_to_point", |bencher| {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(100.0, 100.0);

        bencher.iter(|| {
            criterion::black_box(criterion::black_box(&p1).distance_ext(criterion::black_box(&p2)));
        });
    });

    c.bench_function("distance_linestring_to_linestring", |bencher| {
        let ls1 = geo_test_fixtures::norway_main::<f64>();
        let ls2 = LineString::from(vec![
            coord!(x: 100.0, y: 100.0),
            coord!(x: 200.0, y: 200.0),
            coord!(x: 300.0, y: 300.0),
        ]);

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&ls1).distance_ext(criterion::black_box(&ls2)),
            );
        });
    });

    c.bench_function("distance_polygon_to_polygon", |bencher| {
        let poly1 = Polygon::new(
            LineString::from(vec![
                coord!(x: 0.0, y: 0.0),
                coord!(x: 100.0, y: 0.0),
                coord!(x: 100.0, y: 100.0),
                coord!(x: 0.0, y: 100.0),
                coord!(x: 0.0, y: 0.0),
            ]),
            vec![],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![
                coord!(x: 200.0, y: 200.0),
                coord!(x: 300.0, y: 200.0),
                coord!(x: 300.0, y: 300.0),
                coord!(x: 200.0, y: 300.0),
                coord!(x: 200.0, y: 200.0),
            ]),
            vec![],
        );

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&poly1).distance_ext(criterion::black_box(&poly2)),
            );
        });
    });

    c.bench_function("distance_wkb_point_to_point", |bencher| {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(100.0, 100.0);
        let wkb_bytes1 = wkb::geo_to_wkb(p1);
        let wkb_bytes2 = wkb::geo_to_wkb(p2);

        bencher.iter(|| {
            let wkb_geom1 = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes1).unwrap();
            let wkb_geom2 = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes2).unwrap();
            criterion::black_box(wkb_geom1.distance_ext(&wkb_geom2));
        });
    });

    c.bench_function("distance_wkb_linestring_to_linestring", |bencher| {
        let ls1 = geo_test_fixtures::norway_main::<f64>();
        let ls2 = LineString::from(vec![
            coord!(x: 100.0, y: 100.0),
            coord!(x: 200.0, y: 200.0),
            coord!(x: 300.0, y: 300.0),
        ]);
        let wkb_bytes1 = wkb::geo_to_wkb(&ls1);
        let wkb_bytes2 = wkb::geo_to_wkb(&ls2);

        bencher.iter(|| {
            let wkb_geom1 = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes1).unwrap();
            let wkb_geom2 = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes2).unwrap();
            criterion::black_box(wkb_geom1.distance_ext(&wkb_geom2));
        });
    });

    c.bench_function("distance_multipolygon_to_multipolygon", |bencher| {
        let poly1 = Polygon::new(
            LineString::from(vec![
                coord!(x: 0.0, y: 0.0),
                coord!(x: 50.0, y: 0.0),
                coord!(x: 50.0, y: 50.0),
                coord!(x: 0.0, y: 50.0),
                coord!(x: 0.0, y: 0.0),
            ]),
            vec![],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![
                coord!(x: 60.0, y: 60.0),
                coord!(x: 110.0, y: 60.0),
                coord!(x: 110.0, y: 110.0),
                coord!(x: 60.0, y: 110.0),
                coord!(x: 60.0, y: 60.0),
            ]),
            vec![],
        );
        let mp1 = MultiPolygon::new(vec![poly1.clone(), poly1]);
        let mp2 = MultiPolygon::new(vec![poly2.clone(), poly2]);

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&mp1).distance_ext(criterion::black_box(&mp2)),
            );
        });
    });

    c.bench_function("distance_concrete_point_to_point", |bencher| {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(100.0, 100.0);

        bencher.iter(|| {
            criterion::black_box(
                Euclidean.distance(criterion::black_box(p1), criterion::black_box(p2)),
            );
        });
    });

    c.bench_function("distance_concrete_linestring_to_linestring", |bencher| {
        let ls1 = geo_test_fixtures::norway_main::<f64>();
        let ls2 = LineString::from(vec![
            coord!(x: 100.0, y: 100.0),
            coord!(x: 200.0, y: 200.0),
            coord!(x: 300.0, y: 300.0),
        ]);

        bencher.iter(|| {
            criterion::black_box(
                Euclidean.distance(criterion::black_box(&ls1), criterion::black_box(&ls2)),
            );
        });
    });

    c.bench_function("distance_cross_type_point_to_linestring", |bencher| {
        let point = Point::new(50.0, 50.0);
        let linestring = LineString::from(vec![
            coord!(x: 0.0, y: 0.0),
            coord!(x: 100.0, y: 100.0),
            coord!(x: 200.0, y: 0.0),
        ]);

        bencher.iter(|| {
            criterion::black_box(Euclidean.distance(
                criterion::black_box(&point),
                criterion::black_box(&linestring),
            ));
        });
    });

    c.bench_function("distance_cross_type_linestring_to_polygon", |bencher| {
        let linestring =
            LineString::from(vec![coord!(x: -50.0, y: 50.0), coord!(x: 150.0, y: 50.0)]);
        let polygon = Polygon::new(
            LineString::from(vec![
                coord!(x: 0.0, y: 0.0),
                coord!(x: 100.0, y: 0.0),
                coord!(x: 100.0, y: 100.0),
                coord!(x: 0.0, y: 100.0),
                coord!(x: 0.0, y: 0.0),
            ]),
            vec![],
        );

        bencher.iter(|| {
            criterion::black_box(Euclidean.distance(
                criterion::black_box(&linestring),
                criterion::black_box(&polygon),
            ));
        });
    });

    c.bench_function("distance_cross_type_point_to_polygon", |bencher| {
        let point = Point::new(150.0, 50.0);
        let polygon = Polygon::new(
            LineString::from(vec![
                coord!(x: 0.0, y: 0.0),
                coord!(x: 100.0, y: 0.0),
                coord!(x: 100.0, y: 100.0),
                coord!(x: 0.0, y: 100.0),
                coord!(x: 0.0, y: 0.0),
            ]),
            vec![],
        );

        bencher.iter(|| {
            criterion::black_box(
                Euclidean.distance(criterion::black_box(&point), criterion::black_box(&polygon)),
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
