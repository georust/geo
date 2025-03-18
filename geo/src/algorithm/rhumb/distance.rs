use crate::{CoordFloat, Distance, Point, Rhumb};
use num_traits::FromPrimitive;

#[deprecated(
    since = "0.29.0",
    note = "Please use the `Rhumb.distance` method from the `Distance` trait instead"
)]
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
    /// # #[allow(deprecated)]
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

#[allow(deprecated)]
impl<T> RhumbDistance<T, Point<T>> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_distance(&self, rhs: &Point<T>) -> T {
        Rhumb.distance(*self, *rhs)
    }
}

#[cfg(test)]
mod test {
    use crate::Point;
    #[allow(deprecated)]
    use crate::RhumbDistance;

    #[test]
    fn distance1_test() {
        let a = Point::new(0., 0.);
        let b = Point::new(1., 0.);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 111195.0802335329_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance2_test() {
        let a = Point::new(-72.1235, 42.3521);
        let b = Point::new(72.1260, 70.612);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 8903668.508603323_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance3_test() {
        // this input comes from issue #100
        let a = Point::new(-77.036585, 38.897448);
        let b = Point::new(-77.009080, 38.889825);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 2526.823513863995_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance3_test_f32() {
        // this input comes from issue #100
        let a = Point::<f32>::new(-77.03658, 38.89745);
        let b = Point::<f32>::new(-77.00908, 38.889825);
        #[allow(deprecated)]
        let distance = a.rhumb_distance(&b);
        assert_relative_eq!(distance, 2527.4585_f32, epsilon = 1.0e-6);
    }
}
