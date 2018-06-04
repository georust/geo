extern crate geo_types;
extern crate num_traits;
#[cfg(feature = "use-serde")]
#[macro_use]
extern crate serde;
extern crate spade;
#[cfg(feature = "postgis-integration")]
extern crate postgis;
#[cfg(feature = "use-proj")]
extern crate proj;
#[cfg_attr(all(test, feature = "use-proj"), macro_use)]
extern crate failure;

pub use traits::ToGeo;
pub use types::*;
pub use algorithm::*;

pub use geo_types::{Geometry, MultiPoint, Point, Polygon,
                    MultiPolygon, LineString, MultiLineString, Line, GeometryCollection,
                    Coordinate, CoordinateType};

mod traits;
mod types;
/// This module includes all the functions of geometric calculations
pub mod algorithm;

#[cfg(test)]
#[macro_use]
extern crate approx;

// Mean radius of Earth in meters
const MEAN_EARTH_RADIUS: f64 = 6_371_000.0;

// Radius of Earth at the equator in meters (derived from the WGS-84 ellipsoid)
const EQUATORIAL_EARTH_RADIUS: f64 = 6_378_137.0;

// Radius of Earth at the poles in meters (derived from the WGS-84 ellipsoid)
const POLAR_EARTH_RADIUS: f64 = 6_356_752.314245;

// Flattening of the WGS-84 ellipsoid - https://en.wikipedia.org/wiki/Flattening
const EARTH_FLATTENING: f64 =
    (EQUATORIAL_EARTH_RADIUS - POLAR_EARTH_RADIUS) / EQUATORIAL_EARTH_RADIUS;

/// A prelude which re-exports the traits for manipulating objects in this
/// crate. Typically imported with `use geo::prelude::*`.
pub mod prelude {
    pub use algorithm::area::Area;
    pub use algorithm::bearing::Bearing;
    pub use algorithm::boundingbox::BoundingBox;
    pub use algorithm::centroid::Centroid;
    pub use algorithm::contains::Contains;
    pub use algorithm::convexhull::ConvexHull;
    pub use algorithm::euclidean_distance::EuclideanDistance;
    pub use algorithm::euclidean_length::EuclideanLength;
    pub use algorithm::extremes::ExtremePoints;
    pub use algorithm::haversine_intermediate::HaversineIntermediate;
    pub use algorithm::haversine_destination::HaversineDestination;
    pub use algorithm::haversine_distance::HaversineDistance;
    pub use algorithm::intersects::Intersects;
    pub use algorithm::map_coords::MapCoords;
    pub use algorithm::orient::Orient;
    pub use algorithm::rotate::{Rotate, RotatePoint};
    pub use algorithm::simplify::Simplify;
    pub use algorithm::simplifyvw::SimplifyVW;
    pub use algorithm::translate::Translate;
    pub use algorithm::closest_point::ClosestPoint;
    pub use algorithm::haversine_length::HaversineLength;
    #[cfg(feature = "postgis-integration")]
    pub use algorithm::from_postgis::FromPostgis;
    #[cfg(feature = "postgis-integration")]
    pub use algorithm::to_postgis::ToPostgis;
    #[cfg(feature = "use-proj")]
    pub use algorithm::proj::Proj;
    pub use algorithm::vincenty_distance::VincentyDistance;
    pub use algorithm::vincenty_length::VincentyLength;
}
