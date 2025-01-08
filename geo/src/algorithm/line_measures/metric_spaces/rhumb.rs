use num_traits::FromPrimitive;

use super::super::{Bearing, Destination, Distance, InterpolatePoint};
use crate::rhumb::RhumbCalculations;
use crate::{CoordFloat, Point, MEAN_EARTH_RADIUS};

/// Provides [rhumb line] (a.k.a. loxodrome) geometry operations. A rhumb line appears as a straight
/// line on a Mercator projection map.
///
/// Rhumb distance is measured in meters.
///
/// # References
///
/// The distance, destination, and bearing implementations are adapted in part
/// from their equivalents in [Turf.js](https://turfjs.org/), which in turn are
/// adapted from the Movable Type
/// [spherical geodesy tools](https://www.movable-type.co.uk/scripts/latlong.html).
///
/// Turf.js is copyright its authors and the geodesy tools are copyright Chris
/// Veness; both are available under an MIT license.
///
/// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
pub struct Rhumb;

impl<F: CoordFloat + FromPrimitive> Bearing<F> for Rhumb {
    /// Returns the bearing from `origin` to `destination` in degrees along a [rhumb line].
    ///
    /// # Units
    ///
    /// - `origin`, `destination`: Points where x/y are lon/lat degree coordinates
    /// - returns: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°/
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, Bearing};
    /// use geo::Point;
    ///
    /// let origin = Point::new(9.177789688110352, 48.776781529534965);
    /// let destination = Point::new(9.274348757829898, 48.84037308229984);
    /// let bearing = Rhumb.bearing(origin, destination);
    /// assert_relative_eq!(bearing, 45., epsilon = 1.0e-6);
    /// ```
    ///
    /// # References
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    ///
    /// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
    /// (<https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf>)
    fn bearing(&self, origin: Point<F>, destination: Point<F>) -> F {
        let three_sixty = F::from(360.0f64).unwrap();

        let calculations = RhumbCalculations::new(&origin, &destination);
        (calculations.theta().to_degrees() + three_sixty) % three_sixty
    }
}

impl<F: CoordFloat + FromPrimitive> Destination<F> for Rhumb {
    /// Returns a new point having travelled the `distance` along a [rhumb line]
    /// from the `origin` point with the given `bearing`.
    ///
    /// # Units
    ///
    /// - `origin`: Point where x/y are lon/lat degree coordinates
    /// - `bearing`: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°
    /// - `distance`: meters
    /// - returns: Point where x/y are lon/lat degree coordinates
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, Destination};
    /// use geo::Point;
    ///
    /// let p_1 = Point::new(9.177789688110352, 48.776781529534965);
    /// let p_2 = Rhumb.destination(p_1, 45., 10000.);
    /// assert_relative_eq!(p_2, Point::new(9.274348757829898, 48.84037308229984))
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn destination(&self, origin: Point<F>, bearing: F, distance: F) -> Point<F> {
        let delta = distance / F::from(MEAN_EARTH_RADIUS).unwrap(); // angular distance in radians
        let lambda1 = origin.x().to_radians();
        let phi1 = origin.y().to_radians();
        let theta = bearing.to_radians();

        crate::algorithm::rhumb::calculate_destination(delta, lambda1, phi1, theta)
    }
}

impl<F: CoordFloat + FromPrimitive> Distance<F, Point<F>, Point<F>> for Rhumb {
    /// Determine the distance along the [rhumb line] between two points.
    ///
    /// # Units
    ///
    /// - `origin`, `destination`: Points where x/y are lon/lat degree coordinates
    /// - returns: meters
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Rhumb, Distance};
    /// use geo::point;
    ///
    /// // New York City
    /// let p1 = point!(x: -74.006f64, y: 40.7128);
    ///
    /// // London
    /// let p2 = point!(x: -0.1278, y: 51.5074);
    ///
    /// let distance = Rhumb.distance(p1, p2);
    ///
    /// assert_eq!(
    ///     5_794_129., // meters
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn distance(&self, origin: Point<F>, destination: Point<F>) -> F {
        let calculations = RhumbCalculations::new(&origin, &destination);
        calculations.delta() * F::from(MEAN_EARTH_RADIUS).unwrap()
    }
}

