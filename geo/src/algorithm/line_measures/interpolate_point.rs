use crate::{CoordFloat, Point};

/// Interpolate a `Point` along a line between two existing points
pub trait InterpolatePoint<F: CoordFloat> {
    /// Returns a new Point along a line between two existing points.
    ///
    /// See [specific implementations](#implementors) for details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Euclidean, InterpolatePoint, Point};
    ///
    /// let p1: Point = Point::new(0.0, 0.0);
    /// let p2: Point = Point::new(0.0, 2.0);
    ///
    /// assert_relative_eq!(Euclidean.point_at_distance_between(p1, p2, 0.5), Point::new(0.0, 0.5));
    ///
    /// // The units of the argument depend on the metric space.
    /// // In the case of [`Haversine`], it's meters.
    /// // See the documentation for each metric space for details.
    /// assert_relative_eq!(Haversine.point_at_distance_between(p1, p2, 111_111.0), Point::new(0.0, 0.9992438493379715));
    /// ```
    fn point_at_distance_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        distance_from_start: F,
    ) -> Point<F>;

    /// Returns a new Point along a line between two existing points.
    ///
    /// See [specific implementations](#implementors) for details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Euclidean, InterpolatePoint, Point};
    /// let p1: Point = Point::new(0.0, 0.0);
    /// let p2: Point = Point::new(20.0, 20.0);
    ///
    /// assert_relative_eq!(Euclidean.point_at_ratio_between(p1, p2, 0.5), Point::new(10.0, 10.0));
    /// assert_relative_eq!(Haversine.point_at_ratio_between(p1, p2, 0.5), Point::new(9.685895184381804, 10.150932342575631));
    /// ```
    fn point_at_ratio_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        ratio_from_start: F,
    ) -> Point<F>;

    /// Interpolates `Point`s along a line between `start` and `end`.
    ///
    /// See [specific implementations](#implementors) for details.
    ///
    /// As many points as necessary will be added such that the distance between points
    /// never exceeds `max_distance`. If the distance between start and end is less than
    /// `max_distance`, no additional points will be included in the output.
    ///
    /// `include_ends`: Should the start and end points be included in the output?
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Euclidean, InterpolatePoint, Point, MultiPoint, LineString, wkt};
    /// let p1: Point = Point::new(0.0, 0.0);
    /// let p2: Point = Point::new(0.0, 2.0);
    ///
    /// let intermediate_points: Vec<Point> = Euclidean.points_along_line(p1, p2, 0.5, false).collect();
    /// let multi_point = MultiPoint(intermediate_points);
    /// assert_relative_eq!(multi_point, wkt!(MULTIPOINT(0. 0.5,0. 1.,0. 1.5)));
    ///
    /// // The units of the argument depend on the metric space.
    /// // In the case of [`Haversine`], it's meters.
    /// // See the documentation for each metric space for details.
    /// let intermediate_points: Vec<Point> = Haversine.points_along_line(p1, p2, 55_555.0, false).collect();
    /// let multi_point = MultiPoint(intermediate_points);
    /// assert_relative_eq!(multi_point, wkt!(MULTIPOINT(0. 0.4,0. 0.8,0. 1.2,0. 1.6)));
    /// ```
    fn points_along_line(
        &self,
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>>;
}

#[cfg(test)]
mod tests {
    use crate::{Euclidean, Geodesic, Haversine, InterpolatePoint, Point, Rhumb};

    #[test]
    fn point_at_ratio_between_line_ends() {
        let start = Point::new(0.0, 0.0);
        let end = Point::new(1.0, 1.0);

        let ratio = 0.0;
        assert_eq!(Haversine.point_at_ratio_between(start, end, ratio), start);
        assert_eq!(Euclidean.point_at_ratio_between(start, end, ratio), start);
        assert_eq!(Geodesic.point_at_ratio_between(start, end, ratio), start);
        assert_eq!(Rhumb.point_at_ratio_between(start, end, ratio), start);

        let ratio = 1.0;
        assert_eq!(Haversine.point_at_ratio_between(start, end, ratio), end);
        assert_eq!(Euclidean.point_at_ratio_between(start, end, ratio), end);
        assert_eq!(Geodesic.point_at_ratio_between(start, end, ratio), end);
        assert_eq!(Rhumb.point_at_ratio_between(start, end, ratio), end);
    }

    mod degenerate {
        use super::*;

        #[test]
        fn point_at_ratio_between_collapsed_line() {
            let start = Point::new(1.0, 1.0);

            let ratio = 0.0;
            assert_eq!(Haversine.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Euclidean.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Geodesic.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Rhumb.point_at_ratio_between(start, start, ratio), start);

            let ratio = 0.5;
            assert_eq!(Haversine.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Euclidean.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Geodesic.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Rhumb.point_at_ratio_between(start, start, ratio), start);

            let ratio = 1.0;
            assert_eq!(Haversine.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Euclidean.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Geodesic.point_at_ratio_between(start, start, ratio), start);
            assert_eq!(Rhumb.point_at_ratio_between(start, start, ratio), start);
        }

        #[test]
        fn point_at_distance_between_collapsed_line() {
            // This method just documents existing behavior. I don't think our current behavior
            // is especially useful, but we might consider handling it uniformly one day.
            let start: Point = Point::new(1.0, 1.0);

            let distance = 0.0;
            assert_eq!(
                Haversine.point_at_distance_between(start, start, distance),
                start
            );

            let euclidean_result = Euclidean.point_at_distance_between(start, start, distance);
            assert!(euclidean_result.x().is_nan());
            assert!(euclidean_result.y().is_nan());
            assert_eq!(
                Geodesic.point_at_distance_between(start, start, distance),
                start
            );
            assert_eq!(
                Rhumb.point_at_distance_between(start, start, distance),
                start
            );

            let distance = 100000.0;
            let due_north = Point::new(1.0, 1.9);
            let due_south = Point::new(1.0, 0.1);
            assert_relative_eq!(
                Haversine.point_at_distance_between(start, start, distance),
                due_north,
                epsilon = 1.0e-1
            );
            let euclidean_result = Euclidean.point_at_distance_between(start, start, distance);
            assert!(euclidean_result.x().is_nan());
            assert!(euclidean_result.y().is_nan());
            assert_relative_eq!(
                Geodesic.point_at_distance_between(start, start, distance),
                due_south,
                epsilon = 1.0e-1
            );
            assert_relative_eq!(
                Rhumb.point_at_distance_between(start, start, distance),
                due_north,
                epsilon = 1.0e-1
            );
        }

        #[test]
        fn points_along_collapsed_line() {
            let start = Point::new(1.0, 1.0);

            let max_distance = 1.0;

            let include_ends = true;
            let points: Vec<_> = Haversine
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![start, start]);

            let points: Vec<_> = Euclidean
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![start, start]);

            let points: Vec<_> = Geodesic
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![start, start]);

            let points: Vec<_> = Rhumb
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![start, start]);

            let include_ends = false;
            let points: Vec<_> = Haversine
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![]);

            let points: Vec<_> = Euclidean
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![]);

            let points: Vec<_> = Geodesic
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![]);

            let points: Vec<_> = Rhumb
                .points_along_line(start, start, max_distance, include_ends)
                .collect();
            assert_eq!(points, vec![]);
        }
    }
}
