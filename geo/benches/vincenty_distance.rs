#![feature(test)]

extern crate geo;
extern crate test;

use geo::algorithm::vincenty_distance::VincentyDistance;

const NITER: usize = 10_000;

#[bench]
fn bench_vincenty(bencher: &mut test::Bencher) {
    let a = geo::Point::<f64>::new(17.107558, 48.148636);
    let b = geo::Point::<f64>::new(16.372477, 48.208810);
    bencher.iter(|| {
        for _ in 0..NITER {
            test::black_box(a.vincenty_distance(&b));
        }
    });
}
