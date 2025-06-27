use criterion::{black_box, criterion_group, criterion_main, Criterion};
use geo::algorithm::Relate;
use geo::MultiPolygon;
use geo::PreparedGeometry;

fn criterion_benchmark(c: &mut Criterion) {
    let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
    let zone_polygons = geo_test_fixtures::nl_zones();
    c.bench_function("build prepared polygons", |bencher| {
        bencher.iter(|| {
            let plot_polygons = plot_polygons
                .iter()
                .map(PreparedGeometry::from)
                .collect::<Vec<_>>();

            let zone_polygons = zone_polygons
                .iter()
                .map(PreparedGeometry::from)
                .collect::<Vec<_>>();

            black_box((&plot_polygons, &zone_polygons));
        });
    });

    c.bench_function("relate already prepared polygons", |bencher| {
        let plot_polygons = plot_polygons
            .iter()
            .map(PreparedGeometry::from)
            .collect::<Vec<_>>();

        let zone_polygons = zone_polygons
            .iter()
            .map(PreparedGeometry::from)
            .collect::<Vec<_>>();

        bencher.iter(|| {
            for a in &plot_polygons {
                for b in &zone_polygons {
                    black_box(a.relate(b).is_intersects());
                }
            }
        });
    });

    c.bench_function("build and relate prepared polygons", |bencher| {
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            let plot_polygons = plot_polygons
                .iter()
                .map(PreparedGeometry::from)
                .collect::<Vec<_>>();

            let zone_polygons = zone_polygons
                .iter()
                .map(PreparedGeometry::from)
                .collect::<Vec<_>>();

            for a in &plot_polygons {
                for b in &zone_polygons {
                    if black_box(a.relate(b).is_intersects()) {
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

    let mut slow_group = c.benchmark_group("unprepared polygons");
    slow_group.sample_size(10);
    slow_group.bench_function("relate unprepared polygons", |bencher| {
        let plot_polygons: MultiPolygon = geo_test_fixtures::nl_plots_wgs84();
        let zone_polygons = geo_test_fixtures::nl_zones();

        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_polygons {
                for b in &zone_polygons {
                    if black_box(a.relate(b).is_intersects()) {
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
