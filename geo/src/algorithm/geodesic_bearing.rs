use crate::Point;
use geo_types::CoordNum;
use geographiclib_rs::{Geodesic, InverseGeodesic};

/// Returns the bearing to another Point in degrees on a geodesic.
///
/// This uses the geodesic methods given by [Karney (2013)].
///
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub trait GeodesicBearing<T: CoordNum> {
    /// Returns the bearing to another Point in degrees, where North is 0° and East is 90°.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::GeodesicBearing;
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Point::new(9.27411867078536, 48.8403266058781);
    /// let bearing = p_1.geodesic_bearing(p_2)?;
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    fn geodesic_bearing(&self, point: Point<T>) -> Result<T, String>;

    /// Returns the bearing and distance to another Point in a (bearing, distance) tuple.
    ///
    /// # Units
    ///
    /// - `bearing`: degrees, zero degrees is north. East is 90°.
    /// - `distance`: meters
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::GeodesicBearing;
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Point::new(9.27411867078536, 48.8403266058781);
    /// let (bearing, distance) = p_1.geodesic_bearing_distance(p_2);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// assert_relative_eq!(distance, 10000., epsilon = 1.0e-6);
    /// ```
    fn geodesic_bearing_distance(&self, point: Point<T>) -> (T, T);
}

impl GeodesicBearing<f64> for Point<f64> {
    fn geodesic_bearing(&self, rhs: Point<f64>) -> Result<f64, String> {
        self.check_coordinate_limits()?;
        rhs.check_coordinate_limits()?;
        let (azi1, _, _) = Geodesic::wgs84().inverse(self.y(), self.x(), rhs.y(), rhs.x());
        Ok(azi1)
    }

    fn geodesic_bearing_distance(&self, rhs: Point<f64>) -> (f64, f64) {
        let (distance, azi1, _, _) =
            Geodesic::wgs84().inverse(self.y(), self.x(), rhs.y(), rhs.x());
        (azi1, distance)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::point;

    #[test]
    fn north_bearing() {
        let p_1 = point!(x: 9., y: 47.);
        let p_2 = point!(x: 9., y: 48.);
        let bearing = p_1.geodesic_bearing(p_2).unwrap();
        assert_relative_eq!(bearing, 0.);
    }

    #[test]
    fn east_bearing() {
        let p_1 = point!(x: 9., y: 10.);
        let p_2 = point!(x: 18.118501133357412, y: 9.875322179340463);
        let bearing = p_1.geodesic_bearing(p_2).unwrap();
        assert_relative_eq!(bearing, 90.);
    }

    #[test]
    fn northeast_bearing() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        let p_2 = point!(x: 9.27411867078536, y: 48.8403266058781);
        let bearing = p_1.geodesic_bearing(p_2).unwrap();
        assert_relative_eq!(bearing, 45., epsilon = 1.0e-11);
    }

    #[test]
    fn consistent_with_destination() {
        use crate::algorithm::GeodesicDestination;
        let p_1 = point!(x: 9.177789688110352, y: 48.776781529534965);
        let p_2 = p_1.geodesic_destination(45., 10000.);
        let (bearing, distance) = p_1.geodesic_bearing_distance(p_2);
        assert_relative_eq!(bearing, 45., epsilon = 1.0e-11);
        assert_relative_eq!(distance, 10000.0, epsilon = 1.0e-9);
    }
}
