use criterion::{criterion_group, criterion_main, Criterion};
use geo::algorithm::{Contains, Convert, Relate};
use geo::{coord, line_string, point, polygon};
use geo::{geometry::*, CoordsIter};

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

fn line_contains_multi_point(c: &mut Criterion) {
    c.bench_function(
        "Line not contains 1000 MultiPoint within bounding box (Contains trait)",
        |bencher| {
            let line = Line::new(coord! {x:0.,y:0.}, coord! {x:100., y:100.});
            let mut pts: Vec<Point> = (0..1000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            pts.push(point! {x: f64::from(99), y: f64::from(100)});

            let multi_point: MultiPoint<f64> = geo::MultiPoint::new(pts).convert();

            bencher.iter(|| {
                assert!(!line.contains(&multi_point));
            });
        },
    );

    c.bench_function(
        "Line not contains 1000 MultiPoint within bounding box (Relate trait)",
        |bencher| {
            let line = Line::new(coord! {x:0.,y:0.}, coord! {x:100., y:100.});
            let mut pts: Vec<Point> = (0..1000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            pts.push(point! {x: f64::from(99), y: f64::from(100)});

            let multi_point: MultiPoint<f64> = geo::MultiPoint::new(pts).convert();

            bencher.iter(|| {
                assert!(!line.relate(&multi_point).is_contains());
            });
        },
    );

    c.bench_function(
        "Line contains 1000 MultiPoint (Contains trait)",
        |bencher| {
            let line = Line::new(coord! {x:0.,y:0.}, coord! {x:100., y:100.});
            let pts: Vec<Point> = (0..1000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            let multi_point: MultiPoint<f64> = geo::MultiPoint::new(pts).convert();

            bencher.iter(|| {
                assert!(line.contains(&multi_point));
            });
        },
    );

    c.bench_function("Line contains 1000 MultiPoint (Relate trait)", |bencher| {
        let line = Line::new(coord! {x:0.,y:0.}, coord! {x:100., y:100.});
        let pts: Vec<Point> = (0..1000)
            .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
            .collect();
        let multi_point: MultiPoint<f64> = geo::MultiPoint::new(pts).convert();

        bencher.iter(|| {
            assert!(line.relate(&multi_point).is_contains());
        });
    });
}

fn rect_contains_line(c: &mut Criterion) {
    c.bench_function("line within rect (Contains Trait)", |bencher| {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ln = Line::new(Point::new(1., 1.), Point::new(9., 4.));

        bencher.iter(|| {
            assert!(criterion::black_box(&rect).contains(criterion::black_box(&ln)));
        });
    });
    c.bench_function("line within rect (Relates Trait)", |bencher| {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ln = Line::new(Point::new(1., 1.), Point::new(9., 4.));

        bencher.iter(|| {
            assert!(criterion::black_box(&rect)
                .relate(criterion::black_box(&ln))
                .is_contains());
        });
    });

    c.bench_function("line disjoint rect (Contains Trait)", |bencher| {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ln = Line::new(Point::new(1., 6.), Point::new(9., 6.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect).contains(criterion::black_box(&ln)));
        });
    });
    c.bench_function("line disjoint rect (Relates Trait)", |bencher| {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ln = Line::new(Point::new(1., 6.), Point::new(9., 6.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect)
                .relate(criterion::black_box(&ln))
                .is_contains());
        });
    });

    c.bench_function("line along rect (Contains Trait)", |bencher| {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ln = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect).contains(criterion::black_box(&ln)));
        });
    });
    c.bench_function("line along rect (Relates Trait)", |bencher| {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ln = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect)
                .relate(criterion::black_box(&ln))
                .is_contains());
        });
    });
}

