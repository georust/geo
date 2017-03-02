use cgmath::Angle;
use num_traits::{Float, One, cast};
use types::Point;

/// Returns the distance between two geometries.

pub trait HaversineDistance<RHS = Self>
{
    /// The resulting type after applying the Haversine algorithm
    type Output;

    /// Returns the distance between two points:
    ///
    /// ```
    /// # extern crate geo;
    /// # extern crate cgmath;
    /// # #[macro_use] extern crate approx;
    /// #
    /// use geo::Point;
    /// use geo::algorithm::haversine_distance::HaversineDistance;
    /// use cgmath::Deg;
    ///
    /// # fn main() {
    /// let p = Point::new(Deg(-72.1235), Deg(42.3521));
    /// let dist = p.haversine_distance(&Point::new(Deg(-72.1260), Deg(42.45)));
    /// assert_relative_eq!(dist, 10900.115612674515, epsilon = 1.0e-6)
    /// # }
    /// ```
    fn haversine_distance(&self, rhs: &RHS) -> Self::Output;
}

impl<T> HaversineDistance for Point<T>
    where T: Angle
{
    type Output = T::Unitless;

    fn haversine_distance(&self, rhs: &Point<T>) -> Self::Output {
        let (lhs_sin, lhs_cos) = self.lat().sin_cos();
        let (rhs_sin, rhs_cos) = rhs.lat().sin_cos();
        let delta_lng = rhs.lng() - self.lng();

        let a = (lhs_sin * rhs_sin) + (lhs_cos * rhs_cos) * delta_lng.cos();

        cast::<u32, Self::Output>(6378137).unwrap() * a.acos().min(Self::Output::one())
    }
}

#[cfg(test)]
mod test {
    use cgmath::{Deg, Rad};
    use types::Point;
    use algorithm::haversine_distance::HaversineDistance;

    #[test]
    fn distance1_test() {
        let a = Point::new(Deg(0. as f64), Deg(0. as f64));
        let b = Point::new(Deg(1. as f64), Deg(0. as f64));
        assert_relative_eq!(a.haversine_distance(&b), 111319.49079326246_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance2_test() {
        let a = Point::new(Deg(-72.1235), Deg(42.3521));
        let b = Point::new(Deg(72.1260), Deg(70.612));
        assert_relative_eq!(a.haversine_distance(&b), 6378137_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn rad_test() {
        let a = Point::new(Rad(0. as f64), Rad(0. as f64));
        let b = Point::new(Rad(1. as f64), Rad(0. as f64));
        assert_relative_eq!(a.haversine_distance(&b), 6378137_f64, epsilon = 1.0e-6);
    }
}
