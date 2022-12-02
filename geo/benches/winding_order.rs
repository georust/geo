#[macro_use]
extern crate criterion;
extern crate geo;

use geo::prelude::*;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("winding order: winding_order (f32)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let _ = criterion::black_box(
                criterion::black_box(&ls).winding_order(),
            );
        });
    });

    c.bench_function("winding order: winding_order (f64)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f64>();

        bencher.iter(|| {
            let _ = criterion::black_box(
                criterion::black_box(&ls).winding_order(),
            );
        });
    });

    c.bench_function("winding order: points_cw (f32)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(
                criterion::black_box(&ls).points_cw(),
            );
            for point in points_iter.0 {
                criterion::black_box(point);
            }
        });
    });

    c.bench_function("winding order: points_cw (f64)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(
                criterion::black_box(&ls).points_cw(),
            );
            for point in points_iter.0 {
                criterion::black_box(point);
            }
        });
    });

    c.bench_function("winding order: points_ccw (f32)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(
                criterion::black_box(&ls).points_ccw(),
            );
            for point in points_iter.0 {
                criterion::black_box(point);
            }
        });
    });

    c.bench_function("winding order: points_ccw (f64)", |bencher| {
        let ls = geo_test_fixtures::louisiana::<f32>();

        bencher.iter(|| {
            let points_iter = criterion::black_box(
                criterion::black_box(&ls).points_ccw(),
            );
            for point in points_iter.0 {
                criterion::black_box(point);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
