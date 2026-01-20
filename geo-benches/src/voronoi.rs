use criterion::{criterion_group, criterion_main};
use geo::geometry::{Coord, LineString, Polygon};
use geo::{Voronoi, VoronoiClip, VoronoiParams};
use geo_types::MultiPoint;

fn criterion_benchmark(c: &mut criterion::Criterion) {
    // Load Islington post boxes (147 unique points)
    let points_mp: MultiPoint<f64> = geo_test_fixtures::islington_post_boxes();
    let coords: Vec<Coord<f64>> = points_mp.iter().map(|p| p.0).collect();
    let mut ring_coords = coords.clone();
    ring_coords.push(coords[0]);
    let points = Polygon::new(LineString::from(ring_coords), vec![]);

    c.bench_function("voronoi_cells - Islington (padded)", |bencher| {
        bencher.iter(|| {
            let cells = points.voronoi_cells().unwrap();
            criterion::black_box(cells);
        });
    });

    c.bench_function("voronoi_cells - Islington (envelope)", |bencher| {
        bencher.iter(|| {
            let cells = points
                .voronoi_cells_with_params(VoronoiParams::new().clip(VoronoiClip::Envelope))
                .unwrap();
            criterion::black_box(cells);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
