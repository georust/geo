#[macro_use]
extern crate criterion;
extern crate geo;
use geo::frechet_distance::FrechetDistance;
use geo::{Coordinate, LineString};

fn criterion_benchmark(c: &mut criterion::Criterion) {
    c.bench_function("frechet distance f32", |bencher| {
        let points_a = include!("../src/algorithm/test_fixtures/vw_orig.rs");
        let ls_a = LineString::<f32>(
            points_a
                .iter()
                .map(|e| Coordinate { x: e[0], y: e[1] })
                .collect(),
        );
        let points_b = include!("../src/algorithm/test_fixtures/vw_simplified.rs");
        let ls_b = LineString::<f32>(
            points_b
                .iter()
                .map(|e| Coordinate { x: e[0], y: e[1] })
                .collect(),
        );

        bencher.iter(|| {
            let _ = ls_a.frechet_distance(&ls_b);
        });
    });

    c.bench_function("frechet distance f64", |bencher| {
        let points_a = include!("../src/algorithm/test_fixtures/vw_orig.rs");
        let ls_a = LineString::<f64>(
            points_a
                .iter()
                .map(|e| Coordinate { x: e[0], y: e[1] })
                .collect(),
        );
        let points_b = include!("../src/algorithm/test_fixtures/vw_simplified.rs");
        let ls_b = LineString::<f64>(
            points_b
                .iter()
                .map(|e| Coordinate { x: e[0], y: e[1] })
                .collect(),
        );

        bencher.iter(|| {
            let _ = ls_a.frechet_distance(&ls_b);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
