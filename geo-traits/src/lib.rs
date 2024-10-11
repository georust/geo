//! A trait-based interface for geospatial vector data interchange in Rust.
//!
//! This crate contains a set of traits based on the Simple Features standard for geospatial vector
//! data. These traits are designed to make it easy to operate on and consume geometries throughout
//! the Rust ecosystem without knowing library-specific APIs or memory layouts.
//!
//! It is expected that accessing any individual coordinate or value from a geometry is
//! **constant-time**. This means that when implementing these traits on a format like WKB that
//! requires linear-time search to locate coordinates, the WKB wrapper should have already
//! undergone an initial pass to find the relevant byte offsets where coordinate sequences start
//! and end.
//!
//! This interface will usually but not always be zero-copy. Coordinate access is expected to be
//! constant-time but not necessarily _free_. For example, WKB is not aligned and may use a
//! different endianness than the current machine, so individual values may need to be cloned on
//! read.

pub use dimension::Dimension;
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

mod dimension;
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