fn multipoint_contains_multipoint(c: &mut Criterion) {
    // worst case where the point is at the end of the sorted list

    c.bench_function(
        "Multipoint contains multipoint 10000 (Contains trait)",
        |bencher| {
            let mut base: Vec<Point> = (0..10000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            let comp = base.clone();

            base.reverse();
            base.push(point! {x: f64::from(1000), y: f64::from(1000)});

            let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
            let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

            bencher.iter(|| {
                assert!(base.contains(&comp));
            });
        },
    );

    c.bench_function(
        "Multipoint contains multipoint 10000 (Relate trait)",
        |bencher| {
            let mut base: Vec<Point> = (0..10000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            let comp = base.clone();

            base.reverse();
            base.push(point! {x: f64::from(1000), y: f64::from(1000)});

            let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
            let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

            bencher.iter(|| {
                assert!(base.relate(&comp).is_contains());
            });
        },
    );

    c.bench_function(
        "Multipoint not contains multipoint 10000 (Contains trait)",
        |bencher| {
            let base: Vec<Point> = (0..10000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            let mut comp = base.clone();

            comp.reverse();
            comp.push(point! {x: f64::from(1000), y: f64::from(1000)});

            let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
            let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

            bencher.iter(|| {
                assert!(!base.contains(&comp));
            });
        },
    );

    c.bench_function(
        "Multipoint not contains multipoint 10000 (Relate trait)",
        |bencher| {
            let base: Vec<Point> = (0..10000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            let mut comp = base.clone();

            comp.reverse();
            comp.push(point! {x: f64::from(1000), y: f64::from(1000)});

            let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
            let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

            bencher.iter(|| {
                assert!(!base.relate(&comp).is_contains());
            });
        },
    );

    c.bench_function(
        "Multipoint not contains multipoint 10000 best case (Contains trait)",
        |bencher| {
            let base: Vec<Point> = (0..10000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            let mut comp = base.clone();

            comp.reverse();
            comp.push(point! {x: f64::from(-1000), y: f64::from(-1000)});

            let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
            let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

            bencher.iter(|| {
                assert!(!base.contains(&comp));
            });
        },
    );

    c.bench_function(
        "Multipoint not contains multipoint 10000 best case (Relate trait)",
        |bencher| {
            let base: Vec<Point> = (0..10000)
                .map(|val| point! {x: f64::from(val)/10., y: f64::from(val)/10.})
                .collect();
            let mut comp = base.clone();

            comp.reverse();
            comp.push(point! {x: f64::from(-1000), y: f64::from(-1000)});

            let base: MultiPoint<f64> = geo::MultiPoint::new(base).convert();
            let comp: MultiPoint<f64> = geo::MultiPoint::new(comp).convert();

            bencher.iter(|| {
                assert!(!base.relate(&comp).is_contains());
            });
        },
    );
}

fn polygon_contains_multipoint(c: &mut Criterion) {
    // worst case where the point is at the end of the sorted list

    c.bench_function(
        "Polygon contains multipoint 1000 (Contains trait)",
        |bencher| {
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

            bencher.iter(|| {
                assert!(criterion::black_box(&base).contains(criterion::black_box(&mp)));
            });
        },
    );

    c.bench_function(
        "Polygon contains multipoint 1000 (Relates trait)",
        |bencher| {
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

            bencher.iter(|| {
                assert!(criterion::black_box(&base)
                    .relate(criterion::black_box(&mp))
                    .is_contains());
            });
        },
    );

    c.bench_function(
        "Polygon not contains multipoint 1000 (Contains trait)",
        |bencher| {
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

            bencher.iter(|| {
                assert!(!criterion::black_box(&base).contains(criterion::black_box(&mp)));
            });
        },
    );

    c.bench_function(
        "Polygon not contains multipoint 1000 (Relates trait)",
        |bencher| {
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

            bencher.iter(|| {
                assert!(!criterion::black_box(&base)
                    .relate(criterion::black_box(&mp))
                    .is_contains());
            });
        },
    );
}

fn triangle_contains_line(c: &mut Criterion) {
    c.bench_function("line within triangle (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );
        let ln = Line::new(Point::new(3., 1.), Point::new(9., 3.));

        bencher.iter(|| {
            assert!(criterion::black_box(&tri).contains(criterion::black_box(&ln)));
        });
    });
    c.bench_function("line within triangle (Relates Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );
        let ln = Line::new(Point::new(3., 1.), Point::new(9., 3.));

        bencher.iter(|| {
            assert!(criterion::black_box(&tri)
                .relate(criterion::black_box(&ln))
                .is_contains());
        });
    });

    c.bench_function("line disjoint triangle (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );
        let ln = Line::new(Point::new(1., 6.), Point::new(9., 6.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&tri).contains(criterion::black_box(&ln)));
        });
    });
    c.bench_function("line disjoint triangle (Relates Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );
        let ln = Line::new(Point::new(1., 6.), Point::new(9., 6.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&tri)
                .relate(criterion::black_box(&ln))
                .is_contains());
        });
    });

    c.bench_function("line along triangle (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );
        let ln = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&tri).contains(criterion::black_box(&ln)));
        });
    });
    c.bench_function("line along triangle (Relates Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );
        let ln = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&tri)
                .relate(criterion::black_box(&ln))
                .is_contains());
        });
    });
}

