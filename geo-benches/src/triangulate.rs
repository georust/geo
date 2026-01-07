use criterion::{criterion_group, criterion_main};
use geo::algorithm::{TriangulateDelaunay, TriangulateDelaunayUnconstrained, TriangulateEarcut};
use geo::geometry::Polygon;
use geo::triangulate_delaunay::DelaunayTriangulationConfig;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function(
        "TriangulateSpade (unconstrained) - small polys",
        |bencher| {
            let multi_poly = geo_test_fixtures::nl_zones::<f64>();
            bencher.iter(|| {
                for poly in &multi_poly {
                    let triangulation =
                        TriangulateDelaunayUnconstrained::unconstrained_triangulation(poly)
                            .unwrap();
                    assert!(triangulation.len() > 1);
                    criterion::black_box(triangulation);
                }
            });
        },
    );

    c.bench_function("TriangulateSpade (constrained) - small polys", |bencher| {
        let multi_poly = geo_test_fixtures::nl_zones::<f64>();
        bencher.iter(|| {
            for poly in &multi_poly {
                let triangulation = TriangulateDelaunay::constrained_triangulation(
                    poly,
                    DelaunayTriangulationConfig { snap_radius: 1e-8 },
                )
                .unwrap();
                assert!(triangulation.len() > 1);
                criterion::black_box(triangulation);
            }
        });
    });

    c.bench_function("TriangulateEarcut - small polys", |bencher| {
        let multi_poly = geo_test_fixtures::nl_zones::<f64>();
        bencher.iter(|| {
            for poly in &multi_poly {
                let triangulation = TriangulateEarcut::earcut_triangles(poly);
                assert!(triangulation.len() > 1);
                criterion::black_box(triangulation);
            }
        });
    });

    c.bench_function("TriangulateSpade (unconstrained) - large_poly", |bencher| {
        let poly = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        bencher.iter(|| {
            let triangulation =
                TriangulateDelaunayUnconstrained::unconstrained_triangulation(&poly).unwrap();
            assert!(triangulation.len() > 1);
            criterion::black_box(triangulation);
        });
    });

    c.bench_function("TriangulateSpade (constrained) - large_poly", |bencher| {
        let poly = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        bencher.iter(|| {
            let triangulation = TriangulateDelaunay::constrained_triangulation(
                &poly,
                DelaunayTriangulationConfig { snap_radius: 1e-8 },
            )
            .unwrap();
            assert!(triangulation.len() > 1);
            criterion::black_box(triangulation);
        });
    });

    c.bench_function("TriangulateEarcut - large_poly", |bencher| {
        let poly = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        bencher.iter(|| {
            let triangulation = TriangulateEarcut::earcut_triangles(&poly);
            assert!(triangulation.len() > 1);
            criterion::black_box(triangulation);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
