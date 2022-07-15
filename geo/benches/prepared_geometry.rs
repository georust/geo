use criterion::{criterion_group, criterion_main, Criterion};
use geo::algorithm::Relate;
use geo::geometry::Polygon;
use geo::prepared_geometry::PreparedGeometry;
use geo::relate::IntersectionMatrix;
use geo::GeoFloat;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("relate prepared polygon", |bencher| {
        let plot_polygons: Vec<PreparedGeometry> = geo_test_fixtures::nl_plots()
            .into_iter()
            .map(PreparedGeometry::from)
            .collect();
        let zone_polygons: Vec<PreparedGeometry> = geo_test_fixtures::nl_zones()
            .into_iter()
            .map(PreparedGeometry::from)
            .collect();
        bencher.iter(|| {
            let mut intersects = 0;
            let mut non_intersects = 0;

            for a in &plot_polygons {
                for b in &zone_polygons {
                    if criterion::black_box(a.relate(b).is_intersects()) {
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
