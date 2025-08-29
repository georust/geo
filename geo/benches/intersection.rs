use criterion::{Criterion, criterion_group, criterion_main};
use geo::intersects::Intersects;
use geo::{MultiPolygon, Triangle};

fn multi_polygon_intersection(c: &mut Criterion) {
    let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
    let zone_polygons: MultiPolygon = geo_test_fixtures::nl_zones();

    c.bench_function("MultiPolygon intersects", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_polygons {
                for b in &zone_polygons {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 974);
            assert_eq!(non_intersects, 27782);
        });
    });
}

fn rect_intersection(c: &mut Criterion) {
    use geo::algorithm::BoundingRect;
    use geo::geometry::Rect;
    let plot_bbox: Vec<Rect> = geo_test_fixtures::nl_plots_wgs84()
        .iter()
        .map(|plot| plot.bounding_rect().unwrap())
        .collect();
    let zone_bbox: Vec<Rect> = geo_test_fixtures::nl_zones()
        .iter()
        .map(|plot| plot.bounding_rect().unwrap())
        .collect();

    c.bench_function("Rect intersects", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_bbox {
                for b in &zone_bbox {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 3054);
            assert_eq!(non_intersects, 25702);
        });
    });
}

fn point_rect_intersection(c: &mut Criterion) {
    use geo::algorithm::{BoundingRect, Centroid};
    use geo::geometry::{Point, Rect};
    let plot_centroids: Vec<Point> = geo_test_fixtures::nl_plots_wgs84()
        .iter()
        .map(|plot| plot.centroid().unwrap())
        .collect();
    let zone_bbox: Vec<Rect> = geo_test_fixtures::nl_zones()
        .iter()
        .map(|plot| plot.bounding_rect().unwrap())
        .collect();

    c.bench_function("Point intersects rect", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_centroids {
                for b in &zone_bbox {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 2246);
            assert_eq!(non_intersects, 26510);
        });
    });
}

fn point_triangle_intersection(c: &mut Criterion) {
    use geo::{Centroid, TriangulateEarcut};
    use geo_types::{Point, Triangle};
    let plot_centroids: Vec<Point> = geo_test_fixtures::nl_plots_wgs84()
        .iter()
        .map(|plot| plot.centroid().unwrap())
        .collect();
    let zone_triangles: Vec<Triangle> = geo_test_fixtures::nl_zones()
        .iter()
        .flat_map(|plot| plot.earcut_triangles_iter())
        .collect();

    c.bench_function("Point intersects triangle", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_centroids {
                for b in &zone_triangles {
                    if criterion::black_box(a.intersects(b)) {
                        intersects += 1;
                    } else {
                        non_intersects += 1;
                    }
                }
            }

            assert_eq!(intersects, 533);
            assert_eq!(non_intersects, 5450151);
        });
    });

    c.bench_function("Triangle intersects point", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (5., 10.)]);
        let point = Point::new(5., 5.);

        bencher.iter(|| {
            assert!(criterion::black_box(&triangle).intersects(criterion::black_box(&point)));
        });
    });

    c.bench_function("Triangle intersects point on edge", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (6., 10.)]);
        let point = Point::new(3., 5.);

        bencher.iter(|| {
            assert!(criterion::black_box(&triangle).intersects(criterion::black_box(&point)));
        });
    });
}

fn linestring_polygon_intersection(c: &mut Criterion) {
    use geo::{LineString, Polygon, Rect, coord, line_string};
    c.bench_function("LineString above Polygon", |bencher| {
        let ls = line_string![
            coord! {x:0., y:1.},
            coord! {x:5., y:6.},
            coord! {x:10., y:1.}
        ];
        let poly = Polygon::new(
            line_string![
                coord! {x:0., y:0.},
                coord! {x:5., y:4.},
                coord! {x:10., y:0.}
            ],
            vec![],
        );

        bencher.iter(|| {
            assert!(!criterion::black_box(&ls).intersects(criterion::black_box(&poly)));
        });
    });
    c.bench_function("LineString above Triangle", |bencher| {
        let ls = line_string![
            coord! {x:0., y:1.},
            coord! {x:5., y:6.},
            coord! {x:10., y:1.}
        ];
        let poly = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:5., y:4.},
            coord! {x:10., y:0.},
        );

        bencher.iter(|| {
            assert!(!criterion::black_box(&ls).intersects(criterion::black_box(&poly)));
        });
    });
    c.bench_function("LineString around Rectangle", |bencher| {
        let ls = line_string![
            coord! {x:-1., y:-1.},
            coord! {x:-1., y:11.},
            coord! {x:11., y:11.}
        ];
        let poly = Rect::new(coord! {x:0., y:0.}, coord! {x:10., y:10.});

        bencher.iter(|| {
            assert!(!criterion::black_box(&ls).intersects(criterion::black_box(&poly)));
        });
    });

    c.bench_function("long disjoint ", |bencher| {
        let ls = LineString::from_iter((0..1000).map(|x| coord! {x:x as f64, y:x as f64}));
        let ln = (0..1000).map(|x| coord! {x:x as f64, y:(x-1) as f64});
        let k = vec![coord! {x:-5. ,y:-5. }].into_iter();
        let ext = ln.chain(k);

        let poly = Polygon::new(LineString::from_iter(ext), vec![]);

        bencher.iter(|| {
            assert!(!criterion::black_box(&ls).intersects(criterion::black_box(&poly)));
        });
    });

    c.bench_function("ls within poly ", |bencher| {
        let ls = line_string![
            coord! {x:1., y:1.},
            coord! {x:5., y:6.},
            coord! {x:9., y:1.}
        ];

        let poly: Polygon = Rect::new(coord! {x:0., y:0.}, coord! {x:10., y:10.}).into();

        bencher.iter(|| {
            assert!(criterion::black_box(&ls).intersects(criterion::black_box(&poly)));
        });
    });
    c.bench_function("ls within rect ", |bencher| {
        let ls = line_string![
            coord! {x:1., y:1.},
            coord! {x:5., y:6.},
            coord! {x:9., y:1.}
        ];

        let poly = Rect::new(coord! {x:0., y:0.}, coord! {x:10., y:10.});

        bencher.iter(|| {
            assert!(criterion::black_box(&ls).intersects(criterion::black_box(&poly)));
        });
    });
}

criterion_group! {
    name = bench_multi_polygons;
    config = Criterion::default().sample_size(10);
    targets = multi_polygon_intersection
}
criterion_group!(bench_rects, rect_intersection);
criterion_group! {
    name = bench_point_rect;
    config = Criterion::default().sample_size(50);
    targets = point_rect_intersection
}
criterion_group! {
    name = bench_point_triangle;
    config = Criterion::default().sample_size(50);
    targets = point_triangle_intersection
}

criterion_group! { bench_linestring_poly,linestring_polygon_intersection}

criterion_main!(
    bench_multi_polygons,
    bench_rects,
    bench_point_rect,
    bench_point_triangle,
    bench_linestring_poly,
);
