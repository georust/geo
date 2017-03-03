/// Calculation of the centroid.
pub mod centroid;
/// Checks if the geometry A is completely inside the B geometry.
pub mod contains;
/// Checks if the geometry A intersects the geometry B.
pub mod intersects;
/// Returns the area of the surface of a geometry.
pub mod area;
/// Returns the length of a line.
pub mod length;
/// Returns the Euclidean distance between two geometries.
pub mod distance;
/// Returns the Haversine distance between two geometries.
pub mod haversine_distance;
/// Returns the Bbox of a geometry.
pub mod boundingbox;
/// Simplifies a `LineString` using the Ramer-Douglas-Peucker algorithm.
pub mod simplify;
/// Simplifies a `LineString` using the Visvalingam-Whyatt algorithm.
pub mod simplifyvw;
/// Calculates the convex hull of a geometry.
pub mod convexhull;
