use crate::EQUATORIAL_EARTH_RADIUS;
use geo_types::Point;
use geographiclib_rs::{DirectGeodesic, Geodesic, InverseGeodesic};

/// Determine the cross track distance (also known as the cross track error),
/// which is the shortest distance between a point and a line on an ellipsoid.
pub trait CrossTrackDistanceGeodesic<T, Rhs = Self> {
    /// Determine the cross track distance on an ellipsoid between this point
    /// and a line which passes through line_point_a and link_point_b
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
    /// let distance = p1.cross_track_distance_geodesic(&line_point_a, &line_point_b);
    ///
    ///     assert_eq!(
    ///     1_546_717., // meters
    ///     distance.round()
    /// );
    /// ```
    fn cross_track_distance_geodesic(&self, line_point_a: &Rhs, line_point_b: &Rhs) -> T;
}

impl CrossTrackDistanceGeodesic<f64> for Point<f64> {
    fn cross_track_distance_geodesic(
        &self,
        line_point_a: &Point<f64>,
        line_point_b: &Point<f64>,
    ) -> f64 {
        let geod = Geodesic::wgs84();
        let mut iter = 0;

        // Earth constants
        let a: f64 = EQUATORIAL_EARTH_RADIUS;
        let mut s_ax: f64;

        let mut line_point_a = *line_point_a;

        loop {
            // Get the lat/lon values of each point so that it is easier to read
            let (pp_x, pp_y) = (self.x(), self.y());
            let (lpa_x, lpa_y, lpb_x, lpb_y) = (
                line_point_a.x(),
                line_point_a.y(),
                line_point_b.x(),
                line_point_b.y(),
            );
            let (s_ap, azi1_ap, _, m_ap, mm_ap, _, _) = geod.inverse(lpa_y, lpa_x, pp_y, pp_x);
            let (azi1_ab, _, _) = geod.inverse(lpa_y, lpa_x, lpb_y, lpb_x);
            let azi1 = azi1_ap - azi1_ab;

            s_ax = m_ap * azi1.to_radians().cos()
                / ((m_ap / s_ap) * azi1.to_radians().cos().powi(2)
                    + mm_ap * azi1.to_radians().sin().powi(2));

            if iter == 0 {
                s_ax = a * ((s_ap / a).sin() * azi1.to_radians().cos()).atan2((s_ap / a).cos());
            }

            let (pa2_y, pa2_x) = geod.direct(lpa_y, lpa_x, azi1_ab, s_ax);

            if s_ax.abs() < 1e-4 {
                return s_ap;
            }

            line_point_a = Point::new(pa2_x, pa2_y);
            iter += 1
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CrossTrackDistanceGeodesic;
    use crate::GeodesicDistance;
    use crate::Point;

    #[test]
    fn distance1_test() {
        let p = Point::new(-0.7972, 53.2611);
        let line_point_a = Point::new(-1.7297, 53.3206);
        let line_point_b = Point::new(0.1334, 53.1887);
        assert_relative_eq!(
            p.cross_track_distance_geodesic(&line_point_a, &line_point_b),
            306.8446329,
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn cross_track_distance_geodesic_to_line_passing_through_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(1., 0.);
        let line_point_b = Point::new(2., 0.);

        assert_relative_eq!(
            p.cross_track_distance_geodesic(&line_point_a, &line_point_b),
            0.,
            epsilon = 1.0e-6
        );
    }
    #[test]
    fn cross_track_distance_geodesic_to_line_orthogonal_to_point() {
        let p = Point::new(0., 0.);
        let line_point_a = Point::new(1., -1.);
        let line_point_b = Point::new(1., 1.);

        assert_relative_eq!(
            p.cross_track_distance_geodesic(&line_point_a, &line_point_b),
            p.geodesic_distance(&Point::new(1., 0.)),
            epsilon = 1.0e-6
        );

        assert_relative_eq!(
            p.cross_track_distance_geodesic(&line_point_b, &line_point_a),
            p.geodesic_distance(&Point::new(1., 0.)),
            epsilon = 1.0e-6
        );
    }

    #[test]
    fn new_york_to_line_between_miami_and_washington() {
        let p1 = Point::new(-74.006f64, 40.7128f64);
        let line_point_a = Point::new(-80.1918f64, 25.7617f64);
        let line_point_b = Point::new(-120.7401f64, 47.7511f64);

        assert_relative_eq!(
            p1.cross_track_distance_geodesic(&line_point_a, &line_point_b),
            1_546_716.,
            epsilon = 1.0
        );
    }
}
