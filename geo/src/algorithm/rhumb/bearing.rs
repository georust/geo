use num_traits::FromPrimitive;

use crate::{CoordFloat, Point};

use super::RhumbCalculations;

/// Returns the bearing to another Point in degrees.
///
/// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
/// (<https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf>)

pub trait RhumbBearing<T: CoordFloat + FromPrimitive> {
    /// Returns the bearing to another Point in degrees along a [rhumb line], where North is 0° and East is 90°.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::RhumbBearing;
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Point::new(9.274348757829898, 48.84037308229984);
    /// let bearing = p_1.rhumb_bearing(p_2);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line

    fn rhumb_bearing(&self, point: Point<T>) -> T;
}

impl<T> RhumbBearing<T> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_bearing(&self, point: Point<T>) -> T {
        let three_sixty = T::from(360.0f64).unwrap();

        let calculations = RhumbCalculations::new(self, &point);
        (calculations.theta().to_degrees() + three_sixty) % three_sixty
    }
}

#[cfg(test)]
mod test {
    use crate::point;
    use crate::RhumbBearing;
    use crate::RhumbDestination;

    #[test]
    fn north_bearing() {
        let p_1 = point!(x: 9., y: 47.);
        let p_2 = point!(x: 9., y: 48.);
        let bearing = p_1.rhumb_bearing(p_2);
        assert_relative_eq!(bearing, 0.);
    }

    #[test]
    fn equatorial_east_bearing() {
        let p_1 = point!(x: 9., y: 0.);
        let p_2 = point!(x: 10., y: 0.);
        let bearing = p_1.rhumb_bearing(p_2);
        assert_relative_eq!(bearing, 90.);
    }

    #[test]
    fn east_bearing() {
        let p_1 = point!(x: 9., y: 10.);
        let p_2 = point!(x: 18.131938299366652, y: 10.);

        let bearing = p_1.rhumb_bearing(p_2);
        assert_relative_eq!(bearing, 90.);
    }

    #[test]
    fn northeast_bearing() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        let p_2 = point!(x: 9.274348757829898, y: 48.84037308229984);
        let bearing = p_1.rhumb_bearing(p_2);
        assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    }

    #[test]
    fn consistent_with_destination() {
        let p_1 = point!(x: 9.177789688110352f64, y: 48.776781529534965);
        let p_2 = p_1.rhumb_destination(45., 10000.);

        let b_1 = p_1.rhumb_bearing(p_2);
        assert_relative_eq!(b_1, 45., epsilon = 1.0e-6);
    }
}
