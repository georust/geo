use super::super::{Bearing, Destination, Distance, InterpolatePoint};
use crate::Point;
use geographiclib_rs::{DirectGeodesic, InverseGeodesic};

/// An ellipsoidal model of the earth, using methods given by [Karney (2013)].
///
/// Distances are computed using [geodesic lines] and are measured in meters.
///
/// [geodesic lines]: https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid
/// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
pub struct Geodesic;

impl Bearing<f64> for Geodesic {
    /// Returns the bearing from `origin` to `destination` in degrees along a [geodesic line].
    ///
    /// # Units
    ///
    /// - `origin`, `destination`: Point where x/y are lon/lat degree coordinates
    /// - returns: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Geodesic, Bearing};
    /// use geo::Point;
    ///
    /// let origin = Point::new(9.0, 10.0);
    /// let destination = Point::new(9.5, 10.1);
    /// let bearing = Geodesic::bearing(origin, destination);
    /// // A little north of east
    /// assert_relative_eq!(bearing, 78.54, epsilon = 1.0e-2);
    /// ```
    ///
    /// # References
    ///
    /// This uses the geodesic methods given by [Karney (2013)].
    ///
    /// [geodesic line]: https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn bearing(origin: Point<f64>, destination: Point<f64>) -> f64 {
        let (azi1, _, _) = geographiclib_rs::Geodesic::wgs84().inverse(
            origin.y(),
            origin.x(),
            destination.y(),
            destination.x(),
        );
        (azi1 + 360.0) % 360.0
    }
}

impl Destination<f64> for Geodesic {
    /// Returns a new point having travelled the `distance` along a [geodesic line]
    /// from the `origin` point with the given `bearing`.
    ///
    /// This uses the geodesic methods given by [Karney (2013)].
    ///
    /// # Units
    ///
    /// - `bearing`: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°
    /// - `distance`: meters
    /// - returns: Point where x/y are lon/lat degree coordinates
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Geodesic, Destination};
    /// use geo::Point;
    ///
    /// // Determine the point 100 km NE of JFK airport.
    /// let jfk = Point::new(-73.78, 40.64);
    /// let northeast_bearing = 45.0;
    /// let distance = 100_000.0;
    ///
    /// let northeast_of_jfk = Geodesic::destination(jfk, northeast_bearing, distance);
    /// assert_relative_eq!(Point::new(-72.94, 41.27), northeast_of_jfk, epsilon = 1.0e-2);
    /// ```
    ///
    /// # References
    ///
    /// This uses the geodesic methods given by [Karney (2013)].
    ///
    /// [geodesic line]: https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn destination(origin: Point<f64>, bearing: f64, distance: f64) -> Point<f64> {
        let (lat, lon) =
            geographiclib_rs::Geodesic::wgs84().direct(origin.y(), origin.x(), bearing, distance);
        Point::new(lon, lat)
    }
}

impl Distance<f64, Point<f64>, Point<f64>> for Geodesic {
    /// Determine the length of the [geodesic line] between two geometries on an ellipsoidal model of the earth.
    ///
    /// # Units
    /// - `origin`, `destination`: Point where x/y are lon/lat degree coordinates/
    /// - returns: meters
    ///
    /// # Examples
    /// ```rust
    /// use geo::{Geodesic, Distance};
    /// use geo::Point;
    ///
    /// // New York City
    /// let new_york_city = Point::new(-74.006, 40.7128);
    ///
    /// // London
    /// let london = Point::new(-0.1278, 51.5074);
    ///
    /// let distance = Geodesic::distance(new_york_city, london);
    ///
    /// assert_eq!(
    ///     5_585_234., // meters
    ///     distance.round()
    /// );
    /// ```
    ///
    /// # References
    ///
    /// This uses the geodesic methods given by [Karney (2013)].
    ///
    /// [geodesic line]: https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn distance(origin: Point<f64>, destination: Point<f64>) -> f64 {
        geographiclib_rs::Geodesic::wgs84().inverse(
            origin.y(),
            origin.x(),
            destination.y(),
            destination.x(),
        )
    }
}

