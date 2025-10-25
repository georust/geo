//! Provides structs that implement the geo-traits crate's traits.

pub use self::coord::Coord;
pub use self::geometry::Geometry;
// pub use self::geometry_type::GeometryType;
// pub use self::geometrycollection::GeometryCollection;
pub use self::linestring::LineString;
pub use self::multilinestring::MultiLineString;
pub use self::multipoint::MultiPoint;
pub use self::multipolygon::MultiPolygon;
pub use self::point::Point;
pub use self::polygon::Polygon;

mod coord;
mod geometry;
// mod geometry_type;
// mod geometrycollection;
mod linestring;
mod multilinestring;
mod multipoint;
mod multipolygon;
mod point;
mod polygon;
