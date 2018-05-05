extern crate geo_types;
extern crate num_traits;
#[cfg(feature = "use-serde")]
#[macro_use]
extern crate serde;
extern crate spade;
#[cfg(feature = "postgis-integration")]
extern crate postgis;
#[cfg(feature = "proj")]
extern crate proj_sys;
#[cfg(feature = "proj")]
extern crate libc;
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
    #[cfg(feature = "proj")]
    pub use algorithm::proj::{Proj};
}
