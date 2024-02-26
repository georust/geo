use criterion::{criterion_group, criterion_main};
use geo::coord;
use geo::{polygon, Polygon, TriangulateDelaunay, TriangulateSpade};

fn get_polygon() -> Polygon {
    polygon![
        coord! { x: 10., y: 10.},
        coord! { x: 10., y: 20.},
        coord! { x: 20., y: 20.},
        coord! { x: 20., y: 10.},
        coord! { x: 10., y: 0.},
        coord! { x: 10., y: 0.},
        coord! { x: 0., y: 10.},
        coord! { x: 0., y: 20.},
    ]
}

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("Delaunay Triangulation", |bencher| {
        let poly = get_polygon();
        bencher.iter(|| poly.delaunay_triangulation().unwrap())
    });

    c.bench_function("Spade Unconstrained Triangulation", |bencher| {
        let poly = get_polygon();
        bencher.iter(|| {
            poly.unconstrained_triangulation().unwrap();
        })
    });

    c.bench_function("Constrained Outer Triangulation", |bencher| {
        let poly = get_polygon();
        bencher.iter(|| {
            poly.constrained_outer_triangulation(Default::default())
                .unwrap();
        })
    });

    c.bench_function("Constrain Triangulation", |bencher| {
        let poly = get_polygon();
        bencher.iter(|| {
            poly.constrained_triangulation(Default::default()).unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
