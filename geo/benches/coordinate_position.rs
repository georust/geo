#[macro_use]
extern crate criterion;
extern crate geo;

use geo::{
    coordinate_position::CoordPos, BoundingRect, Centroid, CoordinatePosition, Point, Rect,
    Triangle,
};

use criterion::Criterion;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Point position to rect", |bencher| {
        let plot_centroids: Vec<Point> = geo_test_fixtures::nl_plots()
            .iter()
            .map(|plot| plot.centroid().unwrap())
            .collect();
        let zone_bbox: Vec<Rect> = geo_test_fixtures::nl_zones()
            .iter()
            .map(|plot| plot.bounding_rect().unwrap())
            .collect();
        bencher.iter(|| {
            let mut inside = 0;
            let mut outsied = 0;
            let mut boundary = 0;

            for a in &plot_centroids {
                for b in &zone_bbox {
                    match criterion::black_box(b).coordinate_position(criterion::black_box(&a.0)) {
                        CoordPos::OnBoundary => boundary += 1,
                        CoordPos::Inside => inside += 1,
                        CoordPos::Outside => outsied += 1,
                    }
                }
            }

            assert_eq!(inside, 2246);
            assert_eq!(outsied, 26510);
            assert_eq!(boundary, 0);
        });
    });

    c.bench_function("Point in triangle", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (5., 10.)]);
        let point = Point::new(5., 5.);

        bencher.iter(|| {
            assert!(
                criterion::black_box(&triangle).coordinate_position(criterion::black_box(&point.0))
                    != CoordPos::Outside
            );
        });
    });

    c.bench_function("Point on triangle boundary", |bencher| {
        let triangle = Triangle::from([(0., 0.), (10., 0.), (6., 10.)]);
        let point = Point::new(3., 5.);

        bencher.iter(|| {
            assert!(
                criterion::black_box(&triangle).coordinate_position(criterion::black_box(&point.0))
                    == CoordPos::OnBoundary
            );
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
