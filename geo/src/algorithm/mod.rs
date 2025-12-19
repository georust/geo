/// Kernels to compute various predicates
pub mod kernels;
pub use kernels::{Kernel, Orientation};

/// Calculate the area of the surface of a `Geometry`.
pub mod area;
pub use area::Area;

/// Boolean Operations such as the union, xor, or difference of two geometries.
pub mod bool_ops;
pub use bool_ops::{BooleanOps, OpType, unary_union};

/// Calculate the bounding rectangle of a `Geometry`.
pub mod bounding_rect;
pub use bounding_rect::BoundingRect;

pub mod buffer;
pub use buffer::Buffer;

/// Calculate the minimum rotated rectangle of a `Geometry`.
pub mod minimum_rotated_rect;
pub use minimum_rotated_rect::MinimumRotatedRect;

/// Calculate the centroid of a `Geometry`.
pub mod centroid;
pub use centroid::Centroid;

/// Smoothen `LineString`, `Polygon`, `MultiLineString` and `MultiPolygon` using Chaikins algorithm.
pub mod chaikin_smoothing;
pub use chaikin_smoothing::ChaikinSmoothing;

/// Calculate the signed approximate geodesic area of a `Geometry`.
pub mod chamberlain_duquette_area;
pub use chamberlain_duquette_area::ChamberlainDuquetteArea;

/// Calculate the closest `Point` between a `Geometry` and an input `Point`.
pub mod closest_point;
pub use closest_point::ClosestPoint;

/// Calculate the concave hull of a `Geometry`.
pub mod concave_hull;
pub use concave_hull::ConcaveHull;

/// Determine whether `Geometry` `A` completely encloses `Geometry` `B`.
pub mod contains;
pub use contains::Contains;

pub mod contains_properly;
pub use contains_properly::ContainsProperly;

pub mod covers;
pub use covers::Covers;

/// Convert the type of a geometry’s coordinate value.
pub mod convert;
pub use convert::{Convert, TryConvert};

/// Convert coordinate angle units between radians and degrees.
pub mod convert_angle_unit;
pub use convert_angle_unit::{ToDegrees, ToRadians};

/// Calculate the convex hull of a `Geometry`.
pub mod convex_hull;
pub use convex_hull::ConvexHull;

/// Cross track distance
pub mod cross_track_distance;
pub use cross_track_distance::CrossTrackDistance;

/// Determine whether a `Coord` lies inside, outside, or on the boundary of a geometry.
pub mod coordinate_position;
pub use coordinate_position::CoordinatePosition;

/// Iterate over geometry coordinates.
pub mod coords_iter;
pub use coords_iter::CoordsIter;

/// Densify spherical geometry components
pub mod densify_haversine;
#[allow(deprecated)]
pub use densify_haversine::DensifyHaversine;

/// Dimensionality of a geometry and its boundary, based on OGC-SFA.
pub mod dimensions;
pub use dimensions::HasDimensions;

/// Calculate the minimum Euclidean distance between two `Geometries`.
pub mod euclidean_distance;
#[allow(deprecated)]
pub use euclidean_distance::EuclideanDistance;

/// Calculate the length of a planar line between two `Geometries`.
pub mod euclidean_length;
#[allow(deprecated)]
pub use euclidean_length::EuclideanLength;

/// Calculate the extreme coordinates and indices of a geometry.
pub mod extremes;
pub use extremes::Extremes;

/// Calculate the Frechet distance between two `LineStrings`.
pub mod frechet_distance;
#[allow(deprecated)]
pub use frechet_distance::FrechetDistance;

/// Calculate the bearing to another `Point` on a geodesic.
pub mod geodesic_bearing;
pub use geodesic_bearing::GeodesicBearing;

/// Returns a new Point using a distance and bearing on a geodesic.
pub mod geodesic_destination;
#[allow(deprecated)]
pub use geodesic_destination::GeodesicDestination;

/// Calculate the Geodesic distance between two `Point`s.
pub mod geodesic_distance;
#[allow(deprecated)]
pub use geodesic_distance::GeodesicDistance;

/// Calculate the Geodesic area and perimeter of polygons.
pub mod geodesic_area;
pub use geodesic_area::GeodesicArea;

