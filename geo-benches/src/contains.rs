use criterion::{Criterion, criterion_group, criterion_main};
use geo::algorithm::{Contains, Convert, Relate};
use geo::coordinate_position::CoordPos;
use geo::relate::IntersectionMatrix;
use geo::{BoundingRect, coord, point, polygon};
use geo::{CoordinatePosition, geometry::*};

#[path = "utils/compare_impl.rs"]
mod compare_impl;

#[path = "utils/random.rs"]
mod random;
use rand::rng;
use random::*;

const NUMPOINTS: i32 = 1_000_000;

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

    trait_vs_relate!(
        c,
        "LineString not contains LineString",
        geo_test_fixtures::poly1::<f64>(),
        geo_test_fixtures::poly2::<f64>(),
        Contains::contains,
        IntersectionMatrix::is_contains,
        false
    );

    {
        let ls_1: LineString<f64> = geo_test_fixtures::poly1();
        let ls_2 = LineString::new(ls_1.0[1..].to_vec());

        trait_vs_relate!(
            c,
            "LineString contains LineString",
            ls_1,
            ls_2,
            Contains::contains,
            IntersectionMatrix::is_contains,
            true
        );
    }

    let multi_poly: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    // these points are guaranteed to fall within the mp
    let bounds = Rect::new(
        Coord {
            x: 5.555,
            y: 52.341,
        },
        Coord { x: 5.57, y: 52.351 },
    );
    let points: Vec<Point<f64>> = (0..NUMPOINTS)
        .map(|_| Point(uniform_point(&mut rng(), bounds)))
        .collect();
    let multi_point = MultiPoint::new(points);

    c.bench_function(
        "MultiPolygon contains MultiPoint (Contains trait)",
        |bencher| {
            bencher.iter(|| {
                assert!(multi_poly.contains(&multi_point));
            });
        },
    );

    c.bench_function(
        "MultiPolygon contains MultiPoint (Relate trait)",
        |bencher| {
            let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]);
            let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]);
            let multi_poly = MultiPolygon(vec![p_1, p_2]);
            let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-60 10,-60 -70,-120 -70, 
    -120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100  
    -230,-185 -230))
            .convert();

            bencher.iter(|| {
                assert!(multi_poly.relate(&multi_point).is_contains());
            });
        },
    );

    {
        let p_1: Polygon<f64> = Polygon::new(geo_test_fixtures::poly1(), vec![]);
        let p_2: Polygon<f64> = Polygon::new(geo_test_fixtures::poly2(), vec![]);
        let multi_poly = MultiPolygon(vec![p_1, p_2]);
        let multi_point: MultiPoint<f64> = geo::wkt!(MULTIPOINT (-160 10,-60 -70,-120 -70,-120 10,-40 80,30 80,30 10,-40 10,100 210,100 120,30 120,30 210,-185 -135,-100 -135,-100 -230,-185 -230)).convert();

        trait_vs_relate!(
            c,
            "MultiPolygon contains MultiPoint",
            multi_poly,
            multi_point,
            Contains::contains,
            IntersectionMatrix::is_contains,
            true
        );
    }

    let zones: MultiPolygon<f64> = geo_test_fixtures::nl_zones();
    let bound = zones.bounding_rect().unwrap();
    let mut coords = vec![];

    // Generate a bunch of points inside the zone bounds
    let size = 20;
    let mut x = bound.min().x;
    for _ in 0..=size {
        let mut y = bound.min().y;
        for _ in 0..=size {
            coords.push(Coord { x, y });
            y += bound.height() / size as f64;
        }

        x += bound.width() / size as f64;
    }

    #[allow(unused_variables)]
    c.bench_function(
        "MultiPolygon contains Points via MultiPoint (n = 20)",
        |bencher| {
            // This internally uses IndexedMultiPolygon for optimization
            let multi_point = MultiPoint::new(coords.iter().map(|c| Point::from(*c)).collect());

            bencher.iter(|| {
                let result = zones.contains(&multi_point);
                criterion::black_box(result);
            });
        },
    );

    #[allow(unused_variables)]
    c.bench_function(
        "MultiPolygon contains individual Points (n = 20)",
        |bencher| {
            bencher.iter(|| {
                let mut inside = 0;

                for c in &coords {
                    if zones.coordinate_position(c) == CoordPos::Inside {
                        inside += 1
                    }
                }
            });
        },
    );
}

