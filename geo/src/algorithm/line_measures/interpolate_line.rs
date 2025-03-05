//! Interpolate a Point part way into a linear geometry (1-D).

use super::{InterpolatePoint, Length};
use geo_types::{CoordFloat, Line, LineString, Point};

/// Interpolate a `Point` along a `Line` or `LineString`.
///
/// Related: See [`Densify`](crate::Densify) if you'd like to interpolate potentially many points into a geometry.
pub trait InterpolateLine<F: CoordFloat>: InterpolatePoint<F> + Length<F> + Sized {
    /// Returns a new point part way down the line.
    ///
    /// # Params
    ///
    /// - `line`: A `Line` or `LineString` which implements `InterpolatableLine`.
    /// - `ratio`: the ratio of the total line length. It will be bounded between 0..1.
    ///   - `0.0` will return the start of the line.
    ///   - `1.0` will return the end of the line.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::{Haversine, Euclidean, InterpolateLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    /// let quarter_distance = Euclidean.point_at_ratio_from_start(&line_string, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(0. 5.)));
    ///
    /// let quarter_distance = Haversine.point_at_ratio_from_start(&line_string, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(0. 4.961924877405399)), epsilon=1e-14);
    /// ```
    fn point_at_ratio_from_start<L: InterpolatableLine<F>>(&self, line: &L, ratio: F) -> L::Output {
        line.point_at_ratio_from_start(self, ratio)
    }

    /// Returns a new point part way down the line, starting from the end of the line.
    ///
    /// # Params
    ///
    /// - `line`: A `Line` or `LineString` which implements `InterpolatableLine`.
    /// - `ratio`: the ratio of the total line length. It will be bounded between 0..1.
    ///   - `0.0` will return the end of the line.
    ///   - `1.0` will return the start of the line.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::{Haversine, Euclidean, InterpolateLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    /// let quarter_distance = Euclidean.point_at_ratio_from_end(&line_string, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(5. 10.0)));
    ///
    /// let quarter_distance = Haversine.point_at_ratio_from_end(&line_string, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(4.961333045966285 10.037420806650719)), epsilon=1e-14);
    /// ```
    fn point_at_ratio_from_end<L: InterpolatableLine<F>>(&self, line: &L, ratio: F) -> L::Output {
        line.point_at_ratio_from_end(self, ratio)
    }

    /// Returns a new point `distance` from the start of the line.
    ///
    /// # Params
    ///
    /// - `line`: A `Line` or `LineString` which implements `InterpolatableLine`.
    /// - `distance`: How far down the line. The units of distance depend on the metric space.
    ///     Distance will be clamped so that the returned point will not be outside of `line`.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::{Haversine, Euclidean, InterpolateLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    ///
    /// // For Euclidean calculations, distance is in the same units as your points
    /// let near_start = Euclidean.point_at_distance_from_start(&line_string, 0.5).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(0. 0.5)));
    ///
    /// // For Haversine calculations, distance is in meters
    /// let near_start = Haversine.point_at_distance_from_start(&line_string, 100_000.0).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(0. 0.899320363724538)), epsilon=1e-14);
    /// ```
    fn point_at_distance_from_start<L: InterpolatableLine<F>>(
        &self,
        line: &L,
        distance: F,
    ) -> L::Output {
        line.point_at_distance_from_start(self, distance)
    }

    /// Returns a new point `distance` from the end of the line.
    ///
    /// # Params
    ///
    /// - `line`: A `Line` or `LineString` which implements `InterpolatableLine`.
    /// - `distance`: How far down the line. The units of distance depend on the metric space.
    ///     Distance will be clamped so that the returned point will not be outside of `line`.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::{Haversine, Euclidean, InterpolateLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    ///
    /// // For Euclidean calculations, distance is in the same units as your points
    /// let near_start = Euclidean.point_at_distance_from_end(&line_string, 0.5).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(9.5 10.)));
    ///
    /// // For Haversine calculations, distance is in meters
    /// let near_start = Haversine.point_at_distance_from_end(&line_string, 100_000.0).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(9.086875463645015 10.012416322308656)), epsilon=1e-14);
    /// ```
    fn point_at_distance_from_end<L: InterpolatableLine<F>>(
        &self,
        line: &L,
        distance: F,
    ) -> L::Output {
        line.point_at_distance_from_end(self, distance)
    }
}

