use crate::Point;
use geographiclib_rs::{Geodesic, InverseGeodesic};

/// Determine the distance between two geometries on an ellipsoidal model of the earth.
///
/// This uses the geodesic measurement methods given by [Karney (2013)]. As opposed to older methods
/// like Vincenty, this method is accurate to a few nanometers and always converges.
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicDistance<T, Rhs = Self> {
    /// Determine the distance between two geometries on an ellipsoidal model of the earth.
    ///
    /// This uses the geodesic measurement methods given by [Karney (2013)]. As opposed to older methods
    /// like Vincenty, this method is accurate to a few nanometers and always converges.
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Examples
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::Point;
    ///
    /// // New York City
    /// let p1 = Point::<f64>::from((-74.006, 40.7128));
    ///
    /// // London
    /// let p2 = Point::<f64>::from((-0.1278, 51.5074));
    ///
    /// let distance = p1.geodesic_distance(&p2);
    ///
    /// assert_eq!(
    ///     5_585_234., // meters
    ///     distance.round()
    /// );
    /// ```
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_distance(&self, rhs: &Rhs) -> T;
}

impl GeodesicDistance<f64> for Point<f64> {
    fn geodesic_distance(&self, rhs: &Point<f64>) -> f64 {
        Geodesic::wgs84().inverse(self.lat(), self.lng(), rhs.lat(), rhs.lng())
    }
}
