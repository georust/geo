use super::super::{Distance, InterpolatePoint};
use crate::{GeoFloat, Point};

/// Operations on the [Euclidean plane] measure distance with the pythagorean formula -
/// what you'd measure with a ruler.
///
/// If you have lon/lat points, use the [`Haversine`], [`Geodesic`], or other [metric spaces] -
/// Euclidean methods will give nonsense results.
///
/// Alternatively, you *can* use lon/lat points with Euclidean methods if you first [`Transform`]
/// your points to an appropriate projection.
///
/// [Euclidean plane]: https://en.wikipedia.org/wiki/Euclidean_plane
/// [`Transform`]: crate::Transform
/// [`Haversine`]: super::Haversine
/// [`Geodesic`]: super::Geodesic
/// [metric spaces]: super
pub struct Euclidean;

// NBD - but I think this GeoFloat could be CoordFloat with a little work.
/// Calculate the Euclidean distance (a.k.a. pythagorean distance) between two Points
impl<F: GeoFloat> Distance<F, Point<F>, Point<F>> for Euclidean {
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
        crate::EuclideanDistance::euclidean_distance(&origin, &destination)
    }
}

// NBD - but I think this GeoFloat could be CoordFloat with a little work.
impl<F: GeoFloat> InterpolatePoint<F> for Euclidean {
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
        Self::densify_line(start, end, &mut container, max_distance, include_ends);
        container.into_iter()
    }
}

impl Euclidean {
    // TODO: This method could be moved to InterpolatePoint+Distance and apply to all MetricSpaces
    // InterpolatePoints::points_along_line and Densify::densify have some overlap... I need to
    // think more about how to organize these methods.
    fn densify_line<F: GeoFloat>(
        start: Point<F>,
        end: Point<F>,
        container: &mut Vec<Point<F>>,
        max_segment_length: F,
        include_ends: bool,
    ) {
        assert!(max_segment_length > F::zero());

        if include_ends {
            container.push(start);
        }

        let num_segments = (Self::distance(start, end) / max_segment_length)
            .ceil()
            .to_u64()
            .expect("unreasonable number of segments");

        // distance "unit" for this line segment
        let frac = F::one() / F::from(num_segments).unwrap();

        for segment_idx in 1..num_segments {
            let ratio = frac * F::from(segment_idx).unwrap();
            let start = start;
            let end = end;

            let interpolated_point = Self::point_at_ratio_between(start.into(), end.into(), ratio);
            container.push(interpolated_point);
        }
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
