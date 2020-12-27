#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! The `geo` crate provides geospatial primitive types and algorithms.
//!
//! # Types
//!
//! TODO: diagram
//! https://shapely.readthedocs.io/en/stable/manual.html#linestrings
//!
//! - **[`Coordinate`]**: A two-dimensional coordinate all other geometry types are based on
//! - **[`Point`]**: A single point represented by one [`Coordinate`]
//! - **[`MultiPoint`]**: A collection of [`Point`]s
//! - **[`Line`]**: A line segment represented by two [`Coordinate`]s
//! - **[`LineString`]**: A series of contiguous line segments represented by two or more [`Coordinate`]s
//! - **[`MultiLineString`]**: A collection of [`LineString`]s
//! - **[`Polygon`]**: A bounded area represented by one [`LineString`] exeterior ring, and zero or more [`LineString`] interior rings
//! - **[`MultiPolygon`]**: A collection of [`Polygon`]s
//! - **[`Rect`]**: An axis-aligned bounded rectangle represented by minimum and maximum [`Coordinate`]s
//! - **[`Triangle`]**: A bounded area represented by three [`Coordinate`] vertices
//! - **[`GeometryCollection`]**: A collection of [`Geometry`]s
//! - **[`Geometry`]**: An enumeration of all geometry types, excluding [`Coordinate`]
//!
//! # Algorithms
//!
//! ## Area
//!
//! - **[`Area`]**: Planar area of a geometry
//! - **[`ChamberlainDuquetteArea`]**: Geodesic area of a geometry
//!
//! ## Distance
//!
//! - **[`EuclideanDistance`]**: Euclidean distance between geometries
//! - **[`GeodesicDistance`]**:
//! - **[`HaversineDistance`]**:
//! - **[`VincentyDistance`]**:
//!
//! ## Length
//!
//! - **[`EuclideanLength`]**:
//! - **[`GeodesicLength`]**:
//! - **[`HaversineLength`]**:
//! - **[`VincentyLength`]**:
//!
//! ## Simplify
//!
//! - **[`Simplify`]**:
//! - **[`SimplifyIdx`]**:
//! - **[`SimplifyVW`]**:
//! - **[`SimplifyVWPreserve`]**:
//! - **[`SimplifyVwIdx`]**:
//!
//! ## Query
//!
//! - **[`Bearing`]**:
//! - **[`ClosestPoint`]**:
//! - **[`Contains`]**:
//! - **[`CoordinatePosition`]**:
//! - **[`Intersects`]**:
//! - **[`IsConvex`]**:
//! - **[`LineInterpolatePoint`]**:
//! - **[`LineLocatePoint`]**:
//!
//! ## Similarity
//!
//! - **[`FrechetDistance`]**:
//!
//! ## Winding
//!
//! - **[`Orient`]**:
//! - **[`WindingOrder`]**
//!
//! ## Map
//!
//! - **[`MapCoords`]**:
//! - **[`MapCoordsInplace`]**:
//! - **[`TryMapCoords`]**:
//!
//! ## Boundary
//!
//! - **[`BoundingRect`]**:
//! - **[`ConcaveHull`]**:
//! - **[`ConvexHull`]**:
//! - **[`ExtremePoints`]**:
//! - **[`ExtremeIndices`]**:
//!
//! ## Affine transformations
//!
//! - **[`Rotate`]**
//! - **[`RotatePoint`]**:
//! - **[`Translate`]**:
//!
//! ## Unsorted
//!
//! - **[`Centroid`]**:
//! - **[`CoordsIter`]**:
//! - **[`HasDimensions`]**:
//! - **[`HaversineDestination`]**:
//! - **[`HaversineIntermediate`]**:
//! - **[`Proj`]**:
//!
//! --------
//!
//!
//!
//!
//! The primitive types also provide the basis for other functionality in the `Geo` ecosystem, including:
//!   - Serialization to and from [GeoJSON](https://docs.rs/geojson) and [WKT](https://docs.rs/wkt)
//!   - [Coordinate transformation and projection](https://docs.rs/proj)
//!   - [Geocoding](https://docs.rs/geocoding)
//!   - [Working with GPS data](https://docs.rs/gpx)
//!
//! …allowing these crates to interoperate; GeoJSON can readily be read from a file, deserialised, transformed
//! to a local datum, modified, transformed back to `WGS84`, and serialised back to GeoJSON.
//!
//! Operations available for primitive types can be found in the `algorithm` module, along with
//! comprehensive usage examples.
//!
//! While `Geo` is primarily intended to operate on **planar** geometries, some other useful algorithms are
//! provided: Haversine, Frechet, and Vincenty distances, as well as Chamberlain-Duquette area.
//!
//! ## Optional Features (these can be activated in your `cargo.toml`)
//! The following optional features are available:
//! - `use-proj`: enable coordinate conversion and transformation of `Point` geometries using the [`proj`](https://docs.rs/proj) crate
//! - `proj-network`: enables functionality for `proj` crate's network grid. After enabling
//! this feature, some [further
//! configuration](https://docs.rs/proj/0.20.5/proj/#grid-file-download) is
//! required to actually use the network grid.
//! - `use-serde`: enable serialisation of geometries using `serde`.
//!
//! ## GeoJSON
//! If you wish to read or write `GeoJSON`, use the [`geojson`](https://docs.rs/geojson) crate, with the `geo-types` feature activated.
//! This provides fallible conversions **to** `geo-types` primitives such as `Point` and `Polygon` from `geojson` `Value`
//! structs using the standard [`TryFrom`](https://doc.rust-lang.org/stable/std/convert/trait.TryFrom.html)
//! and [`TryInto`](https://doc.rust-lang.org/stable/std/convert/trait.TryInto.html) traits,
//! and conversion **from** `geo-types` primitives to `geojson`
//! `Value` structs using the [`From`](https://doc.rust-lang.org/stable/std/convert/trait.TryFrom.html) trait.

