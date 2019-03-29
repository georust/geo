extern crate geo_types;
extern crate num_traits;
#[cfg(feature = "use-serde")]
#[macro_use]
extern crate serde;
#[cfg_attr(all(test, feature = "use-proj"), macro_use)]
extern crate failure;
#[cfg(feature = "postgis-integration")]
extern crate postgis;
#[cfg(feature = "use-proj")]
extern crate proj;
extern crate rstar;

pub use crate::algorithm::*;
pub use crate::traits::ToGeo;
pub use crate::types::*;

pub use geo_types::{
    Coordinate, CoordinateType, Geometry, GeometryCollection, Line, LineString, MultiLineString,
    MultiPoint, MultiPolygon, Point, Polygon, Rect, Triangle,
};

/// This module includes all the functions of geometric calculations
pub mod algorithm;
mod traits;
mod types;
mod utils;

#[macro_use]
mod macros;

#[cfg(test)]
#[macro_use]
extern crate approx;

// Mean radius of Earth in meters
const MEAN_EARTH_RADIUS: f64 = 6_371_000.0;

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
    pub use crate::algorithm::closest_point::ClosestPoint;
    pub use crate::algorithm::contains::Contains;
    pub use crate::algorithm::convexhull::ConvexHull;
    pub use crate::algorithm::euclidean_distance::EuclideanDistance;
    pub use crate::algorithm::euclidean_length::EuclideanLength;
    pub use crate::algorithm::extremes::ExtremePoints;
    #[cfg(feature = "postgis-integration")]
    pub use crate::algorithm::from_postgis::FromPostgis;
    pub use crate::algorithm::haversine_destination::HaversineDestination;
    pub use crate::algorithm::haversine_distance::HaversineDistance;
    pub use crate::algorithm::haversine_intermediate::HaversineIntermediate;
    pub use crate::algorithm::haversine_length::HaversineLength;
    pub use crate::algorithm::intersects::Intersects;
    pub use crate::algorithm::map_coords::MapCoords;
    pub use crate::algorithm::orient::Orient;
    #[cfg(feature = "use-proj")]
    pub use crate::algorithm::proj::Proj;
    pub use crate::algorithm::rotate::{Rotate, RotatePoint};
    pub use crate::algorithm::simplify::Simplify;
    pub use crate::algorithm::simplifyvw::SimplifyVW;
    #[cfg(feature = "postgis-integration")]
    pub use crate::algorithm::to_postgis::ToPostgis;
    pub use crate::algorithm::translate::Translate;
    pub use crate::algorithm::vincenty_distance::VincentyDistance;
    pub use crate::algorithm::vincenty_length::VincentyLength;
}