/// Calculate a new `Point` lying on a Geodesic arc between two `Point`s.
pub mod geodesic_intermediate;
#[allow(deprecated)]
pub use geodesic_intermediate::GeodesicIntermediate;

/// Calculate the Geodesic length of a line.
pub mod geodesic_length;
#[allow(deprecated)]
pub use geodesic_length::GeodesicLength;

/// Calculate the Hausdorff distance between two geometries.
pub mod hausdorff_distance;
pub use hausdorff_distance::HausdorffDistance;

/// Calculate the bearing to another `Point`, in degrees.
pub mod haversine_bearing;
#[allow(deprecated)]
pub use haversine_bearing::HaversineBearing;

/// Calculate a destination `Point`, given a distance and a bearing.
pub mod haversine_destination;
#[allow(deprecated)]
pub use haversine_destination::HaversineDestination;

/// Calculate the Haversine distance between two `Geometries`.
pub mod haversine_distance;
#[allow(deprecated)]
pub use haversine_distance::HaversineDistance;

/// Calculate a new `Point` lying on a Great Circle arc between two `Point`s.
pub mod haversine_intermediate;
#[allow(deprecated)]
pub use haversine_intermediate::HaversineIntermediate;

/// Calculate the Haversine length of a Line.
pub mod haversine_length;
#[allow(deprecated)]
pub use haversine_length::HaversineLength;

/// Calculate the closest point on a Great Circle arc geometry to a given point.
pub mod haversine_closest_point;
pub use haversine_closest_point::HaversineClosestPoint;

/// Geometries backed by spatial indexes for fast bulk operations
pub mod indexed;
pub use indexed::PreparedGeometry;

/// Calculate a representative `Point` inside a `Geometry`
pub mod interior_point;
pub use interior_point::InteriorPoint;

/// Determine whether `Geometry` `A` intersects `Geometry` `B`.
pub mod intersects;
pub use intersects::Intersects;

/// Determines whether a `LineString` is convex.
pub mod is_convex;
pub use is_convex::IsConvex;

/// Calculate concave hull using k-nearest algorithm
pub mod k_nearest_concave_hull;
pub use k_nearest_concave_hull::KNearestConcaveHull;

/// Interpolate a point along a `Line` or `LineString`.
pub mod line_interpolate_point;
#[allow(deprecated)]
pub use line_interpolate_point::LineInterpolatePoint;

/// Computes the intersection of two Lines.
pub mod line_intersection;
pub use line_intersection::LineIntersection;

/// Locate a point along a `Line` or `LineString`.
pub mod line_locate_point;
pub use line_locate_point::LineLocatePoint;

/// Iterate over the lines in a geometry.
pub mod lines_iter;
pub use lines_iter::LinesIter;

pub mod line_measures;
pub use line_measures::metric_spaces::{
    Euclidean, Geodesic, GeodesicMeasure, Haversine, HaversineMeasure, Rhumb,
};
pub use line_measures::{
    Bearing, Densify, Destination, Distance, InterpolatableLine, InterpolateLine, InterpolatePoint,
    Length,
};

/// Split a LineString into n segments
pub mod linestring_segment;
pub use linestring_segment::{LineStringSegmentize, LineStringSegmentizeHaversine};

/// Apply a function to all `Coord`s of a `Geometry`.
pub mod map_coords;
pub use map_coords::{MapCoords, MapCoordsInPlace};

/// Orient a `Polygon`'s exterior and interior rings.
pub mod orient;
pub use orient::Orient;

/// Coordinate projections and transformations using the current stable version of [PROJ](http://proj.org).
#[cfg(feature = "use-proj")]
pub mod proj;

/// Relate two geometries based on DE-9IM
pub mod relate;
pub use relate::Relate;

/// Remove (consecutive) repeated points
pub mod remove_repeated_points;
pub use remove_repeated_points::RemoveRepeatedPoints;

/// Rotate a `Geometry` by an angle given in degrees.
pub mod rotate;
pub use rotate::Rotate;

/// Scale a `Geometry` up or down by a factor
pub mod scale;
pub use scale::Scale;

/// Skew a `Geometry` by shearing it at angles along the x and y dimensions
pub mod skew;
pub use skew::Skew;

