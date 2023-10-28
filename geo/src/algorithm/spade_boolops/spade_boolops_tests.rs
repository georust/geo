use super::*;
use geo_types::*;

#[test]
fn basic_intersection_compiles() {
    let zero = Coord::zero();
    let one = Coord { x: 1.0, y: 1.0 };
    let rect1 = Rect::new(zero, one * 2.0);
    let rect2 = Rect::new(one, one * 3.0);

    SpadeBoolops::intersection(&rect1.to_polygon(), &rect2.to_polygon()).unwrap();
}
