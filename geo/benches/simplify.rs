#![feature(test)]

extern crate test;
use test::Bencher;

extern crate geo;
use geo::simplifyvw::{SimplifyVW, SimplifyVWPreserve};
use geo::{LineString, Polygon};

#[bench]
fn simplify_vw_simple(b: &mut Bencher) {
    let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
    let ls: LineString<_> = points.into();
    b.iter(|| {
        ls.simplifyvw(&0.0005);
    });
}

#[bench]
fn simplify_vwp(b: &mut Bencher) {
    let points = include!("../src/algorithm/test_fixtures/norway_main.rs");
    let p = Polygon::new(points.into(), vec![]);
    b.iter(|| {
        p.simplifyvw_preserve(&0.0005);
    });
}
