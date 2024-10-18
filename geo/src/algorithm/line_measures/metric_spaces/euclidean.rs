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

/// Calculate the Euclidean distance (a.k.a. pythagorean distance) between two Points
impl<F: CoordFloat> Distance<F, Point<F>, Point<F>> for Euclidean {
    /// Calculate the Euclidean distance (a.k.a. pythagorean distance) between two Points
    ///
    /// # Units
    /// - `origin`, `destination`: Point where the units of x/y represent non-angular units
    ///    — e.g. meters or miles, not lon/lat. For lon/lat points, use the
    ///    [`Haversine`] or [`Geodesic`] [metric spaces].
    /// - returns: distance in the same units as the `origin` and `destination` points
    ///
    /// # Example
    /// ```
    /// use geo::{Euclidean, Distance};
    /// use geo::Point;
    /// // web mercator
    /// let new_york_city = Point::new(-8238310.24, 4942194.78);
    /// // web mercator
    /// let london = Point::new(-14226.63, 6678077.70);
    /// let distance: f64 = Euclidean::distance(new_york_city, london);
    ///
    /// assert_eq!(
    ///     8_405_286., // meters in web mercator
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [`Haversine`]: super::Haversine
    /// [`Geodesic`]: super::Geodesic
    /// [metric spaces]: super
    fn distance(origin: Point<F>, destination: Point<F>) -> F {
        let delta = origin - destination;
        delta.x().hypot(delta.y())
    }
}

impl<F: CoordFloat + FromPrimitive> InterpolatePoint<F> for Euclidean {
    fn point_at_ratio_between(start: Point<F>, end: Point<F>, ratio_from_start: F) -> Point<F> {
        let diff = end - start;
        start + diff * ratio_from_start
    }

    fn points_along_line(
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>> {
        let mut container = vec![];
        if include_ends {
            container.push(start);
        }
        densify_between::<F, Self>(start, end, &mut container, max_distance);
        if include_ends {
            container.push(end);
        }
        container.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MetricSpace = Euclidean;

    mod distance {
        use super::*;

        #[test]
        fn new_york_to_london() {
            // web mercator
            let new_york_city = Point::new(-8238310.24, 4942194.78);
            // web mercator
            let london = Point::new(-14226.63, 6678077.70);
            let distance: f64 = MetricSpace::distance(new_york_city, london);

            assert_relative_eq!(
                8_405_286., // meters in web mercator
                distance.round()
            );
        }
    }
}
