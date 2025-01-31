use crate::{Bearing, Distance, Haversine, MEAN_EARTH_RADIUS};
use geo_types::{CoordFloat, Point};
use num_traits::FromPrimitive;

/// Determine the cross track distance (also known as the cross track error) which is the shortest
/// distance between a point and a continuous line.
pub trait CrossTrackDistance<T, Rhs = Self> {
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
    fn cross_track_distance(&self, line_point_a: &Rhs, line_point_b: &Rhs) -> T;
}

impl<T> CrossTrackDistance<T, Point<T>> for Point<T>
where
    T: CoordFloat + FromPrimitive,
{
    fn cross_track_distance(&self, line_point_a: &Point<T>, line_point_b: &Point<T>) -> T {
        let mean_earth_radius = T::from(MEAN_EARTH_RADIUS).unwrap();
        let l_delta_13: T = Haversine.distance(*line_point_a, *self) / mean_earth_radius;
        let theta_13: T = Haversine.bearing(*line_point_a, *self).to_radians();
        let theta_12: T = Haversine.bearing(*line_point_a, *line_point_b).to_radians();
        let l_delta_xt: T = (l_delta_13.sin() * (theta_12 - theta_13).sin()).asin();
        mean_earth_radius * l_delta_xt.abs()
    }
}

#[cfg(test)]
mod test {
    use crate::CrossTrackDistance;
    use crate::Point;
    use crate::{Distance, Haversine};

    #[test]
    fn distance1_test() {
        let p = Point::new(-0.7972, 53.2611);
        let line_point_a = Point::new(-1.7297, 53.3206);
        let line_point_b = Point::new(0.1334, 53.1887);
        assert_relative_eq!(
            p.cross_track_distance(&line_point_a, &line_point_b),
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
            p.cross_track_distance(&line_point_a, &line_point_b),
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
            p.cross_track_distance(&line_point_a, &line_point_b),
            Haversine.distance(p, Point::new(1., 0.)),
            epsilon = 1.0e-6
        );

        assert_relative_eq!(
            p.cross_track_distance(&line_point_b, &line_point_a),
            Haversine.distance(p, Point::new(1., 0.)),
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn new_york_to_line_between_miami_and_washington() {
        let p1 = Point::new(-74.006f64, 40.7128f64);
        let line_point_a = Point::new(-80.1918f64, 25.7617f64);
        let line_point_b = Point::new(-120.7401f64, 47.7511f64);

        assert_relative_eq!(
            p1.cross_track_distance(&line_point_a, &line_point_b),
            1_547_104.,
            epsilon = 1.0
        );
    }
}
