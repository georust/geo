//! Provides structs that implement the geo-traits crate's traits.

pub use self::coord::Coord;
pub use self::geometry::Geometry;
pub use self::geometry_collection::GeometryCollection;
pub use self::line_string::LineString;
pub use self::multi_line_string::MultiLineString;
pub use self::multi_point::MultiPoint;
pub use self::multi_polygon::MultiPolygon;
pub use self::point::Point;
pub use self::polygon::Polygon;

mod coord;
mod geometry;
mod geometry_collection;
mod line_string;
mod multi_line_string;
mod multi_point;
mod multi_polygon;
mod point;
mod polygon;
