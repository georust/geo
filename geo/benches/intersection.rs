use criterion::{criterion_group, criterion_main, Criterion};
use geo::intersects::Intersects;
use geo::{Centroid, Geometry, MultiPolygon};
use geo_traits::to_geo::ToGeoGeometry;

#[path = "utils/wkb.rs"]
mod wkb;

fn multi_polygon_intersection(c: &mut Criterion) {
    let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
    let zone_polygons: MultiPolygon = geo_test_fixtures::nl_zones();
    let plot_geoms: Vec<Geometry> = plot_polygons.into_iter().map(|p| p.into()).collect();
    let zone_geoms: Vec<Geometry> = zone_polygons.into_iter().map(|p| p.into()).collect();

    c.bench_function("MultiPolygon intersects", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_geoms {
                for b in &zone_geoms {
                    if criterion::black_box(b.intersects(a)) {
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

fn multi_polygon_intersection_wkb_conv(c: &mut Criterion) {
    let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
    let zone_polygons: MultiPolygon = geo_test_fixtures::nl_zones();

    // Convert intersected polygons to WKB
    let mut plot_polygon_wkbs = Vec::new();
    let mut zone_polygon_wkbs = Vec::new();
    for plot_polygon in &plot_polygons {
        plot_polygon_wkbs.push(wkb::geo_to_wkb(plot_polygon));
    }
    for zone_polygon in &zone_polygons {
        zone_polygon_wkbs.push(wkb::geo_to_wkb(zone_polygon));
    }

    c.bench_function("MultiPolygon intersects wkb conv", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_polygon_wkbs {
                for b in &zone_polygon_wkbs {
                    let a_geom = geo_generic_tests::wkb::reader::read_wkb(a).unwrap();
                    let b_geom = geo_generic_tests::wkb::reader::read_wkb(b).unwrap();
                    let a_geom = a_geom.to_geometry();
                    let b_geom = b_geom.to_geometry();
                    if criterion::black_box(b_geom.intersects(&a_geom)) {
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

fn point_polygon_intersection(c: &mut Criterion) {
    let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
    let zone_polygons: MultiPolygon = geo_test_fixtures::nl_zones();
    let plot_geoms: Vec<Geometry> = plot_polygons
        .into_iter()
        .map(|p| {
            let centroid = p.centroid().unwrap();
            centroid.into()
        })
        .collect();
    let zone_geoms: Vec<Geometry> = zone_polygons.into_iter().map(|p| p.into()).collect();

    c.bench_function("Point polygon intersects", |bencher| {
        bencher.iter(|| {
            for a in &plot_geoms {
                for b in &zone_geoms {
                    criterion::black_box(b.intersects(a));
                }
            }
        });
    });
}

fn point_polygon_intersection_wkb_conv(c: &mut Criterion) {
    let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
    let zone_polygons: MultiPolygon = geo_test_fixtures::nl_zones();

    // Convert intersected polygons to WKB
    let mut plot_centroid_wkbs = Vec::new();
    let mut zone_polygon_wkbs = Vec::new();
    for plot_polygon in &plot_polygons {
        let centroid = plot_polygon.centroid().unwrap();
        plot_centroid_wkbs.push(wkb::geo_to_wkb(centroid));
    }
    for zone_polygon in &zone_polygons {
        zone_polygon_wkbs.push(wkb::geo_to_wkb(zone_polygon));
    }

    c.bench_function("Point polygon intersects wkb conv", |bencher| {
        bencher.iter(|| {
            for a in &plot_centroid_wkbs {
                for b in &zone_polygon_wkbs {
                    let a_geom = geo_generic_tests::wkb::reader::read_wkb(a).unwrap();
                    let b_geom = geo_generic_tests::wkb::reader::read_wkb(b).unwrap();
                    let a_geom = a_geom.to_geometry();
                    let b_geom = b_geom.to_geometry();
                    criterion::black_box(b_geom.intersects(&a_geom));
                }
            }
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

criterion_group! {
    name = bench_multi_polygons;
    config = Criterion::default().sample_size(10);
    targets = multi_polygon_intersection
}
criterion_group! {
    name = bench_multi_polygons_wkb_conv;
    config = Criterion::default().sample_size(10);
    targets = multi_polygon_intersection_wkb_conv
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

criterion_group! {
    name = bench_point_polygon;
    config = Criterion::default().sample_size(50);
    targets = point_polygon_intersection
}
criterion_group! {
    name = bench_point_polygon_wkb_conv;
    config = Criterion::default().sample_size(50);
    targets = point_polygon_intersection_wkb_conv
}

criterion_main!(
    bench_multi_polygons,
    bench_multi_polygons_wkb_conv,
    bench_rects,
    bench_point_rect,
    bench_point_triangle,
    bench_point_polygon,
    bench_point_polygon_wkb_conv
);
