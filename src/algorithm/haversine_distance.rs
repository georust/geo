use num_traits::{Float, FromPrimitive};
use types::Point;

/// Returns the distance between two geometries.

pub trait HaversineDistance<T, Rhs = Self>
{
    /// Returns the distance between two points:
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
    /// assert_relative_eq!(dist, 10900.115612674515, epsilon = 1.0e-6)
    /// # }
    /// ```
    fn haversine_distance(&self, rhs: &Rhs) -> T;
}

impl<T> HaversineDistance<T, Point<T>> for Point<T>
    where T: Float + FromPrimitive
{
    fn haversine_distance(&self, rhs: &Point<T>) -> T {
        let (lhs_sin, lhs_cos) = self.y().to_radians().sin_cos();
        let (rhs_sin, rhs_cos) = rhs.y().to_radians().sin_cos();
        let delta_lng = rhs.lng() - self.lng();

        let a = (lhs_sin * rhs_sin) + (lhs_cos * rhs_cos) * delta_lng.to_radians().cos();

        T::from_i32(6378137).unwrap() * a.acos().min(T::one())
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use algorithm::haversine_distance::HaversineDistance;

    #[test]
    fn distance1_test() {
        let a = Point::<f64>::new(0., 0.);
        let b = Point::<f64>::new(1., 0.);
        assert_relative_eq!(a.haversine_distance(&b), 111319.49079326246_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance2_test() {
        let a = Point::new(-72.1235, 42.3521);
        let b = Point::new(72.1260, 70.612);
        assert_relative_eq!(a.haversine_distance(&b), 6378137_f64, epsilon = 1.0e-6);
    }
}
