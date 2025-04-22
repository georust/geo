use crate::line_measures::{Destination, Geodesic};
use crate::Point;
use geo_types::CoordNum;

#[deprecated(
    since = "0.29.0",
    note = "Please use the `Geodesic.destination` method from the `Destination` trait instead"
)]
/// Returns a new Point using the distance to the existing Point and a bearing for the direction on a geodesic.
///
/// This uses the geodesic methods given by [Karney (2013)].
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicDestination<T: CoordNum> {
    /// Returns a new Point using distance to the existing Point and a bearing for the direction.
    ///
    /// # Units
    ///
    /// - `bearing`: degrees, zero degrees is north
    /// - `distance`: meters
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[allow(deprecated)]
    /// use geo::GeodesicDestination;
    /// use geo::Point;
    ///
    /// // Determine the point 10000 km NE of JFK.
    /// let jfk = Point::new(-73.78, 40.64);
    /// let northeast_bearing = 45.0;
    /// let distance = 10e6;
    ///
    /// # #[allow(deprecated)]
    /// let p_1 = jfk.geodesic_destination(northeast_bearing, distance);
    /// use approx::assert_relative_eq;
    /// assert_relative_eq!(p_1.x(), 49.052487092959836);
    /// assert_relative_eq!(p_1.y(), 32.621100463725796);
    /// ```
    fn geodesic_destination(&self, bearing: T, distance: T) -> Point<T>;
}

#[allow(deprecated)]
impl GeodesicDestination<f64> for Point<f64> {
    fn geodesic_destination(&self, bearing: f64, distance: f64) -> Point<f64> {
        Geodesic.destination(*self, bearing, distance)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(deprecated)]
    use crate::GeodesicDistance;

    #[test]
    fn returns_a_new_point() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        #[allow(deprecated)]
        let p_2 = p_1.geodesic_destination(45., 10000.);

        assert_relative_eq!(
            p_2,
            Point::new(9.27411867078536, 48.8403266058781),
            epsilon = 1.0e-6
        );

        #[allow(deprecated)]
        let distance = p_1.geodesic_distance(&p_2);
        assert_relative_eq!(distance, 10000., epsilon = 1.0e-6)
    }

    #[test]
    fn bearing_zero_is_north() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        #[allow(deprecated)]
        let p_2 = p_1.geodesic_destination(0., 1000.);
        assert_relative_eq!(p_1.x(), p_2.x(), epsilon = 1.0e-6);
        assert!(p_2.y() > p_1.y())
    }

    #[test]
    fn bearing_90_is_east() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        #[allow(deprecated)]
        let p_2 = p_1.geodesic_destination(90., 1000.);
        assert_relative_eq!(p_1.y(), p_2.y(), epsilon = 1.0e-6);
        assert!(p_2.x() > p_1.x())
    }
}
