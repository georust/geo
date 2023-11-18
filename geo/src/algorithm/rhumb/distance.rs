use crate::{CoordFloat, Point, MEAN_EARTH_RADIUS};
use num_traits::FromPrimitive;

use super::RhumbCalculations;

/// Determine the distance between two geometries along a [rhumb line].
///
/// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
///
/// *Note*: this implementation uses a mean earth radius of 6371.088 km, based on the [recommendation of
/// the IUGG](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait RhumbDistance<T, Rhs = Self> {
    /// Determine the distance between along the [rhumb line] between two geometries.
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::point;
    ///
    /// // New York City
    /// let p1 = point!(x: -74.006f64, y: 40.7128f64);
    ///
    /// // London
    /// let p2 = point!(x: -0.1278f64, y: 51.5074f64);
    ///
    /// let distance = p1.rhumb_distance(&p2);
    ///
    /// assert_eq!(
    ///     5_794_129., // meters
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn rhumb_distance(&self, rhs: &Rhs) -> T;
}

impl<T> RhumbDistance<T, Point<T>> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_distance(&self, rhs: &Point<T>) -> T {
        let calculations = RhumbCalculations::new(self, rhs);
        calculations.delta() * T::from(MEAN_EARTH_RADIUS).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::Point;
    use crate::RhumbDistance;

    #[test]
    fn distance1_test() {
        let a = Point::new(0., 0.);
        let b = Point::new(1., 0.);
        assert_relative_eq!(
            a.rhumb_distance(&b),
            111195.0802335329_f64,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn distance2_test() {
        let a = Point::new(-72.1235, 42.3521);
        let b = Point::new(72.1260, 70.612);
        assert_relative_eq!(
            a.rhumb_distance(&b),
            8903668.508603323_f64,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn distance3_test() {
        // this input comes from issue #100
        let a = Point::new(-77.036585, 38.897448);
        let b = Point::new(-77.009080, 38.889825);
        assert_relative_eq!(
            a.rhumb_distance(&b),
            2526.7031699343006_f64,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn distance3_test_f32() {
        // this input comes from issue #100
        let a = Point::<f32>::new(-77.03658, 38.89745);
        let b = Point::<f32>::new(-77.00908, 38.889825);
        assert_relative_eq!(a.rhumb_distance(&b), 2526.7273_f32, epsilon = 1.0e-6);
    }
}
