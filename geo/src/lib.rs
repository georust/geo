#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]

//! The `geo` crate provides geospatial primitive types and algorithms.
//!
//! # Types
//!
//! - **[`Coord`]**: A two-dimensional coordinate. All geometry types are composed of [`Coord`]s, though [`Coord`] itself is not a [`Geometry`] type.
//! - **[`Point`]**: A single point represented by one [`Coord`]
//! - **[`MultiPoint`]**: A collection of [`Point`]s
//! - **[`Line`]**: A line segment represented by two [`Coord`]s
//! - **[`LineString`]**: A series of contiguous line segments represented by two or more
//!   [`Coord`]s
//! - **[`MultiLineString`]**: A collection of [`LineString`]s
//! - **[`Polygon`]**: A bounded area represented by one [`LineString`] exterior ring, and zero or
//!   more [`LineString`] interior rings
//! - **[`MultiPolygon`]**: A collection of [`Polygon`]s
//! - **[`Rect`]**: An axis-aligned bounded rectangle represented by minimum and maximum
//!   [`Coord`]s
//! - **[`Triangle`]**: A bounded area represented by three [`Coord`] vertices
//! - **[`GeometryCollection`]**: A collection of [`Geometry`]s
//! - **[`Geometry`]**: An enumeration of all geometry types, excluding [`Coord`]
//!
//! The preceding types are reexported from the [`geo-types`] crate. Consider using that crate
//! if you only need access to these types and no other `geo` functionality.
//!
//! ## Semantics
//!
//! The geospatial types provided here aim to adhere to the [OpenGIS Simple feature access][OGC-SFA]
//! standards. Thus, the types here are inter-operable with other implementations of the standards:
//! [JTS], [GEOS], etc.
//!
//! # Algorithms
//!
//! ## Area
//!
//! - **[`Area`](Area)**: Calculate the planar area of a geometry
//! - **[`ChamberlainDuquetteArea`](ChamberlainDuquetteArea)**: Calculate the geodesic area of a geometry on a sphere using the algorithm presented in _Some Algorithms for Polygons on a Sphere_ by Chamberlain and Duquette (2007)
//! - **[`GeodesicArea`](GeodesicArea)**: Calculate the geodesic area and perimeter of a geometry on an ellipsoid using the algorithm presented in _Algorithms for geodesics_ by Charles Karney (2013)
//!
//! ## Boolean Operations
//!
//! - **[`BooleanOps`](BooleanOps)**: combine or split (Multi)Polygons using intersecton, union, xor, or difference operations
//!
//! ## Distance
//!
//! - **[`EuclideanDistance`](EuclideanDistance)**: Calculate the minimum euclidean distance between geometries
//! - **[`GeodesicDistance`](GeodesicDistance)**: Calculate the minimum geodesic distance between geometries using the algorithm presented in _Algorithms for geodesics_ by Charles Karney (2013)
//! - **[`HaversineDistance`](HaversineDistance)**: Calculate the minimum geodesic distance between geometries using the haversine formula
//! - **[`VincentyDistance`](VincentyDistance)**: Calculate the minimum geodesic distance between geometries using Vincenty’s formula
//!
//! ## Length
//!
//! - **[`EuclideanLength`](EuclideanLength)**: Calculate the euclidean length of a geometry
//! - **[`GeodesicLength`](GeodesicLength)**: Calculate the geodesic length of a geometry using the algorithm presented in _Algorithms for geodesics_ by Charles Karney (2013)
//! - **[`HaversineLength`](HaversineLength)**: Calculate the geodesic length of a geometry using the haversine formula
//! - **[`VincentyLength`](VincentyLength)**: Calculate the geodesic length of a geometry using Vincenty’s formula
//!
//! ## Outlier Detection
//!
//! - **[`OutlierDetection`](OutlierDetection)**: Detect outliers in a group of points using [LOF](https://en.wikipedia.org/wiki/Local_outlier_factor)
//!
//! ## Simplification
//!
//! - **[`Simplify`](Simplify)**: Simplify a geometry using the Ramer–Douglas–Peucker algorithm
//! - **[`SimplifyIdx`](SimplifyIdx)**: Calculate a simplified geometry using the Ramer–Douglas–Peucker algorithm, returning coordinate indices
//! - **[`SimplifyVw`](SimplifyVw)**: Simplify a geometry using the Visvalingam-Whyatt algorithm
//! - **[`SimplifyVwPreserve`](SimplifyVwPreserve)**: Simplify a geometry using a topology-preserving variant of the Visvalingam-Whyatt algorithm
//! - **[`SimplifyVwIdx`](SimplifyVwIdx)**: Calculate a simplified geometry using a topology-preserving variant of the Visvalingam-Whyatt algorithm, returning coordinate indices
//!
//! ## Query
//!
//! - **[`HaversineBearing`]**: Calculate the bearing between points using great circle calculations.
//! - **[`GeodesicBearing`](GeodesicBearing)**: Calculate the bearing between points on a [geodesic](https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid)
//! - **[`ClosestPoint`](ClosestPoint)**: Find the point on a geometry
//!   closest to a given point
//! - **[`IsConvex`](IsConvex)**: Calculate the convexity of a
//!   [`LineString`]
//! - **[`LineInterpolatePoint`](LineInterpolatePoint)**:
//!   Generates a point that lies a given fraction along the line
//! - **[`LineLocatePoint`](LineLocatePoint)**: Calculate the
//!   fraction of a line’s total length representing the location of the closest point on the
//!   line to the given point
//!
//! ## Similarity
//!
//! - **[`FrechetDistance`](FrechetDistance)**: Calculate the similarity between [`LineString`]s using the Fréchet distance
//!
//! ## Topology
//!
//! - **[`Contains`](Contains)**: Calculate if a geometry contains another
//!   geometry
//! - **[`CoordinatePosition`](CoordinatePosition)**: Calculate
//!   the position of a coordinate relative to a geometry
//! - **[`HasDimensions`](HasDimensions)**: Determine the dimensions of a geometry
//! - **[`Intersects`](Intersects)**: Calculate if a geometry intersects
//!   another geometry
//! - **[`line_intersection`](line_intersection::line_intersection)**: Calculates the
//!   intersection, if any, between two lines.
//! - **[`Relate`](Relate)**: Topologically relate two geometries based on
//!   [DE-9IM](https://en.wikipedia.org/wiki/DE-9IM) semantics.
//! - **[`Within`]**: Calculate if a geometry lies completely within another geometry.
//!
//! ## Winding
//!
//! - **[`Orient`](Orient)**: Apply a specified winding [`Direction`](orient::Direction) to a [`Polygon`]’s interior and exterior rings
//! - **[`Winding`](Winding)**: Calculate and manipulate the [`WindingOrder`](winding_order::WindingOrder) of a [`LineString`]
//!
//! ## Iteration
//!
//! - **[`CoordsIter`]**: Iterate over the coordinates of a geometry
//! - **[`MapCoords`]**: Map a function over all the coordinates
//!   in a geometry, returning a new geometry
//! - **[`MapCoordsInPlace`]**: Map a function over all the
//!   coordinates in a geometry in-place
//! - **[`LinesIter`]**: Iterate over lines of a geometry
//!
//! ## Boundary
//!
//! - **[`BoundingRect`](BoundingRect)**: Calculate the axis-aligned
//!   bounding rectangle of a geometry
//! - **[`MinimumRotatedRect`](MinimumRotatedRect)**: Calculate the
//!   minimum bounding box of a geometry
//! - **[`ConcaveHull`](ConcaveHull)**: Calculate the concave hull of a
//!   geometry
//! - **[`ConvexHull`](ConvexHull)**: Calculate the convex hull of a
//!   geometry
//! - **[`Extremes`](Extremes)**: Calculate the extreme coordinates and
//!   indices of a geometry
//!
//! ## Affine transformations
//!
//! - **[`Rotate`](Rotate)**: Rotate a geometry around its centroid
//! - **[`Scale`](Scale)**: Scale a geometry up or down by a factor
//! - **[`Skew`](Skew)**: Skew a geometry by shearing angles along the `x` and `y` dimension
//! - **[`Translate`](Translate)**: Translate a geometry along its axis
//! - **[`AffineOps`](AffineOps)**: generalised composable affine operations
//!
//! ## Conversion
//!
//! - **[`Convert`]**: Convert (infalliby) the type of a geometry’s coordinate value
//! - **[`TryConvert`]**: Convert (falliby) the type of a geometry’s coordinate value
//!
//! ## Miscellaneous
//!
//! - **[`Centroid`](Centroid)**: Calculate the centroid of a geometry
//! - **[`GeodesicDestination`](GeodesicDestination)**: Given a start point, bearing, and distance, calculate the destination point on a [geodesic](https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid)
//! - **[`GeodesicIntermediate`](GeodesicIntermediate)**: Calculate intermediate points on a [geodesic](https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid)
//! - **[`HaversineDestination`]**: Given a start point, bearing, and distance, calculate the destination point on a sphere
//! - **[`HaversineIntermediate`](HaversineIntermediate)**: Calculate intermediate points on a sphere
//! - **[`proj`](proj)**: Project geometries with the `proj` crate (requires the `use-proj` feature)
//! - **[`ChaikinSmoothing`](ChaikinSmoothing)**: Smoothen `LineString`, `Polygon`, `MultiLineString` and `MultiPolygon` using Chaikins algorithm.
//! - **[`Densify`](Densify)**: Densify linear geometry components by interpolating points
//! - **[`Transform`](Transform)**: Transform a geometry using Proj.
//! - **[`RemoveRepeatedPoints`](RemoveRepeatedPoints)**: Remove repeated points from a geometry.
//!
//! # Features
//!
//! The following optional [Cargo features] are available:
//!
//! - `proj-network`: Enables [network grid] support for the [`proj` crate]. After enabling this feature, [further configuration][proj crate file download] is required to use the network grid
//! - `use-proj`: Enables coordinate conversion and transformation of `Point` geometries using the [`proj` crate]
//! - `use-serde`: Allows geometry types to be serialized and deserialized with [Serde]
//!
//! # Ecosystem
//!
//! There’s a wide variety of `geo`-compatible crates in the ecosystem that offer functionality not
//! included in the `geo` crate, including:
//!
//! * Reading and writing file formats (e.g. [GeoJSON][geojson crate], [WKT][wkt crate],
//!   [shapefile][shapefile crate])
//! * [Latitude and longitude parsing][latlng crate]
//! * [Label placement][polylabel crate]
//! * [Geocoding][geocoding crate]
//! * [and much more...][georust website]
//!
//! [`geo-types`]: https://crates.io/crates/geo-types
//! [`proj` crate]: https://github.com/georust/proj
//! [geojson crate]: https://crates.io/crates/geojson
//! [wkt crate]: https://crates.io/crates/wkt
//! [shapefile crate]: https://crates.io/crates/shapefile
//! [latlng crate]: https://crates.io/crates/latlon
//! [polylabel crate]: https://crates.io/crates/polylabel
//! [geocoding crate]: https://crates.io/crates/geocoding
//! [georust website]: https://georust.org
//! [Cargo features]: https://doc.rust-lang.org/cargo/reference/features.html
//! [GEOS]: https://trac.osgeo.org/geos
//! [JTS]: https://github.com/locationtech/jts
//! [network grid]: https://proj.org/usage/network.html
//! [OGC-SFA]: https://www.ogc.org/standards/sfa
//! [proj crate file download]: https://docs.rs/proj/*/proj/#grid-file-download
//! [Serde]: https://serde.rs/