impl<F, MetricSpace> InterpolateLine<F> for MetricSpace
where
    F: CoordFloat,
    MetricSpace: InterpolatePoint<F> + Length<F> + Sized,
{
}

/// A linear geometry (1-D) which can have a point interpolated partially into it.
///
/// Related: See [`Densify`](crate::Densify) if you'd like to interpolate potentially many points into a geometry.
pub trait InterpolatableLine<F: CoordFloat> {
    type Output;

    /// Returns a new point part way down the line.
    ///
    /// # Params
    ///
    /// - `metric_space`: e.g. [`Euclidean`], [`Haversine`], or [`Geodesic`]. See [`metric_spaces`]
    /// - `ratio`: the ratio of the total line length. It will be bounded between 0..1.
    ///   - `0.0` will return the start of the line.
    ///   - `1.0` will return the end of the line.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::line_measures::{Haversine, Euclidean, InterpolatableLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    /// let quarter_distance = line_string.point_at_ratio_from_start(&Euclidean, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(0. 5.)));
    ///
    /// let quarter_distance = line_string.point_at_ratio_from_start(&Haversine, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(0. 4.961924877405399)), epsilon=1e-14);
    /// ```
    ///
    /// [`Euclidean`]: super::Euclidean
    /// [`Haversine`]: super::Haversine
    /// [`Geodesic`]: super::Geodesic
    /// [`metric_spaces`]: super::metric_spaces
    fn point_at_ratio_from_start<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        ratio: F,
    ) -> Self::Output;

    /// Returns a new point part way down the line, starting from the end of the line.
    ///
    /// # Params
    ///
    /// - `metric_space`: e.g. [`Euclidean`], [`Haversine`], or [`Geodesic`]. See [`metric_spaces`]
    /// - `ratio`: the ratio of the total line length. It will be bounded between 0..1.
    ///   - `0.0` will return the end of the line.
    ///   - `1.0` will return the start of the line.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::line_measures::{Haversine, Euclidean, InterpolatableLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    /// let quarter_distance = line_string.point_at_ratio_from_end(&Euclidean, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(5. 10.0)));
    ///
    /// let quarter_distance = line_string.point_at_ratio_from_end(&Haversine, 0.25).unwrap();
    /// assert_relative_eq!(quarter_distance, wkt!(POINT(4.961333045966285 10.037420806650719)), epsilon=1e-14);
    /// ```
    ///
    /// [`Euclidean`]: super::Euclidean
    /// [`Haversine`]: super::Haversine
    /// [`Geodesic`]: super::Geodesic
    /// [`metric_spaces`]: super::metric_spaces
    fn point_at_ratio_from_end<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        ratio: F,
    ) -> Self::Output;

    /// Returns a new point `distance` from the start of the line.
    ///
    /// # Params
    ///
    /// - `metric_space`: e.g. [`Euclidean`], [`Haversine`], or [`Geodesic`]. See [`metric_spaces`]
    /// - `distance`: How far down the line. The units of distance depend on the metric space.
    ///     Distance will be clamped so that the returned point will not be outside of `line`.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::line_measures::{Haversine, Euclidean, InterpolatableLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    ///
    /// // For Euclidean calculations, distance is in the same units as your points
    /// let near_start = line_string.point_at_distance_from_start(&Euclidean, 0.5).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(0. 0.5)));
    ///
    /// // For Haversine calculations, distance is in meters
    /// let near_start = line_string.point_at_distance_from_start(&Haversine, 100_000.0).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(0. 0.899320363724538)), epsilon=1e-14);
    /// ```
    ///
    /// [`Euclidean`]: super::Euclidean
    /// [`Haversine`]: super::Haversine
    /// [`Geodesic`]: super::Geodesic
    /// [`metric_spaces`]: super::metric_spaces
    fn point_at_distance_from_start<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        distance: F,
    ) -> Self::Output;

    /// Returns a new point `distance` from the end of the line.
    ///
    /// # Params
    ///
    /// - `metric_space`: e.g. [`Euclidean`], [`Haversine`], or [`Geodesic`]. See [`metric_spaces`]
    /// - `distance`: How far down the line. The units of distance depend on the metric space.
    ///     Distance will be clamped so that the returned point will not be outside of `line`.
    ///
    /// # Example
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::algorithm::line_measures::{Haversine, Euclidean, InterpolatableLine};
    /// use geo::wkt;
    ///
    /// let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
    ///
    /// // For Euclidean calculations, distance is in the same units as your points
    /// let near_start = line_string.point_at_distance_from_end(&Euclidean, 0.5).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(9.5 10.)));
    ///
    /// // For Haversine calculations, distance is in meters
    /// let near_start = line_string.point_at_distance_from_end(&Haversine, 100_000.0).unwrap();
    /// assert_relative_eq!(near_start, wkt!(POINT(9.086875463645015 10.012416322308656)), epsilon=1e-14);
    /// ```
    ///
    /// [`Euclidean`]: super::Euclidean
    /// [`Haversine`]: super::Haversine
    /// [`Geodesic`]: super::Geodesic
    /// [`metric_spaces`]: super::metric_spaces
    fn point_at_distance_from_end<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        distance: F,
    ) -> Self::Output;
}

