use crate::{CoordFloat, Geodesic, InterpolatePoint, Point};

#[deprecated(
    since = "0.29.0",
    note = "Please use the `InterpolatePoint` trait instead"
)]
/// Returns a new Point along a route between two existing points on an ellipsoidal model of the earth
pub trait GeodesicIntermediate<T: CoordFloat> {
    #[deprecated(
        since = "0.29.0",
        note = "Please use `Geodesic.point_at_ratio_between` from the `InterpolatePoint` trait instead"
    )]
    /// Returns a new Point along a route between two existing points on an ellipsoidal model of the earth
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// # #[allow(deprecated)]
    /// use geo::GeodesicIntermediate;
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    /// # #[allow(deprecated)]
    /// let i20 = p1.geodesic_intermediate(&p2, 0.2);
    /// # #[allow(deprecated)]
    /// let i50 = p1.geodesic_intermediate(&p2, 0.5);
    /// # #[allow(deprecated)]
    /// let i80 = p1.geodesic_intermediate(&p2, 0.8);
    /// let i20_should = Point::new(29.842907, 29.951445);
    /// let i50_should = Point::new(65.879360, 37.722253);
    /// let i80_should = Point::new(103.556796, 33.506196);
    /// assert_relative_eq!(i20, i20_should, epsilon = 1.0e-6);
    /// assert_relative_eq!(i50, i50_should, epsilon = 1.0e-6);
    /// assert_relative_eq!(i80, i80_should, epsilon = 1.0e-6);
    /// ```
    fn geodesic_intermediate(&self, other: &Point<T>, f: T) -> Point<T>;

    #[deprecated(
        since = "0.29.0",
        note = "Please use `Geodesic.points_along_line` from the `InterpolatePoint` trait instead"
    )]
    fn geodesic_intermediate_fill(
        &self,
        other: &Point<T>,
        max_dist: T,
        include_ends: bool,
    ) -> Vec<Point<T>>;
}

#[allow(deprecated)]
impl GeodesicIntermediate<f64> for Point {
    fn geodesic_intermediate(&self, other: &Point, f: f64) -> Point {
        Geodesic.point_at_ratio_between(*self, *other, f)
    }

    fn geodesic_intermediate_fill(
        &self,
        other: &Point,
        max_dist: f64,
        include_ends: bool,
    ) -> Vec<Point> {
        Geodesic
            .points_along_line(*self, *other, max_dist, include_ends)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn f_is_zero_or_one_test() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(15.0, 25.0);
        #[allow(deprecated)]
        let i0 = p1.geodesic_intermediate(&p2, 0.0);
        #[allow(deprecated)]
        let i100 = p1.geodesic_intermediate(&p2, 1.0);
        assert_relative_eq!(i0, p1, epsilon = 1.0e-6);
        assert_relative_eq!(i100, p2, epsilon = 1.0e-6);
    }

    #[test]
    fn various_f_values_test() {
        let p1 = Point::new(10.0, 20.0);
        let p2 = Point::new(125.0, 25.0);
        #[allow(deprecated)]
        let i20 = p1.geodesic_intermediate(&p2, 0.2);
        #[allow(deprecated)]
        let i50 = p1.geodesic_intermediate(&p2, 0.5);
        #[allow(deprecated)]
        let i80 = p1.geodesic_intermediate(&p2, 0.8);
        let i20_should = Point::new(29.842907, 29.951445);
        let i50_should = Point::new(65.879360, 37.722253);
        let i80_should = Point::new(103.556796, 33.506196);
        assert_relative_eq!(i20, i20_should, epsilon = 1.0e-6);
        assert_relative_eq!(i50, i50_should, epsilon = 1.0e-6);
        assert_relative_eq!(i80, i80_should, epsilon = 1.0e-6);
    }

    #[test]
    fn should_add_i50_test() {
        let p1 = Point::new(30.0, 40.0);
        let p2 = Point::new(40.0, 50.0);
        let max_dist = 1000000.0; // meters
        let include_ends = true;
        #[allow(deprecated)]
        let i50 = p1.geodesic_intermediate(&p2, 0.5);
        #[allow(deprecated)]
        let route = p1.geodesic_intermediate_fill(&p2, max_dist, include_ends);
        assert_eq!(route, vec![p1, i50, p2]);
    }
}
