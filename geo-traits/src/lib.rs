//! An initial, in-progress implementation of [geometry access
//! traits](https://github.com/georust/geo/discussions/838).
//!
//! The idea is that functions should be able to operate on and consume geospatial vector data from
//! _any_ source without overhead, not limited to just the layout defined in the [`geo`] crate.
//!
//! The main work for this is happening in the [`geo`] repository (see
//! [here](https://github.com/georust/geo/pull/1019)) but that is vendored into this repository for
//! use internally, such as in the WKB parser.

pub use coord::CoordTrait;
pub use geometry::{GeometryTrait, GeometryType};
pub use geometry_collection::GeometryCollectionTrait;
pub use iterator::{
    GeometryCollectionIterator, LineStringIterator, MultiLineStringIterator, MultiPointIterator,
    MultiPolygonIterator, PolygonInteriorIterator,
};
pub use line_string::LineStringTrait;
pub use multi_line_string::MultiLineStringTrait;
pub use multi_point::MultiPointTrait;
pub use multi_polygon::MultiPolygonTrait;
pub use point::PointTrait;
pub use polygon::PolygonTrait;
pub use rect::RectTrait;

mod coord;
mod geometry;
mod geometry_collection;
mod iterator;
mod line_string;
mod multi_line_string;
mod multi_point;
mod multi_polygon;
mod point;
mod polygon;
mod rect;
