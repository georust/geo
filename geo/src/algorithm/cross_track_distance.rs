use crate::Distance;
use geo_types::{CoordFloat, Line, Point};
use num_traits::FromPrimitive;

/// Determine the cross track distance (also known as the cross track error) which is the shortest
/// distance between a point and a continuous line.
pub trait CrossTrackDistance<T, Rhs = Self>
where
    T: CoordFloat + FromPrimitive,
{
    /// Determine the cross track distance between this point and a line
    /// which passes through line_point_a and line_point_b
    ///
    /// # Units
    ///
    /// - return value: meters
    ///
    /// # Example
    ///
    /// ```rust
    /// use geo::prelude::*;
    /// use geo::point;
    ///
    /// // New York City
    /// let p1 = point!(x: -74.006f64, y: 40.7128f64);
    ///
    /// // Miami
    /// let line_point_a = point!(x: -80.1918f64, y: 25.7617f64);
    ///
    /// // Washington
    /// let line_point_b = point!(x: -120.7401, y: 47.7511f64);
    ///
    /// let distance = p1.cross_track_distance(&line_point_a, &line_point_b);
    ///
    /// assert_eq!(
    ///     1_547_104., // meters
    ///     distance.round()
    /// );
    /// ```
    fn cross_track_distance<MetricSpace>(&self, line_point_a: &Rhs, line_point_b: &Rhs) -> T
    where
        MetricSpace: for<'a> Distance<T, &'a Line<T>, &'a Point<T>>;
}

impl<T> CrossTrackDistance<T, Point<T>> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn cross_track_distance<MetricSpace>(
        &self,
        line_point_a: &Point<T>,
        line_point_b: &Point<T>,
    ) -> T
    where
        MetricSpace: for<'a> Distance<T, &'a Line<T>, &'a Point<T>>,
    {
        MetricSpace::distance(&Line::new(*line_point_a, *line_point_b), self)
    }
}

#[cfg(test)]
mod test {
    use crate::Point;
    use crate::{CrossTrackDistance, Distance, Euclidean, Haversine};

    #[test]
    fn distance1_test() {
        let p = Point::new(-0.7972, 53.2611);
        let line_point_a = Point::new(-1.7297, 53.3206);
        let line_point_b = Point::new(0.1334, 53.1887);
        assert_relative_eq!(
            p.cross_track_distance::<Haversine>(&line_point_a, &line_point_b),
            307.549995,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn cross_track_distance_to_line_passing_through_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(1., 0.);
        let line_point_b = Point::new(2., 0.);

        assert_relative_eq!(
            p.cross_track_distance::<Haversine>(&line_point_a, &line_point_b),
            0.,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn cross_track_distance_to_line_orthogonal_to_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(1., -1.);
        let line_point_b = Point::new(1., 1.);

        assert_relative_eq!(
            p.cross_track_distance::<Haversine>(&line_point_a, &line_point_b),
            Haversine::distance(p, Point::new(1., 0.)),
            epsilon = 1.0e-6
        );

        assert_relative_eq!(
            p.cross_track_distance::<Haversine>(&line_point_b, &line_point_a),
            Haversine::distance(p, Point::new(1., 0.)),
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn new_york_to_line_between_miami_and_washington() {
        let p1 = Point::new(-74.006f64, 40.7128f64);
        let line_point_a = Point::new(-80.1918f64, 25.7617f64);
        let line_point_b = Point::new(-120.7401f64, 47.7511f64);

        let haversine_distance = p1.cross_track_distance::<Haversine>(&line_point_a, &line_point_b);
        assert_relative_eq!(haversine_distance, 1_547_104., epsilon = 1.0);

        // Same as above, but projected to EPSG:5070 to test Euclidean
        let p1 = Point::new(1826303.9422258963, 2179112.0732980534);
        let line_point_a = Point::new(1594077.349564382, 434470.0414052719);
        let line_point_b = Point::new(-1847681.2454118263, 2992574.0850278544);

        let euclidean_distance = p1.cross_track_distance::<Euclidean>(&line_point_a, &line_point_b);
        assert_relative_eq!(euclidean_distance, 1_538_764., epsilon = 1.0);
    }
}
