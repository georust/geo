#![feature(test)]

extern crate test;
use test::Bencher;

extern crate geo;
use geo::convexhull::ConvexHull;
use geo::Polygon;

#[bench]
fn convexhull(b: &mut Bencher) {
    let coords = include!("../src/algorithm/test_fixtures/norway_main.rs");
    let poly = Polygon::new(coords.into(), vec![]);
    b.iter(|| {
        poly.convex_hull();
    });
}