extern crate geo_types;
extern crate num_traits;
#[cfg(feature = "use-serde")]
#[macro_use]
extern crate serde;
#[cfg(feature = "use-proj")]
extern crate proj;
extern crate rstar;

pub use crate::algorithm::*;
pub use crate::types::Closest;

pub use geo_types::{coord, line_string, point, polygon, CoordFloat, CoordNum};

pub mod geometry;
pub use geometry::*;

/// This module includes all the functions of geometric calculations
pub mod algorithm;
mod geometry_cow;
mod types;
mod utils;
pub(crate) use geometry_cow::GeometryCow;

#[cfg(test)]
#[macro_use]
extern crate approx;

#[macro_use]
extern crate log;

/// Mean radius of Earth in meters
/// This is the value recommended by the IUGG:
/// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
/// "Derived Geometric Constants: mean radius" (p133)
/// https://link.springer.com/article/10.1007%2Fs001900050278
/// https://sci-hub.se/https://doi.org/10.1007/s001900050278
/// https://en.wikipedia.org/wiki/Earth_radius#Mean_radius
const MEAN_EARTH_RADIUS: f64 = 6371008.8;

// Radius of Earth at the equator in meters (derived from the WGS-84 ellipsoid)
const EQUATORIAL_EARTH_RADIUS: f64 = 6_378_137.0;

