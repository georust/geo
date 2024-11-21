//! Line measurements like [`Bearing`] and [`Distance`] for various metric spaces like [`Euclidean`], [`Haversine`], [`Geodesic`], and [`Rhumb`].

mod bearing;
pub use bearing::Bearing;

mod destination;
pub use destination::Destination;

mod distance;
pub use distance::Distance;

mod interpolate_point;
pub use interpolate_point::InterpolatePoint;

mod length;
pub use length::Length;

mod densify;
pub use densify::Densify;

mod frechet_distance;
pub use frechet_distance::FrechetDistance;

pub mod metric_spaces;
pub use metric_spaces::{Euclidean, Geodesic, Haversine, Rhumb};