fn triangle_contains_rect(c: &mut Criterion) {
    c.bench_function("rect within triangle (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));

        bencher.iter(|| {
            assert!(criterion::black_box(&tri).contains(criterion::black_box(&rect)));
        });
    });
    c.bench_function("rect within triangle (Relate Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));

        bencher.iter(|| {
            assert!(criterion::black_box(&tri)
                .relate(criterion::black_box(&rect))
                .is_contains());
        });
    });

    c.bench_function("triangle within rect (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 10.));

        bencher.iter(|| {
            assert!(criterion::black_box(&rect).contains(criterion::black_box(&tri)));
        });
    });
    c.bench_function("triangle within rect (Relate Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 10.));

        bencher.iter(|| {
            assert!(criterion::black_box(&rect)
                .relate(criterion::black_box(&tri))
                .is_contains());
        });
    });

    c.bench_function("rect disjoint triangle (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(-1., -1.), Point::new(-10., -10.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect).contains(criterion::black_box(&tri)));
        });
    });
    c.bench_function("rect disjoint triangle (Relate Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(-1., -1.), Point::new(-10., -10.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect)
                .relate(criterion::black_box(&tri))
                .is_contains());
        });
    });

    c.bench_function("triangle disjoint rect (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(-1., -1.), Point::new(-10., -10.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&tri).contains(criterion::black_box(&rect)));
        });
    });
    c.bench_function("triangle disjoint rect (Relate Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let rect = Rect::new(Point::new(-1., -1.), Point::new(-10., -10.));

        bencher.iter(|| {
            assert!(!criterion::black_box(&tri)
                .relate(criterion::black_box(&rect))
                .is_contains());
        });
    });
}

fn rect_contains_linestring(c: &mut Criterion) {
    c.bench_function(
        "rect contains linestring 40 on boundary (Contains Trait)",
        |bencher| {
            let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));
            let ls = LineString::from_iter(rect.coords_iter().cycle().take(10));

            bencher.iter(|| {
                assert!(!criterion::black_box(&rect).contains(criterion::black_box(&ls)));
            });
        },
    );
    c.bench_function(
        "rect contains linestring 40 on boundary (Relate Trait)",
        |bencher| {
            let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));
            let ls = LineString::from_iter(rect.coords_iter().cycle().take(10));

            bencher.iter(|| {
                assert!(!criterion::black_box(&rect)
                    .relate(criterion::black_box(&ls))
                    .is_contains());
            });
        },
    );

    c.bench_function(
        "rect contains linestring 4000 on boundary (Contains Trait)",
        |bencher| {
            let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));
            let ls = LineString::from_iter(rect.coords_iter().cycle().take(1000));

            bencher.iter(|| {
                assert!(!criterion::black_box(&rect).contains(criterion::black_box(&ls)));
            });
        },
    );
    c.bench_function(
        "rect contains linestring 4000 on boundary (Relate Trait)",
        |bencher| {
            let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));
            let ls = LineString::from_iter(rect.coords_iter().cycle().take(1000));

            bencher.iter(|| {
                assert!(!criterion::black_box(&rect)
                    .relate(criterion::black_box(&ls))
                    .is_contains());
            });
        },
    );

    c.bench_function(
        "rect contains linestring disjoint in bb (Contains Trait)",
        |bencher| {
            let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));
            let ls = line_string![
                coord! {x:0.,y:0.},
                coord! {x:0.,y:10.},
                coord! {x:10.,y:10.},
                coord! {x:10.,y:0.}
            ];

            bencher.iter(|| {
                assert!(!criterion::black_box(&rect).contains(criterion::black_box(&ls)));
            });
        },
    );
    c.bench_function(
        "rect contains linestring disjoint in bb (Relate Trait)",
        |bencher| {
            let rect = Rect::new(Point::new(5., 5.), Point::new(9., 1.));
            let ls = line_string![
                coord! {x:0.,y:0.},
                coord! {x:0.,y:10.},
                coord! {x:10.,y:10.},
                coord! {x:10.,y:0.}
            ];

            bencher.iter(|| {
                assert!(!criterion::black_box(&rect)
                    .relate(criterion::black_box(&ls))
                    .is_contains());
            });
        },
    );
}

