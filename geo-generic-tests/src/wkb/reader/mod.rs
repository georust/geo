//! Parse buffers containing WKB-encoded geometries.

// Each of the data structures in this module is intended to mirror the [WKB
// spec](https://portal.ogc.org/files/?artifact_id=25355).

mod coord;
mod geometry;
mod geometry_collection;
mod linearring;
mod linestring;
mod multilinestring;
mod multipoint;
mod multipolygon;
mod point;
mod polygon;
mod util;

pub use geometry::Wkb;
pub use geometry_collection::GeometryCollection;
pub use linestring::LineString;
pub use multilinestring::MultiLineString;
pub use multipoint::MultiPoint;
pub use multipolygon::MultiPolygon;
pub use point::Point;
pub use polygon::Polygon;

use crate::wkb::error::WKBResult;

/// Parse a WKB byte slice into a geometry.
///
/// This is an alias for [`Wkb::try_new`].
#[allow(dead_code)]
pub fn read_wkb(buf: &[u8]) -> WKBResult<Wkb> {
    Wkb::try_new(buf)
}
