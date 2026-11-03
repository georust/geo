use criterion::{Criterion, criterion_group, criterion_main};
use geo::{ClosestPoint, CoordFloat, GeoFloat, Line, LineString, Point};
use std::hint::black_box;

// A fixed-seed generator, so that every run measures the same inputs.
struct Lcg(u64);

impl Lcg {
    // A value in [-1, 1).
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 11) as f64) / ((1_u64 << 53) as f64) * 2.0 - 1.0
    }
}

// Segments with coordinates in [-scale, scale) and query points with
// coordinates in [-query_scale, query_scale).
fn pairs<F: CoordFloat>(scale: f64, query_scale: f64) -> Vec<(Line<F>, Point<F>)> {
    let mut rng = Lcg(0x5eed);
    let mut coord = |s: f64| F::from(rng.next_f64() * s).unwrap();
    (0..1024)
        .map(|_| {
            let start = (coord(scale), coord(scale));
            let end = (coord(scale), coord(scale));
            let point = Point::new(coord(query_scale), coord(query_scale));
            (Line::new(start, end), point)
        })
        .collect()
}

fn bench_pairs<F: GeoFloat>(c: &mut Criterion, name: &str, input: Vec<(Line<F>, Point<F>)>) {
    c.bench_function(name, |bencher| {
        bencher.iter(|| {
            for (line, point) in black_box(&input) {
                black_box(line.closest_point(point));
            }
        });
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_pairs(c, "closest_point f64", pairs::<f64>(1e3, 1e3));
    bench_pairs(c, "closest_point f32", pairs::<f32>(1e3, 1e3));
    bench_pairs(
        c,
        "closest_point tiny segments",
        pairs::<f64>(1e-162, 1e-162),
    );
    bench_pairs(
        c,
        "closest_point tiny segments, distant points",
        pairs::<f64>(1e-300, 1e300),
    );
    bench_pairs(c, "closest_point long segments", pairs::<f64>(1e200, 1e200));
    bench_pairs(
        c,
        "closest_point overflowing differences",
        pairs::<f64>(1.7e308, 1.7e308),
    );

    let norway: LineString<f64> = geo_test_fixtures::norway_main();
    let points: Vec<Point<f64>> = pairs::<f64>(1.0, 15.0)
        .into_iter()
        .take(64)
        .map(|(_, point)| point + Point::new(15.0, 65.0))
        .collect();
    c.bench_function("closest_point norway linestring", |bencher| {
        bencher.iter(|| {
            for point in black_box(&points) {
                black_box(norway.closest_point(point));
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