/// Interpolate Point(s) along a [rhumb line].
///
/// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
impl<F: CoordFloat + FromPrimitive> InterpolatePoint<F> for Rhumb {
    /// Returns a new Point along a [rhumb line] between two existing points.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Rhumb.point_at_distance_between(p1, p2, 100_000.0);
    /// assert_relative_eq!(closer_to_p1, Point::new(10.96, 20.04), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Rhumb.point_at_distance_between(p1, p2, 10_000_000.0);
    /// assert_relative_eq!(closer_to_p2, Point::new(107.00, 24.23), epsilon = 1.0e-2);
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn point_at_distance_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        meters_from_start: F,
    ) -> Point<F> {
        let bearing = Self.bearing(start, end);
        Self.destination(start, bearing, meters_from_start)
    }

    /// Returns a new Point along a [rhumb line] between two existing points.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Rhumb, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Rhumb.point_at_ratio_between(p1, p2, 0.1);
    /// assert_relative_eq!(closer_to_p1, Point::new(21.32, 20.50), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Rhumb.point_at_ratio_between(p1, p2, 0.9);
    /// assert_relative_eq!(closer_to_p2, Point::new(113.31, 24.50), epsilon = 1.0e-2);
    ///
    /// let midpoint = Rhumb.point_at_ratio_between(p1, p2, 0.5);
    /// assert_relative_eq!(midpoint, Point::new(66.98, 22.50), epsilon = 1.0e-2);
    /// ```
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn point_at_ratio_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        ratio_from_start: F,
    ) -> Point<F> {
        let calculations = RhumbCalculations::new(&start, &end);
        calculations.intermediate(ratio_from_start)
    }

    /// Interpolates `Point`s along a [rhumb line] between `start` and `end`.
    ///
    /// As many points as necessary will be added such that the distance between points
    /// never exceeds `max_distance`. If the distance between start and end is less than
    /// `max_distance`, no additional points will be included in the output.
    ///
    /// `include_ends`: Should the start and end points be included in the output?
    ///
    /// [rhumb line]: https://en.wikipedia.org/wiki/Rhumb_line
    fn points_along_line(
        &self,
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>> {
        let max_delta = max_distance / F::from(MEAN_EARTH_RADIUS).unwrap();
        let calculations = RhumbCalculations::new(&start, &end);
        calculations
            .intermediate_fill(max_delta, include_ends)
            .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod bearing {
        use super::*;

        #[test]
        fn north() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(0.0, 1.0);
            assert_relative_eq!(0.0, Rhumb.bearing(origin, destination));
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(1.0, 0.0);
            assert_relative_eq!(90.0, Rhumb.bearing(origin, destination));
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(0.0, -1.0);
            assert_relative_eq!(180.0, Rhumb.bearing(origin, destination));
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(-1.0, 0.0);
            assert_relative_eq!(270.0, Rhumb.bearing(origin, destination));
        }
    }

    mod destination {
        use super::*;

        #[test]
        fn north() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 0.0;
            assert_relative_eq!(
                Point::new(0.0, 0.899320363724538),
                Rhumb.destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 90.0;
            assert_relative_eq!(
                Point::new(0.8993203637245415, 5.506522912913066e-17),
                Rhumb.destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 180.0;
            assert_relative_eq!(
                Point::new(0.0, -0.899320363724538),
                Rhumb.destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 270.0;
            assert_relative_eq!(
                Point::new(-0.8993203637245415, -1.6520247072649334e-16),
                Rhumb.destination(origin, bearing, 100_000.0)
            );
        }
    }

    mod distance {
        use super::*;

        #[test]
        fn new_york_to_london() {
            let new_york_city = Point::new(-74.006, 40.7128);
            let london = Point::new(-0.1278, 51.5074);

            let distance: f64 = Rhumb.distance(new_york_city, london);

            assert_relative_eq!(
                5_794_129., // meters
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
            let midpoint = Rhumb.point_at_ratio_between(start, end, 0.5);
            assert_relative_eq!(
                midpoint,
                Point::new(66.98011173721943, 22.500000000000007),
                epsilon = 1.0e-10
            );
        }
        #[test]
        fn points_along_line_with_endpoints() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let max_dist = 1000000.0; // meters
            let route = Rhumb
                .points_along_line(start, end, max_dist, true)
                .collect::<Vec<_>>();
            assert_eq!(route.len(), 13);
            assert_eq!(route[0], start);
            assert_eq!(route.last().unwrap(), &end);
            assert_relative_eq!(
                route[1],
                Point::new(19.43061818495096, 20.416666666666668),
                epsilon = 1.0e-10
            );
        }
        #[test]
        fn points_along_line_without_endpoints() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let max_dist = 1000000.0; // meters
            let route = Rhumb
                .points_along_line(start, end, max_dist, false)
                .collect::<Vec<_>>();
            assert_eq!(route.len(), 11);
            assert_relative_eq!(
                route[0],
                Point::new(19.43061818495096, 20.416666666666668),
                epsilon = 1.0e-10
            );
        }
    }
}
