use crate::{Geodesic, Length, Line, LineString, MultiLineString};

#[deprecated(
    since = "0.29.0",
    note = "Please use the `Geodesic.length(&line)` via the `Length` trait instead."
)]
/// Determine the length of a geometry on an ellipsoidal model of the earth.
///
/// This uses the geodesic measurement methods given by [Karney (2013)]. As opposed to older methods
/// like Vincenty, this method is accurate to a few nanometers and always converges.
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicLength<T, RHS = Self> {
    /// Determine the length of a geometry on an ellipsoidal model of the earth.
    ///
    /// This uses the geodesic measurement methods given by [Karney (2013)]. As opposed to older methods
    /// like Vincenty, this method is accurate to a few nanometers and always converges.
    ///
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::prelude::*;
    /// use geo::LineString;
    ///
    /// let linestring = LineString::from(vec![
    ///     // New York City
    ///     (-74.006, 40.7128),
    ///     // London
    ///     (-0.1278, 51.5074),
    ///     // Osaka
    ///     (135.5244559, 34.687455)
    /// ]);
    ///
    /// let length = linestring.geodesic_length();
    ///
    /// assert_eq!(
    ///     15_109_158., // meters
    ///     length.round()
    /// );
    /// ```
    ///
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn geodesic_length(&self) -> T;
}

#[allow(deprecated)]
impl GeodesicLength<f64> for Line {
    /// The units of the returned value is meters.
    fn geodesic_length(&self) -> f64 {
        Geodesic.length(self)
    }
}

#[allow(deprecated)]
impl GeodesicLength<f64> for LineString {
    fn geodesic_length(&self) -> f64 {
        Geodesic.length(self)
    }
}

#[allow(deprecated)]
impl GeodesicLength<f64> for MultiLineString {
    fn geodesic_length(&self) -> f64 {
        Geodesic.length(self)
    }
}
