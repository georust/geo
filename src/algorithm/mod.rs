/// Calculation of the centroid.
pub mod centroid;
/// Checks if the geometry A is completely inside the B geometry.
pub mod contains;
/// Checks if the geometry A intersects the geometry B.
pub mod intersects;
/// Returns the area of the surface of a geometry.
pub mod area;
/// Returns the bearing to another Point in degrees.
pub mod bearing;
/// Returns the length of a line.
pub mod length;
/// Returns the Euclidean distance between two geometries.
pub mod distance;
/// Returns a new Point using distance and bearing.
pub mod haversine_destination;
/// Returns the Haversine distance between two geometries.
pub mod haversine_distance;
/// Returns the Bbox of a geometry.
pub mod boundingbox;
/// Simplifies geometries using the Ramer-Douglas-Peucker algorithm.
pub mod simplify;
/// Simplifies geometries using the Visvalingam-Whyatt algorithm. Includes a topology-preserving variant.
pub mod simplifyvw;
/// Calculates the convex hull of a geometry.
pub mod convexhull;
/// Orients a Polygon's exterior and interior rings.
pub mod orient;
/// Returns the extreme indices of a `Polygon`, `MultiPolygon`, or `MultiPoint`.
pub mod extremes;
/// Rotates a geometry around either its centroid or a point by an angle, given in degrees.
pub mod rotate;
/// Translates a geometry along the given offsets.
pub mod translate;
/// Apply a function to all coordinates
pub mod map_coords;
/// Determine the closest point between two objects.
pub mod closest_point;
/// Produces geometry from PostGIS.
#[cfg(feature = "postgis-integration")]
pub mod from_postgis;
/// Converts geometry into PostGIS types.
#[cfg(feature = "postgis-integration")]
pub mod to_postgis;
/// Returns the Haversine length of a line.
pub mod haversine_length;
/// Calculate, and work with, the winding order of Linestrings
pub mod winding_order;
/// Apply a coordinate projection transformation to a Geometry using `rust-proj`
#[cfg(feature = "reprojection")]
pub mod reproject;