impl<F: CoordFloat> InterpolatableLine<F> for Line<F> {
    type Output = Point<F>;

    fn point_at_ratio_from_start<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        ratio: F,
    ) -> Self::Output {
        if ratio <= F::zero() {
            self.start_point()
        } else if ratio >= F::one() {
            self.end_point()
        } else {
            metric_space.point_at_ratio_between(self.start_point(), self.end_point(), ratio)
        }
    }

    fn point_at_ratio_from_end<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        ratio: F,
    ) -> Self::Output {
        if ratio <= F::zero() {
            self.end_point()
        } else if ratio >= F::one() {
            self.start_point()
        } else {
            metric_space.point_at_ratio_between(self.end_point(), self.start_point(), ratio)
        }
    }

    fn point_at_distance_from_start<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        distance: F,
    ) -> Self::Output {
        if distance <= F::zero() {
            self.start_point()
        } else if distance >= metric_space.length(self) {
            self.end_point()
        } else {
            metric_space.point_at_distance_between(self.start_point(), self.end_point(), distance)
        }
    }

    fn point_at_distance_from_end<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        distance: F,
    ) -> Self::Output {
        if distance <= F::zero() {
            self.end_point()
        } else if distance >= metric_space.length(self) {
            self.start_point()
        } else {
            metric_space.point_at_distance_between(self.end_point(), self.start_point(), distance)
        }
    }
}

impl<F: CoordFloat> InterpolatableLine<F> for LineString<F> {
    type Output = Option<Point<F>>;

