use crate::Point;
use geo_types::{CoordFloat, CoordNum};
use geographiclib_rs::{DirectGeodesic, Geodesic};

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
    /// use geo::GeodesicDestination;
    /// use geo::Point;
    ///
    /// // Determine the point 10000 km NE of JFK.
    /// let jfk = Point::new(-73.78, 40.64);
    /// let northeast_bearing = 45.0;
    /// let distance = 10e6;
    ///
    /// let p_1 = jfk.geodesic_destination(northeast_bearing, distance);
    /// use approx::assert_relative_eq;
    /// assert_relative_eq!(p_1.x(), 49.052487092959836);
    /// assert_relative_eq!(p_1.y(), 32.621100463725796);
    /// ```
    fn geodesic_destination(&self, bearing: T, distance: T) -> Point<T>;
}

impl<T: CoordFloat> GeodesicDestination<T> for Point<T> {
    fn geodesic_destination(&self, bearing: T, distance: T) -> Point<T> {
        let bearing = bearing.to_f64().unwrap();
        let distance = distance.to_f64().unwrap();
        let y = self.y().to_f64().unwrap();
        let x = self.x().to_f64().unwrap();
        let (lat, lon) = Geodesic::wgs84().direct(y, x, bearing, distance);
        Point::new(T::from(lon).unwrap(), T::from(lat).unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::GeodesicDistance;

    #[test]
    fn returns_a_new_point() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.geodesic_destination(45., 10000.);

        assert_relative_eq!(
            p_2,
            Point::new(9.27411867078536, 48.8403266058781),
            epsilon = 1.0e-6
        );

        let distance = p_1.geodesic_distance(&p_2);
        assert_relative_eq!(distance, 10000., epsilon = 1.0e-6)
    }

    #[test]
    fn bearing_zero_is_north() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.geodesic_destination(0., 1000.);
        assert_relative_eq!(p_1.x(), p_2.x(), epsilon = 1.0e-6);
        assert!(p_2.y() > p_1.y())
    }

    #[test]
    fn bearing_90_is_east() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.geodesic_destination(90., 1000.);
        assert_relative_eq!(p_1.y(), p_2.y(), epsilon = 1.0e-6);
        assert!(p_2.x() > p_1.x())
    }
}
