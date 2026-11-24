use criterion::{Criterion, black_box, criterion_group, criterion_main};
use geo::PreparedGeometry;
use geo::algorithm::Relate;
use geo_types::{Coord, LineString, MultiLineString, MultiPolygon, Point, Rect};

/// `members` short line strings of `points` coordinates each, laid out on
/// a grid: a linear geometry whose prepared point-locator state (per-line
/// envelopes, boundary points) is large relative to one evaluation.
fn grid_multi_line_string(members: usize, points: usize) -> MultiLineString<f64> {
    let side = (members as f64).sqrt().ceil() as usize;
    let lines = (0..members)
        .map(|i| {
            let (gx, gy) = ((i % side) as f64 * 10.0, (i / side) as f64 * 10.0);
            LineString::from(
                (0..points)
                    .map(|k| Coord {
                        x: gx + k as f64 * (8.0 / points as f64),
                        y: gy + (k % 2) as f64,
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    MultiLineString::new(lines)
}

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

    // A linear A geometry: the point-locator state is rebuilt per call
    // unless prepared.
    let mls = grid_multi_line_string(1_000, 10);
    let prepared_mls = PreparedGeometry::from(&mls);
    let point = Point::new(4.0, 5.0);
    let polygon = Rect::new(Coord { x: 3.0, y: 4.0 }, Coord { x: 5.0, y: 6.0 }).to_polygon();
    c.bench_function("relate prepared multilinestring with point", |bencher| {
        bencher.iter(|| black_box(prepared_mls.relate(&point).is_intersects()));
    });
    c.bench_function("relate unprepared multilinestring with point", |bencher| {
        bencher.iter(|| black_box(mls.relate(&point).is_intersects()));
    });
    c.bench_function("relate prepared multilinestring with polygon", |bencher| {
        bencher.iter(|| black_box(prepared_mls.relate(&polygon).is_intersects()));
    });
    c.bench_function(
        "relate unprepared multilinestring with polygon",
        |bencher| {
            bencher.iter(|| black_box(mls.relate(&polygon).is_intersects()));
        },
    );

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