    fn point_at_ratio_from_start<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        ratio: F,
    ) -> Self::Output {
        let distance = ratio * metric_space.length(self);
        self.point_at_distance_from_start(metric_space, distance)
    }

    fn point_at_ratio_from_end<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        ratio: F,
    ) -> Self::Output {
        let distance = ratio * metric_space.length(self);
        self.point_at_distance_from_end(metric_space, distance)
    }

    fn point_at_distance_from_start<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        distance: F,
    ) -> Self::Output {
        if distance <= F::zero() {
            return self.0.first().map(|coord| Point(*coord));
        }
        let mut distance_remaining = distance;
        for segment in self.lines() {
            let segment_length = metric_space.length(&segment);
            if segment_length < distance_remaining {
                distance_remaining = distance_remaining - segment_length;
            } else {
                return Some(metric_space.point_at_distance_between(
                    segment.start_point(),
                    segment.end_point(),
                    distance_remaining,
                ));
            }
        }
        // distance >= self.length, so return the final point.
        self.0.last().map(|coord| Point(*coord))
    }

    fn point_at_distance_from_end<MetricSpace: InterpolatePoint<F> + Length<F>>(
        &self,
        metric_space: &MetricSpace,
        distance: F,
    ) -> Self::Output {
        if distance <= F::zero() {
            return self.0.last().map(|coord| Point(*coord));
        }
        let mut distance_remaining = distance;
        for reversed_segment in self.rev_lines() {
            let segment_length = metric_space.length(&reversed_segment);
            if segment_length < distance_remaining {
                distance_remaining = distance_remaining - segment_length;
            } else {
                // To measure from the *end* of the line,
                // we measure from the *start* of the  reversed segment.
                return Some(metric_space.point_at_distance_between(
                    reversed_segment.start_point(),
                    reversed_segment.end_point(),
                    distance_remaining,
                ));
            }
        }
        // distance_from_start >= self.length, so return the final point.
        self.0.first().map(|coord| Point(*coord))
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::line_measures::{Euclidean, InterpolateLine};
    use crate::geometry::{Line, LineString, Point};
    use crate::wkt;

    mod line {
        use super::*;

        #[test]
        fn point_at_ratio_from_start() {
            let line = Line::new((0., 0.), (0., 10.));
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line, 0.0),
                Point::new(0., 0.)
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line, 0.5),
                Point::new(0., 5.)
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line, 1.0),
                Point::new(0., 10.)
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line, 1.5),
                Point::new(0., 10.)
            );
        }

        #[test]
        fn point_at_ratio_from_end() {
            let line = Line::new((0., 0.), (0., 10.));
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line, 0.0),
                Point::new(0., 10.)
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line, 0.5),
                Point::new(0., 5.)
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line, 1.0),
                Point::new(0., 0.)
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line, 1.5),
                Point::new(0., 0.)
            );
        }

        #[test]
        fn point_at_distance_from_start() {
            let line = Line::new((0., 0.), (0., 10.));
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line, 0.0),
                Point::new(0., 0.)
            );
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line, 5.0),
                Point::new(0., 5.)
            );
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line, 10.0),
                Point::new(0., 10.)
            );

            // beyond end
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line, 100.0),
                Point::new(0., 10.)
            );

            // before start
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line, -5.0),
                Point::new(0., 0.)
            );
        }

        #[test]
        fn point_at_distance_from_end() {
            let line = Line::new((0., 0.), (0., 10.));
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line, 0.0),
                Point::new(0., 10.)
            );
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line, 5.0),
                Point::new(0., 5.)
            );
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line, 10.0),
                Point::new(0., 0.)
            );

            // beyond start
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line, 100.0),
                Point::new(0., 0.)
            );

            // before end
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line, -5.0),
                Point::new(0., 10.)
            );
        }
    }

    mod line_string {
        use super::*;

        #[test]
        fn point_at_ratio_from_start() {
            let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line_string, 0.0),
                Some(Point::new(0., 0.))
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line_string, 0.5),
                Some(Point::new(0., 10.))
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line_string, 1.0),
                Some(Point::new(10., 10.))
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_start(&line_string, 1.5),
                Some(Point::new(10., 10.))
            );
        }

        #[test]
        fn point_at_ratio_from_end() {
            let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line_string, 0.0),
                Some(Point::new(10., 10.))
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line_string, 0.5),
                Some(Point::new(0., 10.))
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line_string, 1.0),
                Some(Point::new(0., 0.))
            );
            assert_eq!(
                Euclidean.point_at_ratio_from_end(&line_string, 1.5),
                Some(Point::new(0., 0.))
            );
        }

        #[test]
        fn point_at_distance_from_start() {
            let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line_string, 0.0),
                Some(Point::new(0., 0.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line_string, 5.0),
                Some(Point::new(0., 5.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line_string, 10.0),
                Some(Point::new(0., 10.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line_string, 15.0),
                Some(Point::new(5., 10.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line_string, 20.0),
                Some(Point::new(10., 10.))
            );

            // beyond end
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line_string, 100.0),
                Some(Point::new(10., 10.))
            );

            // before start
            assert_eq!(
                Euclidean.point_at_distance_from_start(&line_string, -5.0),
                Some(Point::new(0., 0.))
            );
        }

        #[test]
        fn point_at_distance_from_end() {
            let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line_string, 0.0),
                Some(Point::new(10., 10.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line_string, 5.0),
                Some(Point::new(5., 10.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line_string, 10.0),
                Some(Point::new(0., 10.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line_string, 15.0),
                Some(Point::new(0., 5.))
            );
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line_string, 20.0),
                Some(Point::new(0., 0.))
            );

            // beyond start
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line_string, 100.0),
                Some(Point::new(0., 0.))
            );

            // before end
            assert_eq!(
                Euclidean.point_at_distance_from_end(&line_string, -5.0),
                Some(Point::new(10., 10.))
            );
        }

        mod haversine {
            use super::*;
            use crate::Haversine;

            #[test]
            fn ratio_from_start() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Haversine
                    .point_at_ratio_from_start(&line_string, 0.25)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(0.0 4.961924877405399)),
                    epsilon = 1e-14
                );
            }

            #[test]
            fn ratio_from_end() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Haversine
                    .point_at_ratio_from_end(&line_string, 0.25)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(4.961333045966285 10.037420806650719)),
                    epsilon = 1e-14
                );
            }

            #[test]
            fn distance_from_start() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Haversine
                    .point_at_distance_from_start(&line_string, 100_000.)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(0.0 0.899320363724538)),
                    epsilon = 1e-14
                );
            }

            #[test]
            fn distance_from_end() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Haversine
                    .point_at_distance_from_end(&line_string, 100_000.)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(9.086875463645015 10.012416322308656)),
                    epsilon = 1e-14
                );
            }
        }

        mod geodesic {
            use super::*;
            use crate::Geodesic;

            #[test]
            fn ratio_from_start() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Geodesic
                    .point_at_ratio_from_start(&line_string, 0.25)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(0.0 4.9788949389766595)),
                    epsilon = 1e-14
                );
            }

            #[test]
            fn ratio_from_end() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Geodesic
                    .point_at_ratio_from_end(&line_string, 0.25)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(4.97832809547093 10.037667662355751)),
                    epsilon = 1e-14
                );
            }

            #[test]
            fn distance_from_start() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Geodesic
                    .point_at_distance_from_start(&line_string, 100_000.)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(0.0 0.9043687229127633)),
                    epsilon = 1e-14
                );
            }

            #[test]
            fn distance_from_end() {
                let line_string = wkt!(LINESTRING(0. 0., 0. 10.,10. 10.));
                let quarter_distance = Geodesic
                    .point_at_distance_from_end(&line_string, 100_000.)
                    .unwrap();
                assert_relative_eq!(
                    quarter_distance,
                    wkt!(POINT(9.087988077970042 10.012483990563286)),
                    epsilon = 1e-14
                );
            }
        }

        mod degenerate {
            use super::*;

            #[test]
            fn empty_line_string() {
                let line_string: LineString = LineString::new(vec![]);
                assert_eq!(None, Euclidean.point_at_ratio_from_start(&line_string, 0.5));
                assert_eq!(
                    None,
                    Euclidean.point_at_distance_from_start(&line_string, 0.5)
                );
            }

            #[test]
            fn line_string_of_1() {
                let point = Point::new(1., 1.);
                let line_string = LineString::new(vec![point.0]);
                assert_eq!(
                    Some(point),
                    Euclidean.point_at_ratio_from_start(&line_string, 0.5)
                );

                assert_eq!(
                    Some(point),
                    Euclidean.point_at_distance_from_start(&line_string, 0.5)
                );
            }
        }
    }
}
