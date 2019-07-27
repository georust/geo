#![feature(test)]

extern crate test;
use test::Bencher;

extern crate geo;
use geo::contains::Contains;
use geo::{polygon, Point};

#[bench]
fn point_in_polygon(b: &mut Bencher) {
    let polygon = polygon![
        (x: 0.0, y: 0.0),
        (x: 1.0, y: 0.0),
        (x: 1.0, y: 1.0),
        (x: 0.0, y: 0.0),
    ];
    let in_candidate = Point::new(0.5, 0.1);
    b.iter(|| {
        test::black_box(polygon.contains(&in_candidate));
    });
}

#[bench]
fn point_outside_polygon(b: &mut Bencher) {
    let polygon = polygon![
        (x: 0.0, y: 0.0),
        (x: 1.0, y: 0.0),
        (x: 1.0, y: 1.0),
        (x: 0.0, y: 0.0),
    ];
    let out_candidate = Point::new(2.0, 2.0);
    b.iter(|| {
        test::black_box(polygon.contains(&out_candidate));
    });
}
