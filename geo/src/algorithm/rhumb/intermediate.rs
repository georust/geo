use crate::{CoordFloat, Point, MEAN_EARTH_RADIUS};
use num_traits::FromPrimitive;

use super::RhumbCalculations;

/// Returns a new Point along a rhumb line between two existing points

pub trait RhumbIntermediate<T: CoordFloat> {
    /// Returns a new Point along a [rhumb line] between two existing points.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use approx::assert_relative_eq;
    /// use geo::RhumbIntermediate;
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    /// let i20 = p1.rhumb_intermediate(&p2, 0.2);
    /// let i50 = p1.rhumb_intermediate(&p2, 0.5);
    /// let i80 = p1.rhumb_intermediate(&p2, 0.8);
    /// let i20_should = Point::new(32.7, 21.0);
    /// let i50_should = Point::new(67.0, 22.5);
    /// let i80_should = Point::new(101.7, 24.0);
    /// assert_relative_eq!(i20.x(), i20_should.x(), epsilon = 0.2);
    /// assert_relative_eq!(i20.y(), i20_should.y(), epsilon = 0.2);
    /// assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 0.2);
    /// assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 0.2);
    /// assert_relative_eq!(i80.x(), i80_should.x(), epsilon = 0.2);
    /// assert_relative_eq!(i80.y(), i80_should.y(), epsilon = 0.2);
    /// ```
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line

    fn rhumb_intermediate(&self, other: &Point<T>, f: T) -> Point<T>;
    fn rhumb_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>>;
}

impl<T> RhumbIntermediate<T> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn rhumb_intermediate(&self, other: &Point<T>, f: T) -> Point<T> {
        let calculations = RhumbCalculations::new(self, other);
        calculations.intermediate(f)
    }

    fn rhumb_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>> {
        let max_delta = max_dist / T::from(MEAN_EARTH_RADIUS).unwrap();
        let calculations = RhumbCalculations::new(self, other);
        calculations.intermediate_fill(max_delta, include_ends)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::RhumbIntermediate;

    #[test]
    fn f_is_zero_or_one_test() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(15.0, 25.0);
        let i0 = p1.rhumb_intermediate(&p2, 0.0);
        let i100 = p1.rhumb_intermediate(&p2, 1.0);
        assert_relative_eq!(i0.x(), p1.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i0.y(), p1.y(), epsilon = 1.0e-6);
        assert_relative_eq!(i100.x(), p2.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i100.y(), p2.y(), epsilon = 1.0e-6);
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(125.0, 25.0);
        let i20 = p1.rhumb_intermediate(&p2, 0.2);
        let i50 = p1.rhumb_intermediate(&p2, 0.5);
        let i80 = p1.rhumb_intermediate(&p2, 0.8);
        let i20_should = Point::new(32.6766, 21.0);
        let i50_should = Point::new(66.9801, 22.5);
        let i80_should = Point::new(101.6577, 24.0);
        assert_relative_eq!(i20.x(), i20_should.x(), epsilon = 0.2);
        assert_relative_eq!(i20.y(), i20_should.y(), epsilon = 0.2);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 0.2);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 0.2);
        assert_relative_eq!(i80.x(), i80_should.x(), epsilon = 0.2);
        assert_relative_eq!(i80.y(), i80_should.y(), epsilon = 0.2);
    }

    #[test]
    fn should_be_straight_across_test() {
        let p1 = Point::new(0.0, 10.0);
        let p2 = Point::new(180.0, 10.0);
        let i50 = p1.rhumb_intermediate(&p2, 0.5);
        let i50_should = Point::new(90.0, 10.0);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 1.0e-6);
    }

    #[test]
    fn should_be_start_end_test() {
        let p1 = Point::new(30.0, 40.0);
        let p2 = Point::new(40.0, 50.0);
        let max_dist = 1500000.0; // meters
        let include_ends = true;
        let route = p1.rhumb_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, p2]);
    }

    #[test]
    fn should_add_i50_test() {
        let p1 = Point::new(30.0, 40.0);
        let p2 = Point::new(40.0, 50.0);
        let max_dist = 1000000.0; // meters
        let include_ends = true;
        let i50 = p1.clone().rhumb_intermediate(&p2, 0.5);
        let route = p1.rhumb_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, i50, p2]);
    }

    #[test]
    fn should_add_i25_i50_i75_test() {
        let p1 = Point::new(30.0, 40.0);
        let p2 = Point::new(40.0, 50.0);
        let max_dist = 400000.0; // meters
        let include_ends = true;
        let i25 = p1.clone().rhumb_intermediate(&p2, 0.25);
        let i50 = p1.clone().rhumb_intermediate(&p2, 0.5);
        let i75 = p1.clone().rhumb_intermediate(&p2, 0.75);
        let route = p1.rhumb_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, i25, i50, i75, p2]);
    }
}
