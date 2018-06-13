/// Returns the area of the surface of a geometry.
pub mod area;
/// Returns the bearing to another Point in degrees.
pub mod bearing;
/// Returns the Bbox of a geometry.
pub mod boundingbox;
/// Calculation of the centroid of a geometry.
pub mod centroid;
/// Determine the minimum distance between two objects.
pub mod closest_point;
/// Checks if the geometry A is completely inside the B geometry.
pub mod contains;
/// Calculates the convex hull of a geometry.
pub mod convexhull;
/// Returns the Euclidean distance between two geometries.
pub mod euclidean_distance;
/// Returns the length of a line.
pub mod euclidean_length;
/// Returns the extreme indices of a `Polygon`, `MultiPolygon`, or `MultiPoint`.
pub mod extremes;
/// Produces geometry from PostGIS.
#[cfg(feature = "postgis-integration")]
pub mod from_postgis;
/// Returns a new Point using distance and bearing.
pub mod haversine_destination;
/// Returns the Haversine distance between two geometries.
pub mod haversine_distance;
/// Returns a new Point along a great circle between two points.
pub mod haversine_intermediate;
/// Returns the Haversine length of a line.
pub mod haversine_length;
/// Checks if the geometry A intersects the geometry B.
pub mod intersects;
/// Apply a function to all coordinates.
pub mod map_coords;
/// Orients a Polygon's exterior and interior rings.
pub mod orient;
/// Helper functions for the "fast path" variant of the Polygon-Polygon distance method.
pub(crate) mod polygon_distance_fast_path;
/// Coordinate projections and transformations using [PROJ](http://proj4.org) v5.0.x.
#[cfg(feature = "use-proj")]
pub mod proj;
/// Rotate a geometry around either its centroid or a point by an angle given in degrees.
pub mod rotate;
/// Simplifies geometries using the Ramer-Douglas-Peucker algorithm.
pub mod simplify;
/// Simplifies geometries using the Visvalingam-Whyatt algorithm. Includes a topology-preserving variant.
pub mod simplifyvw;
/// Converts geometries into PostGIS types.
#[cfg(feature = "postgis-integration")]
pub mod to_postgis;
/// Translates a geometry along the given offsets.
pub mod translate;
/// Calculate and work with the winding order of Linestrings.
pub mod winding_order;
/// Calculate the Vincenty distance between Points.
pub mod vincenty_distance;
/// Calculate the Vincenty length of a LineString.
pub mod vincenty_length;
