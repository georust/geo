#![feature(test)]

extern crate test;
use test::Bencher;

extern crate geo;
use geo::contains::Contains;
use geo::{Point, Polygon};

#[bench]
fn point_in_polygon(b: &mut Bencher) {
    let v = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(0.0, 0.0),
    ];
    let polygon = Polygon::new(v.into(), vec![]);
    let in_candidate = Point::new(0.5, 0.1);
    b.iter(|| {
        test::black_box(polygon.contains(&in_candidate));
    });
}

#[bench]
fn point_outside_polygon(b: &mut Bencher) {
    let v = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(1.0, 1.0),
        Point::new(0.0, 0.0),
    ];
    let polygon = Polygon::new(v.into(), vec![]);
    let out_candidate = Point::new(2.0, 2.0);
    b.iter(|| {
        test::black_box(polygon.contains(&out_candidate));
    });
}
