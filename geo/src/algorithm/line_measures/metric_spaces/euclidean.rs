use super::super::Distance;
use crate::{GeoFloat, Point};

/// Euclidean space measures distance with the pythagorean formula - what you'd measure with a ruler.
///
/// You must use projected coordinates with Euclidean space —
/// for lon/lat points, use the [`Haversine`], [`Geodesic`], or other [metric spaces].
///
/// [`Haversine`]: super::Haversine
/// [`Geodesic`]: super::Geodesic
/// [metric spaces]: super
pub struct Euclidean;

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
