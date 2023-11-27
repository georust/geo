use crate::{CoordFloat, Point, MEAN_EARTH_RADIUS};
use num_traits::FromPrimitive;

use super::calculate_destination;

/// Returns the destination Point having travelled the given distance along a [rhumb line]
/// from the origin geometry with the given bearing
///
/// *Note*: this implementation uses a mean earth radius of 6371.088 km, based on the [recommendation of
/// the IUGG](ftp://athena.fsv.cvut.cz/ZFG/grs80-Moritz.pdf)
pub trait RhumbDestination<T: CoordFloat> {
    /// Returns the destination Point having travelled the given distance along a [rhumb line]
    /// from the origin Point with the given bearing
    ///
    /// # Units
    ///
    /// - `bearing`: degrees, zero degrees is north
    /// - `distance`: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::RhumbDestination;
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = p_1.rhumb_destination(45., 10000.);
    /// assert_eq!(p_2, Point::new(9.274348757829898, 48.84037308229984))
    /// ```
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn rhumb_destination(&self, bearing: T, distance: T) -> Point<T>;
}

impl<T> RhumbDestination<T> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_destination(&self, bearing: T, distance: T) -> Point<T> {
        let delta = distance / T::from(MEAN_EARTH_RADIUS).unwrap(); // angular distance in radians
        let lambda1 = self.x().to_radians();
        let phi1 = self.y().to_radians();
        let theta = bearing.to_radians();

        calculate_destination(delta, lambda1, phi1, theta)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::RhumbDistance;
    use num_traits::pow;

    #[test]
    fn returns_a_new_point() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.rhumb_destination(45., 10000.);
        assert_eq!(p_2, Point::new(9.274348757829898, 48.84037308229984));
        let distance = p_1.rhumb_distance(&p_2);
        assert_relative_eq!(distance, 10000., epsilon = 1.0e-6)
    }

    #[test]
    fn direct_and_indirect_destinations_are_close() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.rhumb_destination(45., 10000.);
        let square_edge = { pow(10000., 2) / 2f64 }.sqrt();
        let p_3 = p_1.rhumb_destination(0., square_edge);
        let p_4 = p_3.rhumb_destination(90., square_edge);
        assert_relative_eq!(p_4, p_2, epsilon = 1.0e-3);
    }

    #[test]
    fn bearing_zero_is_north() {
        let p_1 = Point::new(9.177789688110352, 48.776781529534965);
        let p_2 = p_1.rhumb_destination(0., 1000.);
        assert_relative_eq!(p_1.x(), p_2.x(), epsilon = 1.0e-6);
        assert!(p_2.y() > p_1.y())
    }
}
