use criterion::{criterion_group, criterion_main, Criterion};
use geo::intersects::Intersects;
use geo::{CoordsIter, MultiPolygon, Polygon};

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

fn rect_triangle_intersection(c: &mut Criterion) {
    use geo::{coord, Intersects, Rect, Triangle};

    c.bench_function("intersects triangle pt in rect", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let triangle = Triangle::from([(5., 5.), (11., 4.), (11., 6.)]);
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&triangle)));
        });
    });
    c.bench_function("intersects rect pt in triangle", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let triangle = Triangle::from([(-2., 4.), (4., -2.), (-2., -2.)]);
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&triangle)));
        });
    });
    c.bench_function("intersects edge intersections", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let triangle = Triangle::from([(5., -2.), (4., 11.), (6., 11.)]);
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&triangle)));
        });
    });
    c.bench_function("intersects edge intersections as polygon", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let triangle: Polygon = Triangle::from([(5., -2.), (4., 11.), (6., 11.)]).into();
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&triangle)));
        });
    });
    c.bench_function("triangle within rect", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let triangle = Triangle::from([(1., 1.), (1., 2.), (2., 1.)]);
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&triangle)));
        });
    });
    c.bench_function("rect within triangle", |bencher| {
        let rect = Rect::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. });
        let triangle = Triangle::from([(0., 10.), (10., 0.), (0., 0.)]);
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&triangle)));
        });
    });
    c.bench_function("disjoint", |bencher| {
        let rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. });
        let triangle = Triangle::from([(0., 11.), (1., 11.), (1., 12.)]);
        bencher.iter(|| {
            assert!(!criterion::black_box(&rect).intersects(criterion::black_box(&triangle)));
        });
    });
}

fn poly_rect_intersection(c: &mut Criterion) {
    use geo::{coord, polygon, Intersects, LineString, Polygon, Rect};

    fn get_rect() -> Rect<f64> {
        Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. })
    }

    c.bench_function("intersects polygon pt in rect", |bencher| {
        let rect = get_rect();
        let polygon = polygon![(x:5.,y:5.),(x:4.,y:11.),(x:5.,y:11.),(x:6.,y:11.),(x:5.,y:5.)];
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&polygon)));
        });
    });
    c.bench_function("intersects rect pt in polygon", |bencher| {
        let rect = get_rect();
        let polygon = polygon![(x:-2.,y:-2.),(x:-2.,y:-1.),(x:4.,y:-2.),(x:-2.,y:4.),(x:-1.,y:-2.),(x:-2.,y:-2.)];
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&polygon)));
        });
    });
    c.bench_function("intersects edge intersections", |bencher| {
        let rect = get_rect();
        let polygon =
            polygon![(x:5., y:-2.), (x:4., y:11.),(x:5., y:11.), (x:6., y:11.),(x:5., y:-2.)];
        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&polygon)));
        });
    });

    c.bench_function("not intersects disjoint", |bencher| {
        let rect = get_rect();
        let polygon: Polygon =
            Rect::new(coord! { x: 11., y: 11. }, coord! { x: 12., y: 12. }).into();

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect).intersects(criterion::black_box(&polygon)));
        });
    });

    c.bench_function("intersects rect equals polygon gap", |bencher| {
        let rect = get_rect();
        let ls = LineString::new(
            Rect::new(coord! {x:-1.,y:-1.}, coord! {x:11.,y:11.})
                .exterior_coords_iter()
                .collect(),
        );
        let polygon = Polygon::new(ls, vec![rect.exterior_coords_iter().collect()]);

        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&polygon)));
        });
    });

    c.bench_function("intersects rect within polygon gap", |bencher| {
        let rect = get_rect();
        let ls = LineString::new(
            Rect::new(coord! {x:-1.,y:-1.}, coord! {x:11.,y:11.})
                .exterior_coords_iter()
                .collect(),
        );
        let inner = LineString::new(
            Rect::new(coord! {x:-0.1,y:-0.1}, coord! {x:10.1,y:10.1})
                .exterior_coords_iter()
                .collect(),
        );
        let polygon = Polygon::new(ls, vec![inner]);

        bencher.iter(|| {
            assert!(!criterion::black_box(&rect).intersects(criterion::black_box(&polygon)));
        });
    });

    c.bench_function("intersects rect within polygon", |bencher| {
        let rect: Rect = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 10., y: 10. }).into();
        let polygon: Polygon =
            Rect::new(coord! { x: -1., y: -1. }, coord! { x: 11., y: 11. }).into();

        bencher.iter(|| {
            assert!(criterion::black_box(&rect).intersects(criterion::black_box(&polygon)));
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
criterion_group!(bench_rect_triangle, rect_triangle_intersection);
criterion_group!(bench_poly_rect, poly_rect_intersection);

criterion_main!(
    bench_multi_polygons,
    bench_rects,
    bench_point_rect,
    bench_point_triangle,
    bench_rect_triangle,
    bench_poly_rect,
);
