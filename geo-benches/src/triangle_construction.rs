use criterion::{Criterion, criterion_group, criterion_main};
use geo_types::{Coord, Triangle};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Generate a fixed set of random coordinate triples for deterministic benchmarks.
fn random_coord_triples(n: usize) -> Vec<[Coord<f64>; 3]> {
    let mut rng = StdRng::seed_from_u64(42);
    (0..n)
        .map(|_| {
            [
                Coord {
                    x: rng.random_range(-180.0..180.0),
                    y: rng.random_range(-90.0..90.0),
                },
                Coord {
                    x: rng.random_range(-180.0..180.0),
                    y: rng.random_range(-90.0..90.0),
                },
                Coord {
                    x: rng.random_range(-180.0..180.0),
                    y: rng.random_range(-90.0..90.0),
                },
            ]
        })
        .collect()
}

fn criterion_benchmark(c: &mut Criterion) {
    let triples = random_coord_triples(10_000);

    c.bench_function("triangle_new (orientation check)", |bencher| {
        bencher.iter(|| {
            for coords in criterion::black_box(&triples) {
                criterion::black_box(Triangle::new(coords[0], coords[1], coords[2]));
            }
        });
    });

    c.bench_function("triangle_from (no orientation check)", |bencher| {
        bencher.iter(|| {
            for coords in criterion::black_box(&triples) {
                criterion::black_box(Triangle::from(*coords));
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