fn bench_line_contains_multi_point(c: &mut Criterion) {
    {
        let line = Line::new(coord! {x:0.,y:0.}, coord! {x:100., y:100.});
        let mut pts: Vec<Point> = (0..1000)
            .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
            .collect();
        pts.push(point! {x: f64::from(99), y: f64::from(100)});

        let multi_point: MultiPoint<f64> = geo::MultiPoint::new(pts).convert();

        trait_vs_relate!(
            c,
            "Line not contains 1000 MultiPoint",
            line,
            multi_point,
            Contains::contains,
            IntersectionMatrix::is_contains,
            false
        );
    }

    {
        let line = Line::new(coord! {x:0.,y:0.}, coord! {x:100., y:100.});
        let pts: Vec<Point> = (0..1000)
            .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
            .collect();
        let multi_point: MultiPoint<f64> = geo::MultiPoint::new(pts).convert();

        trait_vs_relate!(
            c,
            "Line contains 1000 MultiPoint",
            line,
            multi_point,
            Contains::contains,
            IntersectionMatrix::is_contains,
            true
        );
    }
}

fn bench_multipoint_contains_multipoint(c: &mut Criterion) {
    {
        let mut base: Vec<Point> = (0..10000)
            .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
            .collect();
        let comp = base.clone();

        base.reverse();
        base.push(point! {x: f64::from(1000), y: f64::from(1000)});

        let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
        let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

        trait_vs_relate!(
            c,
            "Line contains 1000 MultiPoint",
            base,
            comp,
            Contains::contains,
            IntersectionMatrix::is_contains,
            true
        );
    }

    {
        // best case where the point is at the end of the sorted lists
        let base: Vec<Point> = (0..10000)
            .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
            .collect();
        let mut comp = base.clone();

        comp.reverse();
        comp.push(point! {x: f64::from(1000), y: f64::from(1000)});

        let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
        let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

        trait_vs_relate!(
            c,
            "Line not contains 1000 MultiPoint worst case",
            base,
            comp,
            Contains::contains,
            IntersectionMatrix::is_contains,
            false
        );
    }

    {
        // best case where the point is at the start of the sorted lists
        let base: Vec<Point> = (0..10000)
            .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
            .collect();
        let mut comp = base.clone();

        comp.reverse();
        comp.push(point! {x: f64::from(-1000), y: f64::from(-1000)});

        let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
        let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

        trait_vs_relate!(
            c,
            "Multipoint not contains multipoint 10000 best case",
            base,
            comp,
            Contains::contains,
            IntersectionMatrix::is_contains,
            false
        );
    }
}

fn bench_polygon_contains_multipoint(c: &mut Criterion) {
    // worst case where the point is at the end of the sorted list

    {
        let base: Polygon<f64> = Polygon::new(
            LineString::new(vec![
                coord! {x: 0.0, y: 0.0},
                coord! {x: 10.0, y: 0.0},
                coord! {x: 10.0, y: 10.0},
                coord! {x: 0.0, y: 10.0},
                coord! {x: 0.0, y: 0.0},
            ]),
            vec![],
        );

        let mp: Vec<Point> = (0..1000)
            .map(|val| point! {x: f64::from(val)/100., y: f64::from(val)/100.})
            .collect();

        let mp: MultiPoint<f64> = geo::MultiPoint::new(mp).convert();

        trait_vs_relate!(
            c,
            "Polygon contains multipoint 1000",
            base,
            mp,
            Contains::contains,
            IntersectionMatrix::is_contains,
            true
        );
    }

    {
        let base: Polygon<f64> = Polygon::new(
            LineString::new(vec![
                coord! {x: 0.0, y: 0.0},
                coord! {x: 10.0, y: 0.0},
                coord! {x: 10.0, y: 10.0},
                coord! {x: 0.0, y: 10.0},
                coord! {x: 0.0, y: 0.0},
            ]),
            vec![],
        );

        let mut mp: Vec<Point> = (0..1000)
            .map(|val| point! {x: f64::from(val)/100., y: f64::from(val)/100.})
            .collect();
        mp.push(point! {x: f64::from(-1), y: f64::from(-1)});

        let mp: MultiPoint<f64> = geo::MultiPoint::new(mp).convert();

        trait_vs_relate!(
            c,
            "Polygon not contains multipoint 1000",
            base,
            mp,
            Contains::contains,
            IntersectionMatrix::is_contains,
            false
        );
    }
}

criterion_group!(
    benches,
    criterion_benchmark,
    bench_line_contains_multi_point,
    bench_multipoint_contains_multipoint,
    bench_polygon_contains_multipoint,
);
criterion_main!(benches);
