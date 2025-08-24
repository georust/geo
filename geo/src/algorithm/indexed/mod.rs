/// Geometries that are backed by an [R*-tree](https://en.wikipedia.org/wiki/R*-tree) spatial index for faster bulk operations
pub mod prepared_geometry;
pub use prepared_geometry::PreparedGeometry;

/// A MultiPolygon backed by an [interval tree](https://en.wikipedia.org/wiki/Interval_tree) spatial index for fast point containment queries
pub mod interval_tree_multipolygon;
pub use interval_tree_multipolygon::IntervalTreeMultiPolygon;
