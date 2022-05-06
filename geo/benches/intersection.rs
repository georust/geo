#[macro_use]
extern crate criterion;
extern crate geo;
extern crate geo_test_fixtures;

use criterion::Criterion;
use geo::intersects::Intersects;
use geo::MultiPolygon;

fn criterion_benchmark(c: &mut Criterion) {
    let plot_polygons: MultiPolygon<f64> = geo_test_fixtures::nl_plots();
    let zone_polygons: MultiPolygon<f64> = geo_test_fixtures::nl_zones();

    c.bench_function("intersection", |bencher| {
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

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
}

criterion_main!(benches);
