#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
//! The `geo` crate provides geospatial primitive types such as `Coordinate`, `Point`, `LineString`, and `Polygon` as
//! well as their `Multi–` equivalents, and provides algorithms and operations such as:
//!   - Area and centroid calculation
//!   - Simplification and convex hull operations
//!   - Distance measurement
//!   - Intersection checks
//!   - Affine transforms such as rotation and translation.
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
#[cfg(feature = "use-proj")]
extern crate proj;
extern crate rstar;

pub use crate::algorithm::*;
#[allow(deprecated)]
pub use crate::traits::ToGeo;
pub use crate::types::*;

pub use geo_types::{
    line_string, point, polygon, Coordinate, CoordinateType, Geometry, GeometryCollection, Line,
    LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// This module includes all the functions of geometric calculations
pub mod algorithm;
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
    pub use crate::algorithm::extremes::ExtremePoints;
    pub use crate::algorithm::frechet_distance::FrechetDistance;
    pub use crate::algorithm::geodesic_distance::GeodesicDistance;
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
