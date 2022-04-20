#![warn(missing_debug_implementations)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! The `geo-types` library defines geometric types for the [GeoRust] ecosystem.
//!
//! In most cases, you will only need to use this crate if you’re a crate author and want
//! compatibility with other GeoRust crates. Otherwise, the [`geo`](https://crates.io/crates/geo)
//! crate re-exports these types and additionally provides geospatial algorithms.
//!
//! ## Geometries
//!
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
//! ## Coordinates and Numeric Types
//!
//! - **[`Coordinate`]**: A two-dimensional coordinate. All geometry types are composed of [`Coordinate`]s, though [`Coordinate`] itself is not a [`Geometry`] type. See [`Point`] for a single coordinate geometry.
//!
//! By default, coordinates are 64-bit floating point numbers, but this is generic, and you may specify any numeric type that implements [`CoordNum`] or [`CoordFloat`]. As well as [`f64`], this includes common numeric types like [`f32`], [`i32`], [`i64`], etc.
//!
//! ```rust
//! use geo_types::Point;
//!
//! // Geometries are f64 by default
//! let point: Point = Point::new(1.0, 2.0);
//! assert_eq!(std::mem::size_of::<Point>(), 64 * 2 / 8);
//!
//! // You can be explicit about the numeric type.
//! let f64_point: Point<f64> = Point::new(1.0, 2.0);
//! assert_eq!(std::mem::size_of::<Point<f64>>(), 64 * 2 / 8);
//!
//! // Or specify some non-default numeric type
//! let f32_point: Point<f32> = Point::new(1.0, 2.0);
//! assert_eq!(std::mem::size_of::<Point<f32>>(), 32 * 2 / 8);
//!
//! // Integer geometries are supported too, though not all
//! // algorithms will be implemented for all numeric types.
//! let i32_point: Point<i32> = Point::new(1, 2);
//! assert_eq!(std::mem::size_of::<Point<i32>>(), 32 * 2 / 8);
//! ```
//!
//! # Semantics
//!
//! The geospatial types provided here aim to adhere to the [OpenGIS Simple feature access][OGC-SFA]
//! standards. Thus, the types here are inter-operable with other implementations of the standards:
//! [JTS], [GEOS], etc.
//!
//! # Features
//!
//! The following optional [Cargo features] are available:
//!
//! - `approx`: Allows geometry types to be checked for approximate equality with [approx]
//! - `arbitrary`: Allows geometry types to be created from unstructured input with [arbitrary]
//! - `serde`: Allows geometry types to be serialized and deserialized with [Serde]
//! - `use-rstar_0_8`: Allows geometry types to be inserted into [rstar] R*-trees (`rstar v0.8`)
//! - `use-rstar_0_9`: Allows geometry types to be inserted into [rstar] R*-trees (`rstar v0.9`)
//!
//! [approx]: https://github.com/brendanzab/approx
//! [arbitrary]: https://github.com/rust-fuzz/arbitrary
//! [Cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
//! [GeoRust]: https://georust.org
//! [GEOS]: https://trac.osgeo.org/geos
//! [JTS]: https://github.com/locationtech/jts
//! [OGC-SFA]: https://www.ogc.org/standards/sfa
//! [rstar]: https://github.com/Stoeoef/rstar
//! [Serde]: https://serde.rs/
extern crate num_traits;
use num_traits::{Float, Num, NumCast};
use std::fmt::Debug;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[cfg(feature = "rstar_0_8")]
extern crate rstar_0_8;

#[cfg(test)]
#[macro_use]
extern crate approx;

#[deprecated(since = "0.7.0", note = "use `CoordFloat` or `CoordNum` instead")]
pub trait CoordinateType: Num + Copy + NumCast + PartialOrd + Debug {}
#[allow(deprecated)]
impl<T: Num + Copy + NumCast + PartialOrd + Debug> CoordinateType for T {}

/// For algorithms which can use both integer **and** floating point `Point`s/`Coordinate`s
///
/// Floats (`f32` and `f64`) and Integers (`u8`, `i32` etc.) implement this.
///
/// For algorithms which only make sense for floating point, like area or length calculations,
/// see [CoordFloat](trait.CoordFloat.html).
#[allow(deprecated)]
pub trait CoordNum: CoordinateType + Debug {}
#[allow(deprecated)]
impl<T: CoordinateType + Debug> CoordNum for T {}

/// For algorithms which can only use floating point `Point`s/`Coordinate`s, like area or length calculations
pub trait CoordFloat: CoordNum + Float {}
impl<T: CoordNum + Float> CoordFloat for T {}

