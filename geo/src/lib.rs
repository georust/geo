#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]

//! The `geo` crate provides geospatial primitive types and algorithms.
//!
//! # Types
//!
//! - **[`Coordinate`]**: A two-dimensional coordinate. All geometry types are composed of [`Coordinate`]s, though [`Coordinate`] itself is not a [`Geometry`] type.
//! - **[`Point`]**: A single point represented by one [`Coordinate`]
//! - **[`MultiPoint`]**: A collection of [`Point`]s
//! - **[`Line`]**: A line segment represented by two [`Coordinate`]s
//! - **[`LineString`]**: A series of contiguous line segments represented by two or more
//!   [`Coordinate`]s
//! - **[`MultiLineString`]**: A collection of [`LineString`]s
//! - **[`Polygon`]**: A bounded area represented by one [`LineString`] exterior ring, and zero or
//!   more [`LineString`] interior rings
//! - **[`MultiPolygon`]**: A collection of [`Polygon`]s
//! - **[`Rect`]**: An axis-aligned bounded rectangle represented by minimum and maximum
//!   [`Coordinate`]s
//! - **[`Triangle`]**: A bounded area represented by three [`Coordinate`] vertices
//! - **[`GeometryCollection`]**: A collection of [`Geometry`]s
//! - **[`Geometry`]**: An enumeration of all geometry types, excluding [`Coordinate`]
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
//! - **[`Area`](algorithm::area::Area)**: Calculate the planar area of a geometry
//! - **[`ChamberlainDuquetteArea`](algorithm::chamberlain_duquette_area::ChamberlainDuquetteArea)**: Calculate the geodesic area of a geometry
//!
//! ## Distance
//!
//! - **[`EuclideanDistance`](algorithm::euclidean_distance::EuclideanDistance)**: Calculate the minimum euclidean distance between geometries
//! - **[`GeodesicDistance`](algorithm::geodesic_distance::GeodesicDistance)**: Calculate the minimum geodesic distance between geometries using the algorithm presented in _Algorithms for geodesics_ by Charles Karney (2013)
//! - **[`HaversineDistance`](algorithm::haversine_distance::HaversineDistance)**: Calculate the minimum geodesic distance between geometries using the haversine formula
//! - **[`VincentyDistance`](algorithm::vincenty_distance::VincentyDistance)**: Calculate the minimum geodesic distance between geometries using Vincenty’s formula
//!
//! ## Length
//!
//! - **[`EuclideanLength`](algorithm::euclidean_length::EuclideanLength)**: Calculate the euclidean length of a geometry
//! - **[`GeodesicLength`](algorithm::geodesic_length::GeodesicLength)**: Calculate the geodesic length of a geometry using the algorithm presented in _Algorithms for geodesics_ by Charles Karney (2013)
//! - **[`HaversineLength`](algorithm::haversine_length::HaversineLength)**: Calculate the geodesic length of a geometry using the haversine formula
//! - **[`VincentyLength`](algorithm::vincenty_length::VincentyLength)**: Calculate the geodesic length of a geometry using Vincenty’s formula
//!
//! ## Simplification
//!
//! - **[`Simplify`](algorithm::simplify::Simplify)**: Simplify a geometry using the Ramer–Douglas–Peucker algorithm
//! - **[`SimplifyIdx`](algorithm::simplify::SimplifyIdx)**: Calculate a simplified geometry using the Ramer–Douglas–Peucker algorithm, returning coordinate indices
//! - **[`SimplifyVW`](algorithm::simplifyvw::SimplifyVW)**: Simplify a geometry using the Visvalingam-Whyatt algorithm
//! - **[`SimplifyVWPreserve`](algorithm::simplifyvw::SimplifyVWPreserve)**: Simplify a geometry using a topology-preserving variant of the Visvalingam-Whyatt algorithm
//! - **[`SimplifyVwIdx`](algorithm::simplifyvw::SimplifyVwIdx)**: Calculate a simplified geometry using a topology-preserving variant of the Visvalingam-Whyatt algorithm, returning coordinate indices
//!
//! ## Query
//!
//! - **[`Bearing`](algorithm::bearing::Bearing)**: Calculate the bearing between points
//! - **[`ClosestPoint`](algorithm::closest_point::ClosestPoint)**: Find the point on a geometry
//!   closest to a given point
//! - **[`IsConvex`](algorithm::is_convex::IsConvex)**: Calculate the convexity of a
//!   [`LineString`]
//! - **[`LineInterpolatePoint`](algorithm::line_interpolate_point::LineInterpolatePoint)**:
//!   Generates a point that lies a given fraction along the line
//! - **[`LineLocatePoint`](algorithm::line_locate_point::LineLocatePoint)**: Calculate the
//!   fraction of a line’s total length representing the location of the closest point on the
//!   line to the given point
//!
//! ## Similarity
//!
//! - **[`FrechetDistance`](algorithm::frechet_distance::FrechetDistance)**: Calculate the similarity between [`LineString`]s using the Fréchet distance
//!
//! ## Topology
//!
//! - **[`Contains`](algorithm::contains::Contains)**: Calculate if a geometry contains another
//!   geometry
//! - **[`CoordinatePosition`](algorithm::coordinate_position::CoordinatePosition)**: Calculate
//!   the position of a coordinate relative to a geometry
//! - **[`HasDimensions`](algorithm::dimensions::HasDimensions)**: Determine the dimensions of a geometry
//! - **[`Intersects`](algorithm::intersects::Intersects)**: Calculate if a geometry intersects
//!   another geometry
//! - **[`line_intersection`](algorithm::line_intersection::line_intersection)**: Calculates the
//!   intersection, if any, between two lines.
//! - **[`Relate`](algorithm::relate::Relate)**: Topologically relate two geometries based on
//!   [DE-9IM](https://en.wikipedia.org/wiki/DE-9IM) semantics.
//!
//! ## Winding
//!
//! - **[`Orient`](algorithm::orient::Orient)**: Apply a specified [`Winding`](algorithm::winding_order::Winding) to a [`Polygon`]’s interior and exterior rings
//! - **[`Winding`](algorithm::winding_order::Winding)**: Calculate and manipulate the winding order of a [`LineString`]
//!
//! ## Iteration
//!
//! - **[`CoordsIter`](algorithm::coords_iter::CoordsIter)**: Iterate over the coordinates of a geometry
//! - **[`MapCoords`](algorithm::map_coords::MapCoords)**: Map a function over all the coordinates
//!   in a geometry, returning a new geometry
//! - **[`MapCoordsInplace`](algorithm::map_coords::MapCoordsInplace)**: Map a function over all the
//!   coordinates in a geometry in-place
//! - **[`TryMapCoords`](algorithm::map_coords::TryMapCoords)**: Map a fallible function over all
//!   the coordinates in a geometry, returning a new geometry wrapped in a `Result`
//!
//! ## Boundary
//!
//! - **[`BoundingRect`](algorithm::bounding_rect::BoundingRect)**: Calculate the axis-aligned
//!   bounding rectangle of a geometry
//! - **[`ConcaveHull`](algorithm::concave_hull::ConcaveHull)**: Calculate the concave hull of a
//!   geometry
//! - **[`ConvexHull`](algorithm::convex_hull::ConvexHull)**: Calculate the convex hull of a
//!   geometry
//! - **[`Extremes`](algorithm::extremes::Extremes)**: Calculate the extreme coordinates and
//!   indices of a geometry
//!
//! ## Affine transformations
//!
//! - **[`Rotate`](algorithm::rotate::Rotate)**: Rotate a geometry around its centroid
//! - **[`RotatePoint`](algorithm::rotate::RotatePoint)**: Rotate a geometry around a point
//! - **[`Translate`](algorithm::translate::Translate)**: Translate a geometry along its axis
//!
//! ## Miscellaneous
//!
//! - **[`Centroid`](algorithm::centroid::Centroid)**: Calculate the centroid of a geometry
//! - **[`HaversineDestination`](algorithm::haversine_destination::HaversineDestination)**:
//! - **[`HaversineIntermediate`](algorithm::haversine_intermediate::HaversineIntermediate)**:
//! - **`Proj`**: Project geometries with the `proj` crate
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
#[allow(deprecated)]
pub use crate::traits::ToGeo;
pub use crate::types::Closest;