// Radius of Earth at the poles in meters (derived from the WGS-84 ellipsoid)
const POLAR_EARTH_RADIUS: f64 = 6_356_752.314_245;

// Flattening of the WGS-84 ellipsoid - https://en.wikipedia.org/wiki/Flattening
const EARTH_FLATTENING: f64 =
    (EQUATORIAL_EARTH_RADIUS - POLAR_EARTH_RADIUS) / EQUATORIAL_EARTH_RADIUS;

/// A prelude which re-exports the traits for manipulating objects in this
/// crate. Typically imported with `use geo::prelude::*`.
pub mod prelude {
    pub use crate::algorithm::*;
}

/// A common numeric trait used for geo algorithms
///
/// Different numeric types have different tradeoffs. `geo` strives to utilize generics to allow
/// users to choose their numeric types. If you are writing a function which you'd like to be
/// generic over all the numeric types supported by geo, you probably want to constrain
/// your function input to `GeoFloat`. For methods which work for integers, and not just floating
/// point, see [`GeoNum`].
///
/// # Examples
///
/// ```
/// use geo::{GeoFloat, MultiPolygon, Polygon, Point};
///
/// // An admittedly silly method implementation, but the signature shows how to use the GeoFloat trait
/// fn farthest_from<'a, T: GeoFloat>(point: &Point<T>, polygons: &'a MultiPolygon<T>) -> Option<&'a Polygon<T>> {
///     polygons.iter().fold(None, |accum, next| {
///         match accum {
///             None => Some(next),
///             Some(farthest) => {
///                 use geo::{euclidean_distance::EuclideanDistance};
///                 if next.euclidean_distance(point) > farthest.euclidean_distance(point) {
///                     Some(next)
///                 } else {
///                     Some(farthest)
///                 }
///             }
///         }
///     })
/// }
/// ```
pub trait GeoFloat:
    GeoNum + num_traits::Float + num_traits::Signed + num_traits::Bounded + float_next_after::NextAfter
{
}
impl<T> GeoFloat for T where
    T: GeoNum
        + num_traits::Float
        + num_traits::Signed
        + num_traits::Bounded
        + float_next_after::NextAfter
{
}

/// A trait for methods which work for both integers **and** floating point
pub trait GeoNum: CoordNum + HasKernel {}
impl<T> GeoNum for T where T: CoordNum + HasKernel {}