/// Composable affine operations such as rotate, scale, skew, and translate
pub mod affine_ops;
pub use affine_ops::{AffineOps, AffineTransform};

/// Simplify `Geometries` using the Ramer-Douglas-Peucker algorithm.
pub mod simplify;
pub use simplify::{Simplify, SimplifyIdx};

/// Simplify `Geometries` using the Visvalingam-Whyatt algorithm. Includes a topology-preserving variant.
pub mod simplify_vw;
pub use simplify_vw::{SimplifyVw, SimplifyVwIdx, SimplifyVwPreserve};

/// Stitch together triangles with adjacent sides. Alternative to unioning triangles via BooleanOps.
#[allow(dead_code)]
pub(crate) mod stitch;
pub use stitch::StitchTriangles;

/// Transform a geometry using PROJ.
#[cfg(feature = "use-proj")]
pub mod transform;
#[cfg(feature = "use-proj")]
pub use transform::Transform;

/// Translate a `Geometry` along the given offsets.
pub mod translate;
pub use translate::Translate;

/// Triangulate polygons using an [ear-cutting algorithm](https://www.geometrictools.com/Documentation/TriangulationByEarClipping.pdf).
///
/// Requires the `"earcutr"` feature.
#[cfg(feature = "earcutr")]
pub mod triangulate_earcut;
#[cfg(feature = "earcutr")]
pub use triangulate_earcut::TriangulateEarcut;

/// Triangulate polygons using an (un)constrained [Delaunay Triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation) algorithm.
#[cfg(feature = "spade")]
pub mod triangulate_delaunay;
#[cfg(feature = "spade")]
pub use triangulate_delaunay::TriangulateDelaunay;

/// Triangulate polygons using an (un)constrained [Delaunay Triangulation](https://en.wikipedia.org/wiki/Delaunay_triangulation) algorithm.
#[cfg(feature = "spade")]
#[deprecated(
    since = "0.29.4",
    note = "please use the `triangulate_delaunay` module instead"
)]
pub mod triangulate_spade;
#[cfg(feature = "spade")]
#[deprecated(since = "0.29.4", note = "please use `TriangulateDelaunay` instead")]
#[allow(deprecated)]
pub use triangulate_spade::TriangulateSpade;

/// Vector Operations for 2D coordinates
mod vector_ops;
pub use vector_ops::Vector2DOps;

/// Calculate the Vincenty distance between two `Point`s.
pub mod vincenty_distance;
pub use vincenty_distance::VincentyDistance;

/// Calculate the Vincenty length of a `LineString`.
pub mod vincenty_length;
pub use vincenty_length::VincentyLength;

/// Calculate and work with the winding order of `Linestring`s.
pub mod winding_order;
pub use winding_order::Winding;

/// Determine whether `Geometry` `A` is completely within by `Geometry` `B`.
pub mod within;
pub use within::Within;

/// Planar sweep algorithm and related utils for use by the monotone module
pub(crate) mod old_sweep;

/// Find line intersections using the Bentley-Ottmann algorithm
pub mod sweep;

/// Detect outliers in a group of points using [LOF](https://en.wikipedia.org/wiki/Local_outlier_factor)
pub mod outlier_detection;

pub use outlier_detection::OutlierDetection;

/// Cluster points using [DBSCAN](https://en.wikipedia.org/wiki/DBSCAN) (Density-Based Spatial Clustering of Applications with Noise)
pub mod dbscan;
pub use dbscan::Dbscan;

/// Cluster points using [k-means clustering](https://en.wikipedia.org/wiki/K-means_clustering)
#[cfg(feature = "rand")]
pub mod kmeans;
#[cfg(feature = "rand")]
pub use kmeans::{KMeans, KMeansError, KMeansParams};

/// Monotonic polygon subdivision
pub mod monotone;
pub use monotone::{MonoPoly, MonotonicPolygons, monotone_subdivision};

/// Rhumb-line-related algorithms and utils
pub mod rhumb;
#[allow(deprecated)]
pub use rhumb::{RhumbBearing, RhumbDestination, RhumbDistance, RhumbIntermediate, RhumbLength};

pub mod validation;
pub use validation::Validation;