pub use geo_types::{
    line_string, point, polygon, CoordFloat, CoordNum, Coordinate, Geometry, GeometryCollection,
    Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// This module includes all the functions of geometric calculations
pub mod algorithm;
mod geometry_cow;
mod traits;
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
    pub use crate::algorithm::area::Area;
    pub use crate::algorithm::bearing::Bearing;
    pub use crate::algorithm::bounding_rect::BoundingRect;
    pub use crate::algorithm::centroid::Centroid;
    pub use crate::algorithm::chamberlain_duquette_area::ChamberlainDuquetteArea;
    pub use crate::algorithm::closest_point::ClosestPoint;
    pub use crate::algorithm::contains::Contains;
    pub use crate::algorithm::convex_hull::ConvexHull;
    pub use crate::algorithm::dimensions::HasDimensions;
    pub use crate::algorithm::euclidean_distance::EuclideanDistance;
    pub use crate::algorithm::euclidean_length::EuclideanLength;
    pub use crate::algorithm::extremes::Extremes;
    pub use crate::algorithm::frechet_distance::FrechetDistance;
    pub use crate::algorithm::geodesic_distance::GeodesicDistance;
    pub use crate::algorithm::geodesic_intermediate::GeodesicIntermediate;
    pub use crate::algorithm::geodesic_length::GeodesicLength;
    pub use crate::algorithm::haversine_destination::HaversineDestination;
    pub use crate::algorithm::haversine_distance::HaversineDistance;
    pub use crate::algorithm::haversine_intermediate::HaversineIntermediate;
    pub use crate::algorithm::haversine_length::HaversineLength;
    pub use crate::algorithm::intersects::Intersects;
    pub use crate::algorithm::is_convex::IsConvex;
    pub use crate::algorithm::map_coords::MapCoords;
    pub use crate::algorithm::orient::Orient;
    #[cfg(feature = "use-proj")]
    pub use crate::algorithm::proj::Proj;
    pub use crate::algorithm::rotate::{Rotate, RotatePoint};
    pub use crate::algorithm::simplify::Simplify;
    pub use crate::algorithm::simplifyvw::SimplifyVW;
    pub use crate::algorithm::translate::Translate;
    pub use crate::algorithm::vincenty_distance::VincentyDistance;
    pub use crate::algorithm::vincenty_length::VincentyLength;
}

/// A common numeric trait used for geo algorithms.
///
/// Different numeric types have different tradeoffs. `geo` strives to utilize generics to allow
/// users to choose their numeric types. If you are writing a function which you'd like to be
/// generic over all the numeric types supported by geo, you probably want to constraint
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
///                 use geo::algorithm::{euclidean_distance::EuclideanDistance};
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
pub trait GeoFloat: num_traits::Float + GeoNum {}
impl<T> GeoFloat for T where T: num_traits::Float + GeoNum {}

pub trait GeoNum: CoordNum + algorithm::kernels::HasKernel {}
impl<T> GeoNum for T where T: CoordNum + algorithm::kernels::HasKernel {}
