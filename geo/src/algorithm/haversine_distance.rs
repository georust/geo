use num_traits::{Float, FromPrimitive};
use ::{Point, MEAN_EARTH_RADIUS};

/// Returns the Haversine distance between two geometries.

pub trait HaversineDistance<T, Rhs = Self> {
    /// Returns the Haversine distance between two points:
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate geo;
    /// # #[macro_use] extern crate approx;
    /// #
    /// use geo::Point;
    /// use geo::algorithm::haversine_distance::HaversineDistance;
    ///
    /// # fn main() {
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.haversine_distance(&Point::new(-72.1260, 42.45));
    /// assert_relative_eq!(dist, 10887.91861391182, epsilon = 1.0e-6)
    /// # }
    /// ```
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
    use ::Point;
    use algorithm::haversine_distance::HaversineDistance;

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