/// Interpolate Point(s) along a [geodesic line].
///
/// [geodesic line]: https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid
impl InterpolatePoint<f64> for Geodesic {
    /// Returns a new Point along a [geodesic line] between two existing points on an ellipsoidal model of the earth.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Geodesic, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Geodesic::point_at_ratio_between(p1, p2, 0.1);
    /// assert_relative_eq!(closer_to_p1, Point::new(19.52, 25.31), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Geodesic::point_at_ratio_between(p1, p2, 0.9);
    /// assert_relative_eq!(closer_to_p2, Point::new(114.73, 29.69), epsilon = 1.0e-2);
    ///
    /// let midpoint = Geodesic::point_at_ratio_between(p1, p2, 0.5);
    /// assert_relative_eq!(midpoint, Point::new(65.88, 37.72), epsilon = 1.0e-2);
    /// ```
    ///
    /// # References
    ///
    /// This uses the geodesic methods given by [Karney (2013)].
    ///
    /// [geodesic line]: https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn point_at_ratio_between(
        start: Point<f64>,
        end: Point<f64>,
        ratio_from_start: f64,
    ) -> Point<f64> {
        let g = geographiclib_rs::Geodesic::wgs84();
        let (total_distance, azi1, _azi2, _a12) = g.inverse(start.y(), start.x(), end.y(), end.x());
        let distance = total_distance * ratio_from_start;
        let (lat2, lon2) = g.direct(start.y(), start.x(), azi1, distance);

        Point::new(lon2, lat2)
    }

    /// Interpolates `Point`s along a [geodesic line] between `start` and `end`.
    ///
    /// As many points as necessary will be added such that the geodesic distance between points
    /// never exceeds `max_distance`. If the distance between start and end is less than
    /// `max_distance`, no additional points will be included in the output.
    ///
    /// `include_ends`: Should the start and end points be included in the output?
    ///
    /// # References
    ///
    /// This uses the geodesic methods given by [Karney (2013)].
    ///
    /// [geodesic line]: https://en.wikipedia.org/wiki/Geodesics_on_an_ellipsoid
    /// [Karney (2013)]:  https://arxiv.org/pdf/1109.4448.pdf
    fn points_along_line(
        start: Point<f64>,
        end: Point<f64>,
        max_distance: f64,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<f64>> {
        let g = geographiclib_rs::Geodesic::wgs84();
        let (total_distance, azi1, _azi2, _a12) = g.inverse(start.y(), start.x(), end.y(), end.x());

        if total_distance <= max_distance {
            return if include_ends {
                vec![start, end].into_iter()
            } else {
                vec![].into_iter()
            };
        }

        let number_of_points = (total_distance / max_distance).ceil();
        let interval = 1.0 / number_of_points;

        let mut current_step = interval;
        let mut points = if include_ends { vec![start] } else { vec![] };

        while current_step < 1.0 {
            let (lat2, lon2) = g.direct(start.y(), start.x(), azi1, total_distance * current_step);
            let point = Point::new(lon2, lat2);
            points.push(point);
            current_step += interval;
        }

        if include_ends {
            points.push(end);
        }

        points.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type MetricSpace = Geodesic;

    mod bearing {
        use super::*;

        #[test]
        fn north() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(0.0, 1.0);
            assert_relative_eq!(0.0, MetricSpace::bearing(origin, destination));
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(1.0, 0.0);
            assert_relative_eq!(90.0, MetricSpace::bearing(origin, destination));
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(0.0, -1.0);
            assert_relative_eq!(180.0, MetricSpace::bearing(origin, destination));
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(-1.0, 0.0);
            assert_relative_eq!(270.0, MetricSpace::bearing(origin, destination));
        }
    }

    mod destination {
        use super::*;

        #[test]
        fn north() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 0.0;
            assert_relative_eq!(
                Point::new(0.0, 0.9043687229127633),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 90.0;
            assert_relative_eq!(
                Point::new(0.8983152841195217, 0.0),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 180.0;
            assert_relative_eq!(
                Point::new(0.0, -0.9043687229127633),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 270.0;
            assert_relative_eq!(
                Point::new(-0.8983152841195217, 0.0),
                MetricSpace::destination(origin, bearing, 100_000.0)
            );
        }

        mod distance {
            use super::*;

            #[test]
            fn new_york_to_london() {
                let new_york_city = Point::new(-74.006f64, 40.7128f64);
                let london = Point::new(-0.1278f64, 51.5074f64);

                let distance = MetricSpace::distance(new_york_city, london);

                assert_relative_eq!(
                    5_585_234.0, // meters
                    distance.round()
                );
            }
        }

        mod interpolate_point {
            use super::*;

            #[test]
            fn point_at_ratio_between_midpoint() {
                let start = Point::new(10.0, 20.0);
                let end = Point::new(125.0, 25.0);
                let midpoint = MetricSpace::point_at_ratio_between(start, end, 0.5);
                assert_relative_eq!(midpoint, Point::new(65.87936072133309, 37.72225378005785));
            }
            #[test]
            fn points_along_line_with_endpoints() {
                let start = Point::new(10.0, 20.0);
                let end = Point::new(125.0, 25.0);
                let max_dist = 1000000.0; // meters
                let route =
                    MetricSpace::points_along_line(start, end, max_dist, true).collect::<Vec<_>>();
                assert_eq!(route.len(), 13);
                assert_eq!(route[0], start);
                assert_eq!(route.last().unwrap(), &end);
                assert_relative_eq!(route[1], Point::new(17.878754355562464, 24.466667836189565));
            }
            #[test]
            fn points_along_line_without_endpoints() {
                let start = Point::new(10.0, 20.0);
                let end = Point::new(125.0, 25.0);
                let max_dist = 1000000.0; // meters
                let route =
                    MetricSpace::points_along_line(start, end, max_dist, false).collect::<Vec<_>>();
                assert_eq!(route.len(), 11);
                assert_relative_eq!(route[0], Point::new(17.878754355562464, 24.466667836189565));
            }
        }
    }
}
