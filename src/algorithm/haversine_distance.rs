use num_traits::{Float, FromPrimitive};
use types::{Point};

/// Returns the distance between two geometries.

pub trait HaversineDistance<T, Rhs = Self>
{
    /// Returns the distance between two points:
    ///
    /// ```
    /// use geo::Point;
    /// use geo::algorithm::haversine_distance::HaversineDistance;
    ///
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.haversine_distance(&Point::new(-72.1260, 42.45));
    /// assert_eq!(dist, 10900.115612674515)
    /// ```
    fn haversine_distance(&self, rhs: &Rhs) -> T;
}

impl<T> HaversineDistance<T, Point<T>> for Point<T>
    where T: Float + FromPrimitive
{
    fn haversine_distance(&self, p: &Point<T>) -> T {
        let a = (self.y().to_radians().sin() * p.y().to_radians().sin()) +
            (self.y().to_radians().cos() * p.y().to_radians().cos()) *
            (p.x() - self.x()).to_radians().cos();
        T::from_i32(6378137).unwrap() * a.acos().min(T::one())
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use algorithm::haversine_distance::{HaversineDistance};

    #[test]
    fn distance1_test() {
        assert_eq!(Point::<f64>::new(0., 0.).haversine_distance(&Point::<f64>::new(1., 0.)), 111319.49079326246_f64);
    }
    #[test]
    fn distance2_test() {
        let dist = Point::new(-72.1235, 42.3521).haversine_distance(&Point::new(72.1260, 70.612));
        assert_eq!(dist, 6378137_f64);
    }
}
