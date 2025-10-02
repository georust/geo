use criterion::{Criterion, criterion_group, criterion_main};
use geo::PreparedGeometry;
use geo::algorithm::{ContainsProperly, Convert, Relate};
use geo::geometry::*;
use geo::wkt;

fn compare_poly_in_poly(c: &mut Criterion) {
    use geo::algorithm::Contains;
    let poly1: Polygon<f64> = wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0),(6 3,6 6,3 6,3 3,6 3))}.convert();
    let poly2: Polygon<f64> = wkt! {POLYGON((8 1,8 8,1 8,1 1,8 1),(7 2,7 7,2 7,2 2,7 2))}.convert();

    let multipoly1: MultiPolygon<f64> =
        wkt! {MULTIPOLYGON(((9 0,9 9,0 9,0 0,9 0),(8 1,8 8,1 8,1 1,8 1)),((7 2,7 7,2 7,2 2,7 2)))}
            .convert();
    let multipoly2: MultiPolygon<f64> = wkt! {MULTIPOLYGON(((6 3,6 6,3 6,3 3,6 3)))}.convert();

    c.bench_function("contains_properly poly poly", |bencher| {
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

    c.bench_function("contains poly poly", |bencher| {
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

    c.bench_function("contains_properly multipoly multipoly", |bencher| {
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

    c.bench_function("contains multipoly multipoly", |bencher| {
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

criterion_group!(benches, compare_poly_in_poly,);
criterion_main!(benches);
