use criterion::{criterion_group, criterion_main, Criterion};
use geo::algorithm::{Contains, Convert, Relate};
use geo::geometry::*;
use geo::{coord, point, polygon};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("point in simple polygon", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(0.5, 0.1);
        bencher.iter(|| {
            assert!(criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function("point outside simple polygon", |bencher| {
        let polygon = polygon![
            (x: 0.0f64, y: 0.0),
            (x: 1.0, y: 0.0),
            (x: 1.0, y: 1.0),
            (x: 0.0, y: 0.0),
        ];
        let point = Point::new(2.0, 2.0);
        bencher.iter(|| {
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function("point inside complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        let point = geo_test_fixtures::baton_rouge();
        bencher.iter(|| {
            assert!(criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function(
        "point outside, but within bbox, of complex polygon",
        |bencher| {
            let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
            // lake borgne - near and mostly surrounded by, but not inside of, Louisiana
            let point = point!(x: -89.641854, y: 30.026283);
            bencher.iter(|| {
                assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
            });
        },
    );

    c.bench_function("point outside bbox of complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        let point = point!(x: 2.3522, y: 48.8566);
        bencher.iter(|| {
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function(
        "point horizontal to comb teeth aka bart's haircut",
        |bencher| {
            // Testing a pathological case where the point is horizontal to lots of edges.
            //
            // comb teeth -> |\/\/\/\/\/\/|      *  <---point
            //               |____________|
            let polygon = polygon!(
                (x: 0 ,y: 0),
                (x: 0 ,y: 10),
                (x: 1 ,y: 5),
                (x: 2 ,y: 10),
                (x: 3 ,y: 5),
                (x: 4 ,y: 10),
                (x: 5 ,y: 5),
                (x: 6 ,y: 10),
                (x: 7 ,y: 5),
                (x: 8 ,y: 10),
                (x: 9 ,y: 10),
                (x: 10,y:  10),
                (x: 10,y:  0),
                (x: 0 ,y: 0)
            );
            let point = point!(x: 20, y: 7);

            bencher.iter(|| {
                assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&point)));
            })
        },
    );

    c.bench_function("line across complex polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        // crossing part of, but not contained by Louisiana
        let line = Line::new(
            geo_test_fixtures::baton_rouge(),
            point!(x: -89.641854, y: 30.026283),
        );
        bencher.iter(|| {
            assert!(!criterion::black_box(&polygon).contains(criterion::black_box(&line)));
        });
    });

    c.bench_function("complex polygon contains polygon", |bencher| {
        let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
        let contained_polygon = geo_test_fixtures::east_baton_rouge();

        bencher.iter(|| {
            assert!(
                criterion::black_box(&polygon).contains(criterion::black_box(&contained_polygon))
            );
        });
    });

    c.bench_function("Triangle contains point", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (5., 10.)]);
        let point = Point::new(5., 5.);

        bencher.iter(|| {
            assert!(criterion::black_box(&triangle).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function("Triangle contains point on edge", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (6., 10.)]);
        let point = Point::new(3., 5.);

        bencher.iter(|| {
            assert!(!criterion::black_box(&triangle).contains(criterion::black_box(&point)));
        });
    });

    c.bench_function("Rect contains polygon", |bencher| {
        let polygon = polygon![
            (x: 150., y: 350.),
            (x: 100., y: 350.),
            (x: 210., y: 160.),
            (x: 290., y: 350.),
            (x: 250., y: 350.),
            (x: 200., y: 250.),
            (x: 150., y: 350.),
        ];
        let rect = Rect::new(coord! { x: 90., y: 150. }, coord! { x: 300., y: 360. });

        bencher.iter(|| {
            assert!(criterion::black_box(&rect).contains(criterion::black_box(&polygon)));
        });
    });

    c.bench_function(
        "LineString not contains LineString (Contains trait)",
        |bencher| {
            let ls_1: geo::LineString<f64> = geo_test_fixtures::poly1();
            let ls_2: geo::LineString<f64> = geo_test_fixtures::poly2();

            bencher.iter(|| {
                assert!(!ls_1.contains(&ls_2));
            });
        },
    );

    c.bench_function(
        "LineString not contains LineString (Relate trait)",
        |bencher| {
            let ls_1: geo::LineString<f64> = geo_test_fixtures::poly1();
            let ls_2: geo::LineString<f64> = geo_test_fixtures::poly2();

            bencher.iter(|| {
                assert!(!ls_1.relate(&ls_2).is_contains());
            });
        },
    );

    c.bench_function(
        "LineString contains LineString (Contains trait)",
        |bencher| {
            let ls_1: LineString<f64> = geo_test_fixtures::poly1();
            let mut ls_2 = LineString::new(ls_1.0[1..].to_vec());
            ls_2.0.pop();

            bencher.iter(|| {
                assert!(ls_1.contains(&ls_2));
            });
        },
    );

    c.bench_function("LineString contains LineString (Relate trait)", |bencher| {
        let ls_1: geo::LineString<f64> = geo_test_fixtures::poly1();
        let mut ls_2 = LineString::new(ls_1.0[1..].to_vec());
        ls_2.0.pop();

        bencher.iter(|| {
            assert!(ls_1.relate(&ls_2).is_contains());
        });
    });

    c.bench_function("MultiPolygon contains MultiPoint (Contains trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]);
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]);
        let multi_poly = MultiPolygon(vec![p_1, p_2]);
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-60 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            assert!(multi_poly.contains(&multi_point));
        });
    });

    c.bench_function("MultiPolygon contains MultiPoint (Relate trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]);
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]);
        let multi_poly = MultiPolygon(vec![p_1, p_2]);
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-60 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            assert!(multi_poly.relate(&multi_point).is_contains());
        });
    });

    c.bench_function("MultiPolygon not contains MultiPoint (Contains trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]);
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]);
        let multi_poly = MultiPolygon(vec![p_1, p_2]);
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-160 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            assert!(multi_poly.contains(&multi_point));
        });
    });

    c.bench_function("MultiPolygon not contains MultiPoint (Relate trait)", |bencher| {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]);
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]);
        let multi_poly = MultiPolygon(vec![p_1, p_2]);
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-160 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        bencher.iter(|| {
            assert!(multi_poly.relate(&multi_point).is_contains());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
