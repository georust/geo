use criterion::{criterion_group, criterion_main, Criterion};
use geo::contains::Contains;
use geo::{point, polygon, Line, Point, Polygon, Triangle};

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
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
