use crate::{CoordFloat, Haversine, InterpolatePoint, Point};
use num_traits::FromPrimitive;

#[deprecated(
    since = "0.29.0",
    note = "Please use the `InterpolatePoint` trait instead"
)]
/// Returns a new Point along a great circle route between two existing points
pub trait HaversineIntermediate<T: CoordFloat> {
    #[deprecated(
        since = "0.29.0",
        note = "Please use `Haversine.point_at_ratio_between` from the `InterpolatePoint` trait instead"
    )]
    /// Returns a new `Point` along a great circle route between `self` and `other`.
    ///
    /// * `other` - The other point to interpolate towards.
    /// * `ratio` - How far along the route should the new point be, with 0.0 being at `self`
    ///   and 1.0 being at `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// # #[allow(deprecated)]
    /// use geo::HaversineIntermediate;
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// # #[allow(deprecated)]
    /// let i20 = p1.haversine_intermediate(&p2, 0.2);
    /// assert_relative_eq!(i20, Point::new(29.8, 29.9), epsilon = 0.2);
    ///
    /// # #[allow(deprecated)]
    /// let i80 = p1.haversine_intermediate(&p2, 0.8);
    /// assert_relative_eq!(i80, Point::new(103.5, 33.5), epsilon = 0.2);
    /// ```
    fn haversine_intermediate(&self, other: &Point<T>, ratio: T) -> Point<T>;

    #[deprecated(
        since = "0.29.0",
        note = "Please use `Haversine.points_along_line` from the `InterpolatePoint` trait instead"
    )]
    /// Interpolates `Point`s along a great circle route between self and `other`.
    ///
    /// As many points as necessary will be added such that the distance between points
    /// never exceeds `max_dist`.
    ///
    /// `include_ends`: Should the start and end points be included in the output?
    fn haversine_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>>;
}

#[allow(deprecated)]
impl<T> HaversineIntermediate<T> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn haversine_intermediate(&self, other: &Point<T>, ratio: T) -> Point<T> {
        Haversine.point_at_ratio_between(*self, *other, ratio)
    }

    fn haversine_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>> {
        Haversine
            .points_along_line(*self, *other, max_dist, include_ends)
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[allow(deprecated)]
    use crate::HaversineIntermediate;

    #[test]
    fn f_is_zero_or_one_test() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(15.0, 25.0);
        #[allow(deprecated)]
        let i0 = p1.haversine_intermediate(&p2, 0.0);
        #[allow(deprecated)]
        let i100 = p1.haversine_intermediate(&p2, 1.0);
        assert_relative_eq!(i0.x(), p1.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i0.y(), p1.y(), epsilon = 1.0e-6);
        assert_relative_eq!(i100.x(), p2.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i100.y(), p2.y(), epsilon = 1.0e-6);
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(125.0, 25.0);
        #[allow(deprecated)]
        let i20 = p1.haversine_intermediate(&p2, 0.2);
        #[allow(deprecated)]
        let i50 = p1.haversine_intermediate(&p2, 0.5);
        #[allow(deprecated)]
        let i80 = p1.haversine_intermediate(&p2, 0.8);
        let i20_should = Point::new(29.83519, 29.94841);
        let i50_should = Point::new(65.87471, 37.72201);
        let i80_should = Point::new(103.56036, 33.50518);
        assert_relative_eq!(i20.x(), i20_should.x(), epsilon = 0.2);
        assert_relative_eq!(i20.y(), i20_should.y(), epsilon = 0.2);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 0.2);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 0.2);
        assert_relative_eq!(i80.x(), i80_should.x(), epsilon = 0.2);
        assert_relative_eq!(i80.y(), i80_should.y(), epsilon = 0.2);
    }

    #[test]
    fn should_be_north_pole_test() {
        let p1 = Point::new(0.0, 10.0);
        let p2 = Point::new(180.0, 10.0);
        #[allow(deprecated)]
        let i50 = p1.haversine_intermediate(&p2, 0.5);
        let i50_should = Point::new(90.0, 90.0);
        assert_relative_eq!(i50.x(), i50_should.x(), epsilon = 1.0e-6);
        assert_relative_eq!(i50.y(), i50_should.y(), epsilon = 1.0e-6);
    }

    #[test]
    fn should_be_start_end_test() {
        let p1 = Point::new(30.0, 40.0);
        let p2 = Point::new(40.0, 50.0);
        let max_dist = 1500000.0; // meters
        #[allow(deprecated)]
        let route = p1.haversine_intermediate_fill(&p2, max_dist, true);
        assert_eq!(route, vec![p1, p2]);
    }

    #[test]
    fn should_add_i50_test() {
        let p1 = Point::new(30.0, 40.0);
        let p2 = Point::new(40.0, 50.0);
        let max_dist = 1000000.0; // meters
        #[allow(deprecated)]
        let i50 = p1.clone().haversine_intermediate(&p2, 0.5);
        #[allow(deprecated)]
        let fill = p1.haversine_intermediate_fill(&p2, max_dist, true);
        assert_eq!(fill, vec![p1, i50, p2]);
        #[allow(deprecated)]
        let fill = p1.haversine_intermediate_fill(&p2, max_dist, false);
        assert_eq!(fill, vec![i50]);
    }

    #[test]
    fn should_add_i25_i50_i75_test() {
        let p1 = Point::new(30.0, 40.0);
        let p2 = Point::new(40.0, 50.0);
        let max_dist = 400000.0; // meters
        #[allow(deprecated)]
        let i25 = p1.clone().haversine_intermediate(&p2, 0.25);
        #[allow(deprecated)]
        let i50 = p1.clone().haversine_intermediate(&p2, 0.5);
        #[allow(deprecated)]
        let i75 = p1.clone().haversine_intermediate(&p2, 0.75);
        #[allow(deprecated)]
        let route = p1.haversine_intermediate_fill(&p2, max_dist, true);
        assert_eq!(route, vec![p1, i25, i50, i75, p2]);
    }
}
