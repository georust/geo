#[macro_use]
extern crate criterion;
extern crate geo;

use crate::geo::relate::Relate;
use crate::geo::rotate::Rotate;
use crate::geo::translate::Translate;
use criterion::{BatchSize, Criterion};
use geo::{LineString, Polygon};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("relate overlapping 50-point polygons", |bencher| {
        let norway = geo_test_fixtures::norway_main::<f32>();
        let points = norway.0;

        let sub_polygon = {
            let points = points[0..50].to_vec();
            let mut exterior = LineString::<f32>::from(points);
            exterior.close();
            Polygon::new(exterior, vec![])
        };

        let polygon = {
            let points = points[40..90].to_vec();
            let mut exterior = LineString::<f32>::from(points);
            exterior.close();
            Polygon::new(exterior, vec![])
        };

        bencher.iter(|| {
            criterion::black_box(
                criterion::black_box(&polygon).relate(criterion::black_box(&sub_polygon)),
            );
        });
    });

    c.bench_function("entire jts test suite", |bencher| {
        let mut relate_tests = jts_test_runner::TestRunner::new();
        relate_tests.prepare_cases().unwrap();

        bencher.iter_batched(
            || relate_tests.clone(),
            |mut test_runner| {
                test_runner.run().unwrap();
                assert!(test_runner.failures().is_empty());
                assert!(!test_runner.successes().is_empty());
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("jts test suite matching *Relate*", |bencher| {
        let mut relate_tests =
            jts_test_runner::TestRunner::new().matching_filename_glob("*Relate*");
        relate_tests.prepare_cases().unwrap();

        bencher.iter_batched(
            || relate_tests.clone(),
            |mut test_runner| {
                test_runner.run().unwrap();
                assert!(test_runner.failures().is_empty());
                assert!(!test_runner.successes().is_empty());
            },
            BatchSize::SmallInput,
        );
    });

    c.bench_function("disjoint polygons", |bencher| {
        let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        let louisiana = Polygon::new(geo_test_fixtures::louisiana::<f64>(), vec![]);

        bencher.iter(|| {
            criterion::black_box(norway.relate(&louisiana));
        });
    });

    c.bench_function("large rotated polygons", |bencher| {
        let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        let rotated_norway = norway.rotate_around_center(20.0);

        bencher.iter(|| {
            criterion::black_box(norway.relate(&rotated_norway));
        });
    });

    c.bench_function("offset polygons", |bencher| {
        let norway = Polygon::new(geo_test_fixtures::norway_main::<f64>(), vec![]);
        let translated_norway = norway.translate(10.0, 10.0);

        bencher.iter(|| {
            criterion::black_box(norway.relate(&translated_norway));
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
