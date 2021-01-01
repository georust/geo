#![warn(missing_debug_implementations)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! The `geo-types` library provides geospatial primitive types for the [GeoRust] ecosystem.
//!
//! In most cases, you will only need to use this crate if you’re a crate author and want
//! compatibility with other GeoRust crates. Otherwise, the [`geo`](https://crates.io/crates/geo)
//! crate re-exports these types and provides geospatial algorithms.
//!
//! # Types
//!
//! - **[`Coordinate`]**: A two-dimensional coordinate. All [`Geometry`] types are composed of [`Coordinate`]s, though [`Coordinate`] itself is not a [`Geometry`] type.
//! - **[`Point`]**: A single point represented by one [`Coordinate`]
//! - **[`MultiPoint`]**: A collection of [`Point`]s
//! - **[`Line`]**: A line segment represented by two [`Coordinate`]s
//! - **[`LineString`]**: A series of contiguous line segments represented by two or more
//!   [`Coordinate`]s
//! - **[`MultiLineString`]**: A collection of [`LineString`]s
//! - **[`Polygon`]**: A bounded area represented by one [`LineString`] exterior ring, and zero or
//!   more [`LineString`] interior rings
//! - **[`MultiPolygon`]**: A collection of [`Polygon`]s
//! - **[`Rect`]**: An axis-aligned bounded rectangle represented by minimum and maximum
//!   [`Coordinate`]s
//! - **[`Triangle`]**: A bounded area represented by three [`Coordinate`] vertices
//! - **[`GeometryCollection`]**: A collection of [`Geometry`]s
//! - **[`Geometry`]**: An enumeration of all geometry types, excluding [`Coordinate`]
//!
//! # Semantics
//!
//! The geospatial types provided here aim to adhere to the [OpenGIS Simple feature access][OGC-SFA]
//! standards. Thus, the types here are inter-operable with other implementations of the standards:
//! [JTS], [GEOS], etc.
//!
//! [GeoRust]: https://georust.org
//! [OGC-SFA]: https://www.ogc.org/standards/sfa
//! [JTS]: https://github.com/locationtech/jts
//! [GEOS]: https://trac.osgeo.org/geos
extern crate num_traits;
use num_traits::{Num, NumCast};

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(feature = "rstar")]
extern crate rstar;

#[macro_use]
extern crate approx;

/// The type of an x or y value of a point/coordinate.
///
/// Floats (`f32` and `f64`) and Integers (`u8`, `i32` etc.)
/// implement this. Many algorithms only make sense for
/// Float types (like area, or length calculations).
pub trait CoordinateType: Num + Copy + NumCast + PartialOrd {}
// Little bit of a hack to make to make this work
impl<T: Num + Copy + NumCast + PartialOrd> CoordinateType for T {}

mod coordinate;
pub use crate::coordinate::Coordinate;

mod point;
pub use crate::point::Point;

mod multi_point;
pub use crate::multi_point::MultiPoint;

mod line;
pub use crate::line::Line;

mod line_string;
pub use crate::line_string::{LineString, PointsIter};

mod multi_line_string;
pub use crate::multi_line_string::MultiLineString;

mod polygon;
pub use crate::polygon::Polygon;

mod multi_polygon;
pub use crate::multi_polygon::MultiPolygon;

mod geometry;
pub use crate::geometry::Geometry;

mod geometry_collection;
pub use crate::geometry_collection::GeometryCollection;

mod triangle;
pub use crate::triangle::Triangle;

mod rect;
pub use crate::rect::{InvalidRectCoordinatesError, Rect};

#[macro_use]
mod macros;

#[doc(hidden)]
pub mod private_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn type_test() {
        let c = Coordinate {
            x: 40.02f64,
            y: 116.34,
        };

        let p = Point(c);

        let Point(c2) = p;
        assert_eq!(c, c2);
        assert_relative_eq!(c.x, c2.x);
        assert_relative_eq!(c.y, c2.y);

        let p: Point<f32> = (0f32, 1f32).into();
        assert_relative_eq!(p.x(), 0.);
        assert_relative_eq!(p.y(), 1.);
    }

    #[test]
    fn convert_types() {
        let p: Point<f32> = Point::new(0., 0.);
        let p1 = p.clone();
        let g: Geometry<f32> = p.into();
        let p2 = Point::try_from(g).unwrap();
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

        assert_eq!(p.exterior(), &exterior);
        assert_eq!(p.interiors(), &interiors[..]);
    }

    #[test]
    fn iters() {
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
        let _: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        let mut l1: LineString<_> = vec![(0., 0.), (1., 2.)].into();
        assert_eq!(l1[1], Coordinate { x: 1., y: 2. }); // index into linestring
        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        // index mutably into a linestring
        l1[0] = Coordinate { x: 1., y: 1. };
        assert_eq!(l1, vec![(1., 1.), (1., 2.)].into());
    }

    #[test]
    fn test_coordinate_types() {
        let p: Point<u8> = Point::new(0, 0);
        assert_eq!(p.x(), 0u8);

        let p: Point<i64> = Point::new(1_000_000, 0);
        assert_eq!(p.x(), 1_000_000i64);
    }

    #[cfg(feature = "rstar")]
    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test() {
        use rstar::primitives::Line as RStarLine;
        use rstar::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // difference in 15th decimal place
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[test]
    fn test_rects() {
        let r = Rect::new(Coordinate { x: -1., y: -1. }, Coordinate { x: 1., y: 1. });
        let p: Polygon<_> = r.into();
        assert_eq!(
            p,
            Polygon::new(
                vec![(-1., -1.), (1., -1.), (1., 1.), (-1., 1.), (-1., -1.)].into(),
                vec![]
            )
        );
    }
}