fn triangle_contains_linestring(c: &mut Criterion) {
    c.bench_function(
        "triangle contains linestring 30 on boundary (Contains Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let ls = LineString::from_iter(tri.coords_iter().cycle().take(10));

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri).contains(criterion::black_box(&ls)));
            });
        },
    );
    c.bench_function(
        "triangle contains linestring 30 on boundary (Relate Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let ls = LineString::from_iter(tri.coords_iter().cycle().take(10));

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri)
                    .relate(criterion::black_box(&ls))
                    .is_contains());
            });
        },
    );

    c.bench_function(
        "triangle contains linestring 3000 on boundary (Contains Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let ls = LineString::from_iter(tri.coords_iter().cycle().take(1000));

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri).contains(criterion::black_box(&ls)));
            });
        },
    );
    c.bench_function(
        "triangle contains linestring 3000 on boundary (Relate Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let ls = LineString::from_iter(tri.coords_iter().cycle().take(1000));

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri)
                    .relate(criterion::black_box(&ls))
                    .is_contains());
            });
        },
    );

    c.bench_function(
        "triangle contains linestring disjoint in bounding box (Contains Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let ls = line_string![
                coord! {x:0.,y:0.},
                coord! {x:0.,y:10.},
                coord! {x:10.,y:10.},
                coord! {x:10.,y:0.}
            ];

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri).contains(criterion::black_box(&ls)));
            });
        },
    );
    c.bench_function(
        "triangle contains linestring disjoint in bounding box (Relate Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let ls = line_string![
                coord! {x:0.,y:0.},
                coord! {x:0.,y:10.},
                coord! {x:10.,y:10.},
                coord! {x:10.,y:0.}
            ];

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri)
                    .relate(criterion::black_box(&ls))
                    .is_contains());
            });
        },
    );
}

fn triangle_contains_triangle(c: &mut Criterion) {
    /*
       contains
       disjoint
    */

    c.bench_function("triangle contains triangle (Contains Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0.,y:0.},
            coord! {x:10.,y:0.},
            coord! {x:10.,y:10.},
        );

        bencher.iter(|| {
            assert!(criterion::black_box(&tri).contains(criterion::black_box(&tri)));
        });
    });
    c.bench_function("triangle contains triangle (Relate Trait)", |bencher| {
        let tri = Triangle::new(
            coord! {x:0.,y:0.},
            coord! {x:10.,y:0.},
            coord! {x:10.,y:10.},
        );

        bencher.iter(|| {
            assert!(criterion::black_box(&tri)
                .relate(criterion::black_box(&tri))
                .is_contains());
        });
    });

    c.bench_function(
        "triangle disjoint triangle disjoint bounding box(Contains Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let tri2 = Triangle::new(
                coord! {x:-1.,y:-1.},
                coord! {x:-10.,y:-1.},
                coord! {x:-10.,y:-10.},
            );

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri).contains(criterion::black_box(&tri2)));
            });
        },
    );
    c.bench_function(
        "triangle disjoint triangle  disjoint bounding box (Relate Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let tri2 = Triangle::new(
                coord! {x:-1.,y:-1.},
                coord! {x:-10.,y:-1.},
                coord! {x:-10.,y:-10.},
            );
            bencher.iter(|| {
                assert!(!criterion::black_box(&tri)
                    .relate(criterion::black_box(&tri2))
                    .is_contains());
            });
        },
    );

    c.bench_function(
        "triangle disjoint triangle overlapping bounding box (Contains Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let tri2 = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:0.,y:10.},
                coord! {x:10.,y:10.},
            );

            bencher.iter(|| {
                assert!(!criterion::black_box(&tri).contains(criterion::black_box(&tri2)));
            });
        },
    );
    c.bench_function(
        "triangle disjoint triangle overlapping bounding box (Relate Trait)",
        |bencher| {
            let tri = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:10.,y:0.},
                coord! {x:10.,y:10.},
            );
            let tri2 = Triangle::new(
                coord! {x:0.,y:0.},
                coord! {x:0.,y:10.},
                coord! {x:10.,y:10.},
            );
            bencher.iter(|| {
                assert!(!criterion::black_box(&tri)
                    .relate(criterion::black_box(&tri2))
                    .is_contains());
            });
        },
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_group!(bench_line_contains_multi_point, line_contains_multi_point);
criterion_group!(bench_rect_contains_line, rect_contains_line);
criterion_group!(
    bench_multipoint_contains_multipoint,
    multipoint_contains_multipoint
);
criterion_group!(
    bench_polygon_contains_multipoint,
    polygon_contains_multipoint
);
criterion_group!(bench_triangle_contains_line, triangle_contains_line);
criterion_group!(bench_triangle_contains_rect, triangle_contains_rect);
criterion_group!(bench_rect_contains_linestring, rect_contains_linestring);
criterion_group!(
    bench_triangle_contains_linestring,
    triangle_contains_linestring
);
criterion_group!(bench_triangle_contains_triangle, triangle_contains_triangle);

criterion_main!(
    benches,
    bench_line_contains_multi_point,
    bench_rect_contains_line,
    bench_multipoint_contains_multipoint,
    bench_polygon_contains_multipoint,
    bench_triangle_contains_line,
    bench_triangle_contains_rect,
    bench_rect_contains_linestring,
    bench_triangle_contains_linestring,
    bench_triangle_contains_triangle,
);
