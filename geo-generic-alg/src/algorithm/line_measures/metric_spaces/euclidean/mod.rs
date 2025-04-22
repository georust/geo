mod distance;

use super::super::{Distance, InterpolatePoint};
use crate::line_measures::densify::densify_between;
use crate::{CoordFloat, Point};
use num_traits::FromPrimitive;

/// Operations on the [Euclidean plane] measure distance with the pythagorean formula -
/// what you'd measure with a ruler.
///
/// If you have lon/lat points, use the [`Haversine`], [`Geodesic`], or other [metric spaces] -
/// Euclidean methods will give nonsense results.
///
/// If you wish to use Euclidean operations with lon/lat, the coordinates must first be transformed
/// using the [`Transform::transform`](crate::Transform::transform) / [`Transform::transform_crs_to_crs`](crate::Transform::transform_crs_to_crs) methods or their
/// immutable variants. Use of these requires the proj feature
///
/// [Euclidean plane]: https://en.wikipedia.org/wiki/Euclidean_plane
/// [`Transform`]: crate::Transform
/// [`Haversine`]: super::Haversine
/// [`Geodesic`]: super::Geodesic
/// [metric spaces]: super
pub struct Euclidean;

/// Interpolate Point(s) along a line on the [Euclidean plane].
///
/// [Euclidean plane]: https://en.wikipedia.org/wiki/Euclidean_plane
impl<F: CoordFloat + FromPrimitive> InterpolatePoint<F> for Euclidean {
    /// Returns the point at the given distance along the line between `start` and `end`.
    ///
    /// # Units
    /// - `distance`: Measured in whatever units your `start` and `end` points use.
    ///
    ///   `distance` and your `start` and `end` points should have non-angular
    ///   units, like meters or miles, **not** lon/lat.
    ///   For lon/lat points, use the [`Haversine`] or [`Geodesic`] [metric spaces].
    ///
    /// [`Haversine`]: crate::line_measures::Haversine
    /// [`Geodesic`]: crate::line_measures::Geodesic
    /// [metric spaces]: crate::line_measures::metric_spaces
    fn point_at_distance_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        distance_from_start: F,
    ) -> Point<F> {
        let diff = end - start;
        let total_distance = diff.x().hypot(diff.y());
        let offset = diff * distance_from_start / total_distance;
        start + offset
    }

    /// Returns the point at the given ratio along the line between `start` and `end`.
    ///
    /// # Units
    /// - `distance`: Measured in whatever units your `start` and `end` points use.
    ///
    ///   `distance` and your `start` and `end` points should have non-angular
    ///   units, like meters or miles, **not** lon/lat.
    ///   For lon/lat points, use the [`Haversine`] or [`Geodesic`] [metric spaces].
    ///
    /// [`Haversine`]: crate::line_measures::Haversine
    /// [`Geodesic`]: crate::line_measures::Geodesic
    /// [metric spaces]: crate::line_measures::metric_spaces
    fn point_at_ratio_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        ratio_from_start: F,
    ) -> Point<F> {
        let diff = end - start;
        start + diff * ratio_from_start
    }

    /// Interpolates `Point`s along a line between `start` and `end`.
    ///
    /// As many points as necessary will be added such that the distance between points
    /// never exceeds `max_distance`. If the distance between start and end is less than
    /// `max_distance`, no additional points will be included in the output.
    ///
    /// `include_ends`: Should the start and end points be included in the output?
    ///
    /// # Units
    /// - `max_distance`: Measured in whatever units your `start` and `end` points use.
    ///
    ///   `max_distance` and your `start` and `end` points should have non-angular
    ///   units, like meters or miles, **not** lon/lat.
    ///   For lon/lat points, use the [`Haversine`] or [`Geodesic`] [metric spaces].
    ///
    /// [`Haversine`]: crate::line_measures::Haversine
    /// [`Geodesic`]: crate::line_measures::Geodesic
    /// [metric spaces]: crate::line_measures::metric_spaces
    fn points_along_line(
        &self,
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>> {
        let mut container = vec![];
        if include_ends {
            container.push(start);
        }
        densify_between(self, start, end, &mut container, max_distance);
        if include_ends {
            container.push(end);
        }
        container.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod distance {
        use super::*;

        #[test]
        fn new_york_to_london() {
            // web mercator
            let new_york_city = Point::new(-8238310.24, 4942194.78);
            // web mercator
            let london = Point::new(-14226.63, 6678077.70);
            let distance: f64 = Euclidean.distance(new_york_city, london);

            assert_relative_eq!(
                8_405_286., // meters in web mercator
                distance.round()
            );
        }

        #[test]
        fn test_point_at_distance_between() {
            let new_york_city = Point::new(-8_238_310.24, 4_942_194.78);
            // web mercator
            let london = Point::new(-14_226.63, 6_678_077.70);
            let start = Euclidean.point_at_distance_between(new_york_city, london, 0.0);
            assert_relative_eq!(new_york_city, start);

            let midway =
                Euclidean.point_at_distance_between(new_york_city, london, 8_405_286.0 / 2.0);
            assert_relative_eq!(Point::new(-4_126_268., 5_810_136.), midway, epsilon = 1.0);

            let end = Euclidean.point_at_distance_between(new_york_city, london, 8_405_286.0);
            assert_relative_eq!(london, end, epsilon = 1.0);
        }
    }
}
