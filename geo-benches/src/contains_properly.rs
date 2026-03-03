use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use geo::PreparedGeometry;
use geo::algorithm::{ContainsProperly, Convert, Relate};
use geo::wkt;
use geo::{coord, geometry::*};
use std::iter::once;

fn compare_simple_in_complex(c: &mut Criterion) {
    c.bench_function(
        "complex polygon contains_properly polygon (Trait)",
        |bencher| {
            let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
            let contained_polygon = geo_test_fixtures::east_baton_rouge();

            bencher.iter(|| {
                assert!(
                    criterion::black_box(&polygon)
                        .contains_properly(criterion::black_box(&contained_polygon))
                );
            });
        },
    );

    c.bench_function(
        "complex polygon contains_properly polygon (Relate)",
        |bencher| {
            let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
            let contained_polygon = geo_test_fixtures::east_baton_rouge();

            bencher.iter(|| {
                assert!(
                    criterion::black_box(&polygon)
                        .relate(criterion::black_box(&contained_polygon))
                        .is_contains_properly()
                );
            });
        },
    );

    c.bench_function(
        "complex polygon contains_properly polygon (prepared Relate)",
        |bencher| {
            let polygon = Polygon::<f64>::new(geo_test_fixtures::louisiana(), vec![]);
            let polygon = PreparedGeometry::from(&polygon);
            let contained_polygon = geo_test_fixtures::east_baton_rouge();
            let contained_polygon = PreparedGeometry::from(&contained_polygon);

            bencher.iter(|| {
                assert!(
                    criterion::black_box(&polygon)
                        .relate(criterion::black_box(&contained_polygon))
                        .is_contains()
                );
            });
        },
    );
}

fn compare_poly_in_poly(c: &mut Criterion) {
    use geo::algorithm::Contains;

    let poly1: Polygon<f64> = wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0),(6 3,6 6,3 6,3 3,6 3))}.convert();
    let poly2: Polygon<f64> = wkt! {POLYGON((8 1,8 8,1 8,1 1,8 1),(7 2,7 7,2 7,2 2,7 2))}.convert();

    let multipoly1: MultiPolygon<f64> =
        wkt! {MULTIPOLYGON(((9 0,9 9,0 9,0 0,9 0),(8 1,8 8,1 8,1 1,8 1)),((7 2,7 7,2 7,2 2,7 2)))}
            .convert();
    let multipoly2: MultiPolygon<f64> = wkt! {MULTIPOLYGON(((6 3,6 6,3 6,3 3,6 3)))}.convert();

    c.bench_function("contains_properly poly poly (Trait)", |bencher| {
        bencher.iter(|| {
            assert!(criterion::black_box(&poly1).contains_properly(criterion::black_box(&poly2)));
        });
    });

    c.bench_function("relate prepared poly poly", |bencher| {
        let p1 = PreparedGeometry::from(poly1.clone());
        let p2 = PreparedGeometry::from(poly2.clone());
        bencher.iter(|| {
            assert!(
                criterion::black_box(&p1)
                    .relate(criterion::black_box(&p2))
                    .is_contains_properly()
            );
        });
    });

    c.bench_function("contains poly poly (Trait)", |bencher| {
        bencher.iter(|| {
            assert!(criterion::black_box(&poly1).contains(criterion::black_box(&poly2)));
        });
    });

    c.bench_function("relate poly poly", |bencher| {
        bencher.iter(|| {
            assert!(
                criterion::black_box(&poly1)
                    .relate(criterion::black_box(&poly2))
                    .is_contains_properly()
            );
        });
    });

    c.bench_function("contains_properly multipoly multipoly (Trait)", |bencher| {
        bencher.iter(|| {
            assert!(
                criterion::black_box(&multipoly1)
                    .contains_properly(criterion::black_box(&multipoly2))
            );
        });
    });

    c.bench_function("relate prepared multipoly multipoly", |bencher| {
        let p1 = PreparedGeometry::from(multipoly1.clone());
        let p2 = PreparedGeometry::from(multipoly2.clone());
        bencher.iter(|| {
            assert!(
                criterion::black_box(&p1)
                    .relate(criterion::black_box(&p2))
                    .is_contains_properly()
            );
        });
    });

    c.bench_function("contains multipoly multipoly (Trait)", |bencher| {
        bencher.iter(|| {
            assert!(criterion::black_box(&multipoly1).contains(criterion::black_box(&multipoly2)));
        });
    });

    c.bench_function("relate multipoly multipoly", |bencher| {
        bencher.iter(|| {
            assert!(
                criterion::black_box(&multipoly1)
                    .relate(criterion::black_box(&multipoly2))
                    .is_contains_properly()
            );
        });
    });
}

fn polygon_polygon_scaling(c: &mut Criterion) {
    fn make_outer_poly(n: i32) -> Polygon<f64> {
        let pts: LineString<f64> = LineString::new(
            once(coord! {x:0,y:0})
                .chain((1..n).map(|i: i32| coord! {x:i*5,y:5*(1+(1+i%2))}))
                .chain(once(coord! {x:n,y:0}))
                .collect(),
        )
        .convert();
        Polygon::<f64>::new(pts, vec![])
    }
    fn make_inner_poly(n: i32) -> Polygon<f64> {
        let pts: LineString<f64> = LineString::new(
            once(coord! {x:1,y:1})
                .chain((1..n).map(|i: i32| coord! {x:i,y:(1+i%2)}))
                .chain(once(coord! {x:n-1,y:0}))
                .collect(),
        )
        .convert();
        Polygon::<f64>::new(pts, vec![])
    }

    {
        // create two polygons, both of  of n+2 sides and no holes
        let mut group = c.benchmark_group("contains_properly polygon polygon scaling");

        // trait is faster for small polygons, but relate overtakes from around 700*700 boundary segment checks
        for i in [10, 100, 200, 300, 400, 500, 600, 700, 800] {
            group.throughput(Throughput::Elements(i as u64));

            let inner_poly = make_inner_poly(i);
            let outer_poly = make_outer_poly(i);

            group.bench_with_input(
                BenchmarkId::new("trait", i),
                &(&outer_poly, &inner_poly),
                |bencher, &(a, b)| {
                    bencher.iter(|| a.contains_properly(b));
                },
            );

            group.bench_with_input(
                BenchmarkId::new("relate", i),
                &(&outer_poly, &inner_poly),
                |bencher, &(a, b)| {
                    bencher.iter(|| a.relate(b).is_contains_properly());
                },
            );
        }
        group.finish();
    }
}

criterion_group!(benches, compare_simple_in_complex, compare_poly_in_poly,);
criterion_group!(benches_scaling, polygon_polygon_scaling);

criterion_main!(benches, benches_scaling);
