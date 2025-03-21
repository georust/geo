//! Line measurements like [`Bearing`] and [`Distance`] for various metric spaces like [`Euclidean`], [`Haversine`], [`Geodesic`], and [`Rhumb`].
//!
//! ## Example
//! ```
//! use geo::{Haversine, Euclidean, Distance, Point, Bearing};
//! let p1: Point = Point::new(0.0, 0.0);
//! let p2: Point = Point::new(0.0, 2.0);
//!
//! assert_eq!(Euclidean.distance(p1, p2), 2.0);
//!
//! // The units of the output depend on the metric space.
//! // In the case of [`Haversine`], it's meters.
//! // See the documentation for each metric space for details.
//! assert_eq!(Haversine.distance(p1, p2).round(), 222_390.0);
//!
//! // Due north
//! assert_eq!(Haversine.bearing(p1, p2), 0.0);
//! ```
//!
//! See the individual [`metric_spaces`] or algorithm [traits](#traits) for more details.

mod bearing;
pub use bearing::Bearing;

mod destination;
pub use destination::Destination;

mod distance;
pub use distance::Distance;

mod interpolate_point;
pub use interpolate_point::InterpolatePoint;

mod interpolate_line;
pub use interpolate_line::{InterpolatableLine, InterpolateLine};

mod length;
pub use length::{Length, LengthMeasurable};

mod densify;
pub use densify::{Densifiable, Densify};

mod frechet_distance;
pub use frechet_distance::FrechetDistance;

pub mod metric_spaces;
pub use metric_spaces::{Euclidean, Geodesic, GeodesicMeasure, Haversine, HaversineMeasure, Rhumb};