mod novalue;
pub use crate::novalue::NoValue;

mod coordinate;
pub use crate::coordinate::{Coordinate, Coordinate3D, Coordinate3DM, CoordinateM};

mod point;
pub use crate::point::{Point, Point3D, Point3DM, PointM};

mod multi_point;
pub use crate::multi_point::{MultiPoint, MultiPoint3D, MultiPoint3DM, MultiPointM};

mod line;
pub use crate::line::{Line, Line3D, Line3DM, LineM};

mod line_string;
pub use crate::line_string::{
    LineString, LineString3D, LineString3DM, LineStringM, PointsIter, PointsIter3D, PointsIter3DM,
    PointsIterM,
};

mod multi_line_string;
pub use crate::multi_line_string::{
    MultiLineString, MultiLineString3D, MultiLineString3DM, MultiLineStringM,
};

mod polygon;
pub use crate::polygon::{Polygon, Polygon3D, Polygon3DM, PolygonM};

mod multi_polygon;
pub use crate::multi_polygon::{MultiPolygon, MultiPolygon3D, MultiPolygon3DM, MultiPolygonM};

mod geometry;
pub use crate::geometry::{Geometry, Geometry3D, Geometry3DM, GeometryM};

mod geometry_collection;
pub use crate::geometry_collection::{
    GeometryCollection, GeometryCollection3D, GeometryCollection3DM, GeometryCollectionM,
};

mod triangle;
pub use crate::triangle::{Triangle, Triangle3D, Triangle3DM, TriangleM};

mod rect;
pub use crate::rect::{Rect, Rect3D, Rect3DM, RectM};

pub mod geometry;
pub use geometry::*;

pub use geometry::line_string::PointsIter;

#[allow(deprecated)]
pub use geometry::rect::InvalidRectCoordinatesError;

mod error;
pub use error::Error;

#[macro_use]
mod macros;

#[cfg(feature = "arbitrary")]
mod arbitrary;

#[cfg(any(feature = "rstar_0_8", feature = "rstar_0_9"))]
#[doc(hidden)]
pub mod private_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn type_test() {
        let c = coord! {
            x: 40.02f64,
            y: 116.34,
        };

        let p = Point::from(c);

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
        let p1 = p;
        let g: Geometry<f32> = p.into();
        let p2 = Point::try_from(g).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn polygon_new_test() {
        let exterior = LineString::new(vec![
            coord! { x: 0., y: 0. },
            coord! { x: 1., y: 1. },
            coord! { x: 1., y: 0. },
            coord! { x: 0., y: 0. },
        ]);
        let interiors = vec![LineString::new(vec![
            coord! { x: 0.1, y: 0.1 },
            coord! { x: 0.9, y: 0.9 },
            coord! { x: 0.9, y: 0.1 },
            coord! { x: 0.1, y: 0.1 },
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
        assert_eq!(l1[1], coord! { x: 1., y: 2. }); // index into linestring
        let _: LineString<_> = vec![(0., 0.), (1., 2.)].into_iter().collect();

        // index mutably into a linestring
        l1[0] = coord! { x: 1., y: 1. };
        assert_eq!(l1, vec![(1., 1.), (1., 2.)].into());
    }

    #[test]
    fn test_coordinate_types() {
        let p: Point<u8> = Point::new(0, 0);
        assert_eq!(p.x(), 0u8);

        let p: Point<i64> = Point::new(1_000_000, 0);
        assert_eq!(p.x(), 1_000_000i64);
    }

    #[cfg(feature = "rstar_0_8")]
    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test() {
        use rstar_0_8::primitives::Line as RStarLine;
        use rstar_0_8::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // difference in 15th decimal place
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[cfg(feature = "rstar_0_9")]
    #[test]
    /// ensure Line's SpatialObject impl is correct
    fn line_test_0_9() {
        use rstar_0_9::primitives::Line as RStarLine;
        use rstar_0_9::{PointDistance, RTreeObject};

        let rl = RStarLine::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0));
        let l = Line::new(coord! { x: 0.0, y: 0.0 }, coord! { x: 5., y: 5. });
        assert_eq!(rl.envelope(), l.envelope());
        // difference in 15th decimal place
        assert_relative_eq!(26.0, rl.distance_2(&Point::new(4.0, 10.0)));
        assert_relative_eq!(25.999999999999996, l.distance_2(&Point::new(4.0, 10.0)));
    }

    #[test]
    fn test_rects() {
        let r = Rect::new(coord! { x: -1., y: -1. }, coord! { x: 1., y: 1. });
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
