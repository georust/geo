use crate::{CoordFloat, Distance, Haversine, Point};
use num_traits::FromPrimitive;

#[deprecated(
    since = "0.29.0",
    note = "Please use the `Haversine.distance` method from the `Distance` trait instead"
)]
/// Determine the distance between two geometries using the [haversine formula].
///
/// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
///
/// *Note*: this implementation uses a mean earth radius of 6371.0088 km (6_371_008.7714 m), based on the recommendation of
/// the IUGG:
///
/// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
/// "Derived Geometric Constants: **R1: mean radius**" (p131)
/// - <https://link.springer.com/article/10.1007%2Fs001900050278>
/// - <https://sci-hub.se/https://doi.org/10.1007/s001900050278>
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
    /// let distance = p1.haversine_distance(&p2);
    ///
    /// assert_eq!(
    ///     5_570_230., // meters
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
    fn haversine_distance(&self, rhs: &Rhs) -> T;
}

#[allow(deprecated)]
impl<T> HaversineDistance<T, Point<T>> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_distance(&self, rhs: &Point<T>) -> T {
        Haversine.distance(*self, *rhs)
    }
}

#[cfg(test)]
mod test {
    #[allow(deprecated)]
    use crate::HaversineDistance;
    use crate::Point;

    #[test]
    fn distance1_test() {
        let a = Point::new(0., 0.);
        let b = Point::new(1., 0.);
        #[allow(deprecated)]
        let distance = a.haversine_distance(&b);
        assert_relative_eq!(distance, 111195.0802335329_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance2_test() {
        let a = Point::new(-72.1235, 42.3521);
        let b = Point::new(72.1260, 70.612);
        #[allow(deprecated)]
        let distance = a.haversine_distance(&b);
        assert_relative_eq!(distance, 7130580.307935911_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance3_test() {
        // this input comes from issue #100
        let a = Point::new(-77.036585, 38.897448);
        let b = Point::new(-77.009080, 38.889825);
        #[allow(deprecated)]
        let distance = a.haversine_distance(&b);
        assert_relative_eq!(distance, 2526.823504306046_f64, epsilon = 1.0e-6);
    }

    #[test]
    fn distance3_test_f32() {
        // this input comes from issue #100
        let a = Point::<f32>::new(-77.03658, 38.89745);
        let b = Point::<f32>::new(-77.00908, 38.889825);
        #[allow(deprecated)]
        let distance = a.haversine_distance(&b);
        assert_relative_eq!(distance, 2526.8354_f32, epsilon = 1.0e-6);
    }
}
