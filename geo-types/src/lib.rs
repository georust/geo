extern crate num_traits;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(feature = "spade")]
extern crate spade;

#[cfg(feature = "spade")]
mod algorithms;

use num_traits::{Num, NumCast};

/// The type of an x or y value of a point/coordinate.
///
/// Floats (`f32` and `f64`) and Integers (`u8`, `i32` etc.) implement this. Many algorithms only
/// make sense for Float types (like area, or length calculations).
pub trait CoordinateType: Num + Copy + NumCast + PartialOrd {}
// Little bit of a hack to make to make this work
impl<T: Num + Copy + NumCast + PartialOrd> CoordinateType for T {}

mod coordinate;
pub use coordinate::Coordinate;

mod point;
pub use point::Point;

mod multi_point;
pub use multi_point::MultiPoint;

mod line;
pub use line::Line;

pub mod line_string;
pub use line_string::LineString;

mod multi_line_string;
pub use multi_line_string::MultiLineString;

mod polygon;
pub use polygon::Polygon;

mod multi_polygon;
pub use multi_polygon::MultiPolygon;

mod geometry;
pub use geometry::Geometry;

mod geometry_collection;
pub use geometry_collection::GeometryCollection;

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "spade")]
    use spade::SpatialObject;

    #[test]
    fn type_test() {
        let c = Coordinate {
            x: 40.02f64,
            y: 116.34,
        };

        let p = Point(c);

        let Point(c2) = p;
        assert_eq!(c, c2);
        assert_eq!(c.x, c2.x);
        assert_eq!(c.y, c2.y);

        let p: Point<f32> = (0f32, 1f32).into();
        assert_eq!(p.x(), 0.);
        assert_eq!(p.y(), 1.);
    }

    #[test]
    fn convert_types() {
        let p: Point<f32> = Point::new(0., 0.);
        let p1 = p.clone();
        let g: Geometry<f32> = p.into();
        let p2 = g.as_point().unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn polygon_new_test() {
        let exterior = LineString(vec![
            Coordinate { x: 0., y: 0. },
            Coordinate { x: 1., y: 1. },
            Coordinate { x: 1., y: 0. },
            Coordinate { x: 0., y: 0. },
        ]);
        let interiors = vec![LineString(vec![
            Coordinate { x: 0.1, y: 0.1 },
            Coordinate { x: 0.9, y: 0.9 },
            Coordinate { x: 0.9, y: 0.1 },
            Coordinate { x: 0.1, y: 0.1 },
        ])];
        let p = Polygon::new(exterior.clone(), interiors.clone());

        assert_eq!(p.exterior, exterior);
        assert_eq!(p.interiors, interiors);
    }

    #[test]
    fn iters() {
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into();
        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();
    }

    #[test]
    fn test_coordinate_types() {
        let p: Point<u8> = Point::new(0, 0);
        assert_eq!(p.x(), 0u8);

        let p: Point<i64> = Point::new(1_000_000, 0);
        assert_eq!(p.x(), 1_000_000i64);
    }

    #[cfg(feature = "spade")]
    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test() {
        use spade::primitives::SimpleEdge;
        let se = SimpleEdge::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 5., y: 5. },
        );
        assert_eq!(se.mbr(), l.mbr());
        // difference in 15th decimal place
        assert_eq!(26.0, se.distance2(&Point::new(4.0, 10.0)));
        assert_eq!(25.999999999999996, l.distance2(&Point::new(4.0, 10.0)));
    }
}
