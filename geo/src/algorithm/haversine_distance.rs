use crate::{Point, MEAN_EARTH_RADIUS};
use num_traits::{Float, FromPrimitive};

/// Determine the distance between two geometries using the [haversine formula].
///
/// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
pub trait HaversineDistance<T, Rhs = Self> {
    /// Determine the distance between two geometries using the [haversine
    /// formula].
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::Point;
    /// use geo::prelude::*;
    ///
    /// // New York City
    /// let p1 = Point::<f64>::from((-74.006, 40.7128));
    /// // London
    /// let p2 = Point::<f64>::from((-0.1278, 51.5074));
    ///
    /// let distance = p1.haversine_distance(&p2);
    ///
    /// assert_eq!(
    ///   5_570_222., // meters
    ///   distance.round()
    /// );
    /// ```
    ///
    /// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
    fn haversine_distance(&self, rhs: &Rhs) -> T;
}

impl<T> HaversineDistance<T, Point<T>> for Point<T>
where
    T: Float + FromPrimitive,
{
    fn haversine_distance(&self, rhs: &Point<T>) -> T {
        let two = T::one() + T::one();
        let theta1 = self.y().to_radians();
        let theta2 = rhs.y().to_radians();
        let delta_theta = (rhs.y() - self.y()).to_radians();
        let delta_lambda = (rhs.x() - self.x()).to_radians();
        let a = (delta_theta / two).sin().powi(2)
            + theta1.cos() * theta2.cos() * (delta_lambda / two).sin().powi(2);
        let c = two * a.sqrt().asin();
        T::from(MEAN_EARTH_RADIUS).unwrap() * c
    }
}

#[cfg(test)]
mod test {
    use crate::algorithm::haversine_distance::HaversineDistance;
    use crate::Point;

    #[test]
    fn distance1_test() {
        let a = Point::<f64>::new(0., 0.);
        let b = Point::<f64>::new(1., 0.);
        assert_relative_eq!(
            a.haversine_distance(&b),
            111194.92664455874_f64,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn distance2_test() {
        let a = Point::new(-72.1235, 42.3521);
        let b = Point::new(72.1260, 70.612);
        assert_relative_eq!(
            a.haversine_distance(&b),
            7130570.458772508_f64,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn distance3_test() {
        // this input comes from issue #100
        let a = Point::<f64>::new(-77.036585, 38.897448);
        let b = Point::<f64>::new(-77.009080, 38.889825);
        assert_relative_eq!(
            a.haversine_distance(&b),
            2526.820014113592_f64,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn distance3_test_f32() {
        // this input comes from issue #100
        let a = Point::<f32>::new(-77.036585, 38.897448);
        let b = Point::<f32>::new(-77.009080, 38.889825);
        assert_relative_eq!(a.haversine_distance(&b), 2526.8318_f32, epsilon = 1.0e-6);
    }
}
