extern crate num_traits;
#[macro_use]
extern crate serde_derive;
extern crate spade;
#[cfg(feature = "reprojection")]
extern crate proj;
#[cfg(feature = "postgis-integration")]
extern crate postgis;

pub use traits::ToGeo;
pub use types::*;
pub use algorithm::*;

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
    pub use algorithm::distance::Distance;
    pub use algorithm::extremes::ExtremePoints;
    pub use algorithm::haversine_destination::HaversineDestination;
    pub use algorithm::haversine_distance::HaversineDistance;
    pub use algorithm::intersects::Intersects;
    pub use algorithm::length::Length;
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
    #[cfg(feature = "reprojection")]
    pub use algorithm::reproject::Reproject;
}
