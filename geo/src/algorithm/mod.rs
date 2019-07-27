/// Calculate the area of the surface of a `Geometry`.
pub mod area;
/// Calculate the bearing to another `Point`, in degrees.
pub mod bearing;
/// Calculate the bounding rectangle of a `Geometry`.
pub mod bounding_rect;
/// Calculate the centroid of a `Geometry`.
pub mod centroid;
/// Signed approximate geodesic area of a geometry.
pub mod chamberlain_duquette_area;
/// Calculate the minimum distance between two `Geometries`.
pub mod closest_point;
/// Determine whether `Geometry` `A` is completely enclosed by `Geometry` `B`.
pub mod contains;
/// Calculate the convex hull of a `Geometry`.
pub mod convexhull;
/// Calculate the Euclidean distance between two `Geometries`.
pub mod euclidean_distance;
/// Calculate the length of a planar line between two `Geometries`.
pub mod euclidean_length;
/// Calculate the extreme indices of a `Polygon`, `MultiPolygon`, or `MultiPoint`.
pub mod extremes;
/// Calculate the Frechet distance between two `LineStrings`.
pub mod frechet_distance;
/// Produces a `Geometry` from PostGIS.
#[cfg(feature = "postgis-integration")]
pub mod from_postgis;
/// Calculate a new Point given a distance and a bearing.
pub mod haversine_destination;
/// Calculate the Haversine distance between two `Geometries`.
pub mod haversine_distance;
/// Calculate a new `Point` lying on a Great Circle arc between two `Point`s.
pub mod haversine_intermediate;
/// Calculate the Haversine length of a Line.
pub mod haversine_length;
/// Determine whether `Geometry` `A` intersects `Geometry` `B`.
pub mod intersects;
/// Apply a function to all `Coordinates` of a `Geometry`.
pub mod map_coords;
/// Orient a `Polygon`'s exterior and interior rings.
pub mod orient;
/// Helper functions for the "fast path" variant of the Polygon-Polygon Euclidean distance method.
pub(crate) mod polygon_distance_fast_path;
/// Coordinate projections and transformations using [PROJ](http://proj4.org) v5.0.x.
#[cfg(feature = "use-proj")]
pub mod proj;
/// Rotate a `Geometry` around either its centroid or a `Point` by an angle given in degrees.
pub mod rotate;
/// Simplify `Geometries` using the Ramer-Douglas-Peucker algorithm.
pub mod simplify;
/// Simplify `Geometries` using the Visvalingam-Whyatt algorithm. Includes a topology-preserving variant.
pub mod simplifyvw;
/// Convert `Geometries` into PostGIS types.
#[cfg(feature = "postgis-integration")]
pub mod to_postgis;
/// Translate a `Geometry` along the given offsets.
pub mod translate;
/// Calculate the Vincenty distance between two `Point`s.
pub mod vincenty_distance;
/// Calculate the Vincenty length of a `LineString`.
pub mod vincenty_length;
/// Calculate and work with the winding order of `Linestring`s.
pub mod winding_order;