extern crate geo_types;
extern crate num_traits;
#[cfg(feature = "use-serde")]
#[macro_use]
extern crate serde;
extern crate rstar;

#[allow(deprecated)]
pub use crate::algorithm::*;
#[allow(deprecated)]
pub use crate::traits::ToGeo;
pub use crate::types::*;

pub use geo_types::{
    line_string, point, polygon, Coordinate, CoordinateType, Geometry, GeometryCollection, Line,
    LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// Kernels to compute various predicates
pub(crate) mod kernels;

/// Calculate the area of the surface of a `Geometry`.
pub mod area;
/// Calculate the bearing to another `Point`, in degrees.
pub mod bearing;
/// Calculate the bounding rectangle of a `Geometry`.
pub mod bounding_rect;
/// Calculate the centroid of a `Geometry`.
pub mod centroid;
/// Calculate the signed approximate geodesic area of a `Geometry`.
pub mod chamberlain_duquette_area;
/// Calculate the closest `Point` between a `Geometry` and an input `Point`.
pub mod closest_point;
/// Calculate the concave hull of a `Geometry`.
pub mod concave_hull;
/// Determine whether `Geometry` `A` is completely enclosed by `Geometry` `B`.
pub mod contains;
/// Calculate the convex hull of a `Geometry`.
pub mod convex_hull;
/// Determine whether a `Coordinate` lies inside, outside, or on the boundary of a geometry.
pub mod coordinate_position;
/// Iterate over geometry coordinates.
pub mod coords_iter;
/// Dimensionality of a geometry and its boundary, based on OGC-SFA.
pub mod dimensions;
/// Calculate the minimum Euclidean distance between two `Geometries`.
pub mod euclidean_distance;
/// Calculate the length of a planar line between two `Geometries`.
pub mod euclidean_length;
/// Calculate the extreme indices of a `Polygon`, `MultiPolygon`, or `MultiPoint`.
pub mod extremes;
/// Calculate the Frechet distance between two `LineStrings`.
pub mod frechet_distance;
/// Calculate the Geodesic distance between two `Point`s.
pub mod geodesic_distance;
/// Calculate the Geodesic length of a line.
pub mod geodesic_length;
/// Calculate a destination `Point`, given a distance and a bearing.
pub mod haversine_destination;
/// Calculate the Haversine distance between two `Geometries`.
pub mod haversine_distance;
/// Calculate a new `Point` lying on a Great Circle arc between two `Point`s.
pub mod haversine_intermediate;
/// Calculate the Haversine length of a Line.
pub mod haversine_length;
/// Determine whether `Geometry` `A` intersects `Geometry` `B`.
pub mod intersects;
/// Determins whether a `LineString` is convex.
pub mod is_convex;
/// Interpolate a point along a `Line` or `LineString`.
pub mod line_interpolate_point;
/// Locate a point along a `Line` or `LineString`.
pub mod line_locate_point;
/// Apply a function to all `Coordinates` of a `Geometry`.
pub mod map_coords;
/// Orient a `Polygon`'s exterior and interior rings.
pub mod orient;
/// Helper functions for the "fast path" variant of the Polygon-Polygon Euclidean distance method.
pub(crate) mod polygon_distance_fast_path;
/// Coordinate projections and transformations using the current stable version of [PROJ](http://proj.org).
#[cfg(feature = "use-proj")]
pub mod proj;
/// Rotate a `Geometry` around either its centroid or a `Point` by an angle given in degrees.
pub mod rotate;
/// Simplify `Geometries` using the Ramer-Douglas-Peucker algorithm.
pub mod simplify;
/// Simplify `Geometries` using the Visvalingam-Whyatt algorithm. Includes a topology-preserving variant.
pub mod simplifyvw;
/// Translate a `Geometry` along the given offsets.
pub mod translate;
/// Calculate the Vincenty distance between two `Point`s.
pub mod vincenty_distance;
/// Calculate the Vincenty length of a `LineString`.
pub mod vincenty_length;
/// Calculate and work with the winding order of `Linestring`s.
pub mod winding_order;

#[deprecated(note = "TODO")]
/// This module includes all the functions of geometric calculations
pub mod algorithm;

pub use area::Area;
pub use bearing::Bearing;
pub use bounding_rect::BoundingRect;
pub use centroid::Centroid;
pub use chamberlain_duquette_area::ChamberlainDuquetteArea;
pub use closest_point::ClosestPoint;
pub use concave_hull::ConcaveHull;
pub use contains::Contains;
pub use convex_hull::ConvexHull;
pub use coordinate_position::CoordinatePosition;
pub use coords_iter::CoordsIter;
pub use dimensions::HasDimensions;
pub use euclidean_distance::EuclideanDistance;
pub use euclidean_length::EuclideanLength;
pub use extremes::{ExtremeIndices, ExtremePoints};
pub use frechet_distance::FrechetDistance;
pub use geodesic_distance::GeodesicDistance;
pub use geodesic_length::GeodesicLength;
pub use haversine_destination::HaversineDestination;
pub use haversine_distance::HaversineDistance;
pub use haversine_intermediate::HaversineIntermediate;
pub use haversine_length::HaversineLength;
pub use intersects::Intersects;
pub use is_convex::IsConvex;
pub use line_interpolate_point::LineInterpolatePoint;
pub use line_locate_point::LineLocatePoint;
pub use map_coords::{MapCoords, MapCoordsInplace, TryMapCoords};
pub use orient::Orient;
#[cfg(feature = "use-proj")]
pub use crate::proj::Proj;
pub use rotate::{Rotate, RotatePoint};
pub use simplify::{Simplify, SimplifyIdx};
pub use simplifyvw::{SimplifyVW, SimplifyVWPreserve, SimplifyVwIdx};
pub use translate::Translate;
pub use vincenty_distance::VincentyDistance;
pub use vincenty_length::VincentyLength;
pub use winding_order::WindingOrder;

mod traits;
mod types;
mod utils;

#[cfg(test)]
#[macro_use]
extern crate approx;

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
    pub use crate::area::Area;
    pub use crate::bearing::Bearing;
    pub use crate::bounding_rect::BoundingRect;
    pub use crate::centroid::Centroid;
    pub use crate::chamberlain_duquette_area::ChamberlainDuquetteArea;
    pub use crate::closest_point::ClosestPoint;
    pub use crate::contains::Contains;
    pub use crate::convex_hull::ConvexHull;
    pub use crate::dimensions::HasDimensions;
    pub use crate::euclidean_distance::EuclideanDistance;
    pub use crate::euclidean_length::EuclideanLength;
    pub use crate::extremes::ExtremePoints;
    pub use crate::frechet_distance::FrechetDistance;
    pub use crate::geodesic_distance::GeodesicDistance;
    pub use crate::geodesic_length::GeodesicLength;
    pub use crate::haversine_destination::HaversineDestination;
    pub use crate::haversine_distance::HaversineDistance;
    pub use crate::haversine_intermediate::HaversineIntermediate;
    pub use crate::haversine_length::HaversineLength;
    pub use crate::intersects::Intersects;
    pub use crate::is_convex::IsConvex;
    pub use crate::map_coords::MapCoords;
    pub use crate::orient::Orient;
    #[cfg(feature = "use-proj")]
    pub use crate::proj::Proj;
    pub use crate::rotate::{Rotate, RotatePoint};
    pub use crate::simplify::Simplify;
    pub use crate::simplifyvw::SimplifyVW;
    pub use crate::translate::Translate;
    pub use crate::vincenty_distance::VincentyDistance;
    pub use crate::vincenty_length::VincentyLength;
}
