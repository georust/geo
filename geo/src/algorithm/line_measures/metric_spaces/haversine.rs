use num_traits::FromPrimitive;

use super::super::{Bearing, Destination, Distance, InterpolatePoint};
use crate::utils::normalize_longitude;
use crate::{CoordFloat, Point};

/// Use the [`Haversine`] constant (an instance of `HaversineMeasure`) rather than building your own
/// customized [`HaversineMeasure`] for standard spherical Earth measurements.
///
/// `HaversineMeasure` measures distance on a sphere using the [haversine formula].
/// Distances are considered [great circle] lengths and given in units that match those of
/// the `radius` passed to [`HaversineMeasure::new`] (typically meters).
///
/// You may specify a custom radius for the Earth (or other sphere), but for normal spherical
/// measurements of the Earth, you should use the simpler [`Haversine`] which uses the standard
/// earth radius of 6371.0088 km (6_371_008.7714 m), based on the recommendation of the IUGG.
///
/// # Examples
///
/// ```
/// # use approx::assert_relative_eq;
/// use geo::{wkt, HaversineMeasure, Haversine, Distance};
///
/// let start = wkt!(POINT(23.319941 42.698334)); // Sofia: Longitude, Latitude
/// let finish = wkt!(POINT(24.742168 42.136097)); // Plovdiv: Longitude, Latitude
///
/// // Typically, you can use `Haversine` for measuring on the Earth's surface.
/// assert_relative_eq!(
///     132433.09929460194,
///     Haversine.distance(start, finish)
/// );
///
/// // The default `Haversine` const has a radius equal to the mean radius of the GRS80 ellipsoid (6371.0088 km).
/// assert_relative_eq!(
///     Haversine.radius(),
///     HaversineMeasure::GRS80_MEAN_RADIUS.radius()
/// );
///
/// // You may choose to use one of the other well known estimations of the Earth's radius,
/// // which may result in *slightly* different results.
/// assert_relative_eq!(
///     132433.06564071847,
///     HaversineMeasure::GRS80_EQUAL_AREA.distance(start, finish)
/// );
///
/// // Or you can specify whatever radius you want to get some "out of this world" results.
/// let mars_sphere = HaversineMeasure::new(3_389_500.0); // 👽 Mars radius in meters
/// assert_relative_eq!(
///     70456.97222377927,
///     mars_sphere.distance(start, finish)
/// );
/// ```
///
/// # References
///
/// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
/// "Derived Geometric Constants: **R1: mean radius**" (p131)
/// - <https://link.springer.com/article/10.1007%2Fs001900050278>
/// - <https://sci-hub.se/https://doi.org/10.1007/s001900050278>
///
/// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
/// [great circle]: https://en.wikipedia.org/wiki/Great_circle
pub struct HaversineMeasure {
    radius: f64,
}

impl Default for HaversineMeasure {
    fn default() -> Self {
        HaversineMeasure::GRS80_MEAN_RADIUS
    }
}

/// `Haversine` measures distance on a sphere using the [haversine formula]. Distances are
/// considered [great circle] lengths and given in meters.
///
/// See [`HaversineMeasure`](HaversineMeasure#trait-implementations) for all the trait methods available to [`Haversine`].
///
/// # Examples
///
/// ```
/// # use approx::assert_relative_eq;
/// use geo::{wkt, Haversine, Distance};
///
/// let start = wkt!(POINT(23.319941 42.698334)); // Sofia: Longitude, Latitude
/// let finish = wkt!(POINT(24.742168 42.136097)); // Plovdiv: Longitude, Latitude
///
/// // Use `Haversine` for measuring on the Earth's surface.
/// assert_relative_eq!(
///     132433.09929460194,
///     Haversine.distance(start, finish)
/// );
/// ```
///
/// # References
///
/// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
/// "Derived Geometric Constants: **R1: mean radius**" (p131)
/// - <https://link.springer.com/article/10.1007%2Fs001900050278>
/// - <https://sci-hub.se/https://doi.org/10.1007/s001900050278>
///
/// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
/// [great circle]: https://en.wikipedia.org/wiki/Great_circle
#[allow(non_upper_case_globals)]
pub const Haversine: HaversineMeasure = HaversineMeasure::GRS80_MEAN_RADIUS;

impl HaversineMeasure {
    /// ## Parameters
    /// - radius: The radius of the sphere, typically in meters.
    pub const fn new(radius: f64) -> Self {
        Self { radius }
    }

    pub const fn radius(&self) -> f64 {
        self.radius
    }

    /// A sphere with radius equal to the mean radius of the GRS80 ellipsoid — `R₁`,
    /// copied from [Moritz (2000)].
    ///
    /// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
    /// "Derived Geometric Constants: **R₁: mean radius**" (p131)
    /// - <https://link.springer.com/article/10.1007%2Fs001900050278>
    /// - <https://sci-hub.se/https://doi.org/10.1007/s001900050278>
    ///
    /// [Moritz (2000)]: https://sci-hub.se/https://doi.org/10.1007/s001900050278
    pub const GRS80_MEAN_RADIUS: Self = Self {
        radius: 6_371_008.8,
    };

    /// A sphere with the same surface area as the GRS80 ellipsoid, having radius `R₂`,
    /// copied from [Moritz (2000)].
    ///
    /// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
    /// "Derived Geometric Constants: **R₂: radius of sphere of same surface**" (p131)
    /// - <https://link.springer.com/article/10.1007%2Fs001900050278>
    /// - <https://sci-hub.se/https://doi.org/10.1007/s001900050278>
    ///
    /// [Moritz (2000)]: https://sci-hub.se/https://doi.org/10.1007/s001900050278
    pub const GRS80_EQUAL_AREA: Self = Self {
        radius: 6_371_007.181_0,
    };

    /// A sphere with the same volume as the GRS80 ellipsoid, having radius `R₃`,
    /// copied from [Moritz (2000)].
    ///
    /// Moritz, H. (2000). Geodetic Reference System 1980. Journal of Geodesy, 74(1), 128–133. doi:10.1007/s001900050278
    /// "Derived Geometric Constants: **R₃: radius of sphere of same volume**" (p131)
    /// - <https://link.springer.com/article/10.1007%2Fs001900050278>
    /// - <https://sci-hub.se/https://doi.org/10.1007/s001900050278>
    ///
    /// [Moritz (2000)]: https://sci-hub.se/https://doi.org/10.1007/s001900050278
    pub const GRS80_EQUAL_VOLUME: Self = Self {
        radius: 6_371_000.790_0,
    };
}

impl<F: CoordFloat + FromPrimitive> Bearing<F> for HaversineMeasure {
    /// Returns the bearing from `origin` to `destination` in degrees along a [great circle].
    ///
    /// # Units
    ///
    /// - `origin`, `destination`: Points where x/y are lon/lat degree coordinates
    /// - returns: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Bearing};
    /// use geo::Point;
    ///
    /// let origin = Point::new(9.0, 10.0);
    /// let destination = Point::new(9.5, 10.1);
    /// let bearing = Haversine.bearing(origin, destination);
    /// // A little north of east
    /// assert_relative_eq!(bearing, 78.47, epsilon = 1.0e-2);
    /// ```
    ///
    /// # References
    ///
    /// Bullock, R.: Great Circle Distances and Bearings Between Two Locations, 2007.
    /// (<https://dtcenter.org/met/users/docs/write_ups/gc_simple.pdf>)
    ///
    /// [great circle]: https://en.wikipedia.org/wiki/Great_circle
    fn bearing(&self, origin: Point<F>, destination: Point<F>) -> F {
        let three_sixty =
            F::from(360.0).expect("Numeric type to be constructable from primitive 360");
        let (lng_a, lat_a) = (origin.x().to_radians(), origin.y().to_radians());
        let (lng_b, lat_b) = (destination.x().to_radians(), destination.y().to_radians());
        let delta_lng = lng_b - lng_a;
        let s = lat_b.cos() * delta_lng.sin();
        let c = lat_a.cos() * lat_b.sin() - lat_a.sin() * lat_b.cos() * delta_lng.cos();

        let degrees = F::atan2(s, c).to_degrees();
        (degrees + three_sixty) % three_sixty
    }
}

impl<F: CoordFloat + FromPrimitive> Destination<F> for HaversineMeasure {
    /// Returns a new point having travelled the `distance` along a [great circle]
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
    /// use geo::{Haversine, Destination};
    /// use geo::Point;
    ///
    /// let origin = Point::new(9.177789688110352, 48.776781529534965);
    /// let destination = Haversine.destination(origin, 45., 10000.);
    /// assert_relative_eq!(Point::new(9.274409949623532, 48.84033274015048), destination);
    /// ```
    fn destination(&self, origin: Point<F>, bearing: F, meters: F) -> Point<F> {
        let center_lng = origin.x().to_radians();
        let center_lat = origin.y().to_radians();
        let bearing_rad = bearing.to_radians();

        let rad = meters / F::from(self.radius).unwrap();

        let lat =
            { center_lat.sin() * rad.cos() + center_lat.cos() * rad.sin() * bearing_rad.cos() }
                .asin();
        let lng = { bearing_rad.sin() * rad.sin() * center_lat.cos() }
            .atan2(rad.cos() - center_lat.sin() * lat.sin())
            + center_lng;

        Point::new(normalize_longitude(lng.to_degrees()), lat.to_degrees())
    }
}

impl<F: CoordFloat + FromPrimitive> Distance<F, Point<F>, Point<F>> for HaversineMeasure {
    /// Determine the distance between two points using the [haversine formula].
    ///
    /// # Units
    ///
    /// - `origin`, `destination`: Points where x/y are lon/lat degree coordinates
    /// - returns: meters
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Distance};
    /// use geo::Point;
    ///
    /// let new_york_city = Point::new(-74.006f64, 40.7128f64);
    /// let london = Point::new(-0.1278f64, 51.5074f64);
    ///
    /// let distance = Haversine.distance(new_york_city, london);
    ///
    /// assert_relative_eq!(
    ///     5_570_230., // meters
    ///     distance.round()
    /// );
    /// ```
    ///
    /// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
    fn distance(&self, origin: Point<F>, destination: Point<F>) -> F {
        let two = F::one() + F::one();
        let theta1 = origin.y().to_radians();
        let theta2 = destination.y().to_radians();
        let delta_theta = (destination.y() - origin.y()).to_radians();
        let delta_lambda = (destination.x() - origin.x()).to_radians();
        let a = (delta_theta / two).sin().powi(2)
            + theta1.cos() * theta2.cos() * (delta_lambda / two).sin().powi(2);
        let c = two * a.sqrt().asin();
        F::from(self.radius).unwrap() * c
    }
}

/// Interpolate Point(s) along a [great circle].
///
/// [great circle]: https://en.wikipedia.org/wiki/Great_circle
impl<F: CoordFloat + FromPrimitive> InterpolatePoint<F> for HaversineMeasure {
    /// Returns a new Point along a [great circle] between two existing points.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Haversine.point_at_distance_between(p1, p2, 100_000.0);
    /// assert_relative_eq!(closer_to_p1, Point::new(10.81, 20.49), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Haversine.point_at_distance_between(p1, p2, 10_000_000.0);
    /// assert_relative_eq!(closer_to_p2, Point::new(112.33, 30.57), epsilon = 1.0e-2);
    /// ```
    ///
    /// [great circle]: https://en.wikipedia.org/wiki/Great_circle
    fn point_at_distance_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        meters_from_start: F,
    ) -> Point<F> {
        let bearing = self.bearing(start, end);
        self.destination(start, bearing, meters_from_start)
    }

    /// Returns a new Point along a [great circle] between two existing points.
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, InterpolatePoint};
    /// use geo::Point;
    ///
    /// let p1 = Point::new(10.0, 20.0);
    /// let p2 = Point::new(125.0, 25.0);
    ///
    /// let closer_to_p1 = Haversine.point_at_ratio_between(p1, p2, 0.1);
    /// assert_relative_eq!(closer_to_p1, Point::new(19.52, 25.27), epsilon = 1.0e-2);
    ///
    /// let closer_to_p2 = Haversine.point_at_ratio_between(p1, p2, 0.9);
    /// assert_relative_eq!(closer_to_p2, Point::new(114.72, 29.65), epsilon = 1.0e-2);
    ///
    /// let midpoint = Haversine.point_at_ratio_between(p1, p2, 0.5);
    /// assert_relative_eq!(midpoint, Point::new(65.87, 37.62), epsilon = 1.0e-2);
    /// ```
    ///
    /// [great circle]: https://en.wikipedia.org/wiki/Great_circle
    fn point_at_ratio_between(
        &self,
        start: Point<F>,
        end: Point<F>,
        ratio_from_start: F,
    ) -> Point<F> {
        if start == end || ratio_from_start == F::zero() {
            return start;
        }
        if ratio_from_start == F::one() {
            return end;
        }
        let calculation = HaversineIntermediateFillCalculation::new(start, end);
        calculation.point_at_ratio(ratio_from_start)
    }

    /// Interpolates `Point`s along a [great circle] between `start` and `end`.
    ///
    /// As many points as necessary will be added such that the [haversine distance] between points
    /// never exceeds `max_distance`. If the distance between start and end is less than
    /// `max_distance`, no additional points will be included in the output.
    ///
    /// `include_ends`: Should the start and end points be included in the output?
    ///
    /// [great circle]: https://en.wikipedia.org/wiki/Great_circle
    /// [haversine formula]: https://en.wikipedia.org/wiki/Haversine_formula
    fn points_along_line(
        &self,
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>> {
        let calculation = HaversineIntermediateFillCalculation::new(start, end);
        let HaversineIntermediateFillCalculation { d, .. } = calculation;

        let total_distance = d * F::from(self.radius).unwrap();

        if total_distance <= max_distance {
            return if include_ends {
                vec![start, end].into_iter()
            } else {
                vec![].into_iter()
            };
        }

        let number_of_points = (total_distance / max_distance).ceil();
        let interval = F::one() / number_of_points;

        let mut current_step = interval;
        let mut points = if include_ends { vec![start] } else { vec![] };

        while current_step < F::one() {
            let point = calculation.point_at_ratio(current_step);
            points.push(point);
            current_step = current_step + interval;
        }

        if include_ends {
            points.push(end);
        }

        points.into_iter()
    }
}

#[allow(clippy::many_single_char_names)]
struct HaversineIntermediateFillCalculation<T> {
    d: T,
    n: T,
    o: T,
    p: T,
    q: T,
    r: T,
    s: T,
}

impl<T: CoordFloat + FromPrimitive> HaversineIntermediateFillCalculation<T> {
    #[allow(clippy::many_single_char_names)]
    fn new(p1: Point<T>, p2: Point<T>) -> Self {
        let one = T::one();
        let two = one + one;

        let lat1 = p1.y().to_radians();
        let lon1 = p1.x().to_radians();
        let lat2 = p2.y().to_radians();
        let lon2 = p2.x().to_radians();

        let (lat1_sin, lat1_cos) = lat1.sin_cos();
        let (lat2_sin, lat2_cos) = lat2.sin_cos();
        let (lon1_sin, lon1_cos) = lon1.sin_cos();
        let (lon2_sin, lon2_cos) = lon2.sin_cos();

        let m = lat1_cos * lat2_cos;

        let n = lat1_cos * lon1_cos;
        let o = lat2_cos * lon2_cos;
        let p = lat1_cos * lon1_sin;
        let q = lat2_cos * lon2_sin;

        let k =
            (((lat1 - lat2) / two).sin().powi(2) + m * ((lon1 - lon2) / two).sin().powi(2)).sqrt();

        let d = two * k.asin();

        Self {
            d,
            n,
            o,
            p,
            q,
            r: lat1_sin,
            s: lat2_sin,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn point_at_ratio(&self, ratio_from_start: T) -> Point<T> {
        let one = T::one();

        let HaversineIntermediateFillCalculation {
            d,
            n,
            o,
            p,
            q,
            r,
            s,
        } = *self;

        let a = ((one - ratio_from_start) * d).sin() / d.sin();
        let b = (ratio_from_start * d).sin() / d.sin();

        let x = a * n + b * o;
        let y = a * p + b * q;
        let z = a * r + b * s;

        let lat = z.atan2(x.hypot(y));
        let lon = y.atan2(x);

        Point::new(lon.to_degrees(), lat.to_degrees())
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
            assert_relative_eq!(0.0, Haversine.bearing(origin, destination));
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(1.0, 0.0);
            assert_relative_eq!(90.0, Haversine.bearing(origin, destination));
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(0.0, -1.0);
            assert_relative_eq!(180.0, Haversine.bearing(origin, destination));
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let destination = Point::new(-1.0, 0.0);
            assert_relative_eq!(270.0, Haversine.bearing(origin, destination));
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
                Haversine.destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn east() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 90.0;
            assert_relative_eq!(
                Point::new(0.8993203637245415, 5.506522912913066e-17),
                Haversine.destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn south() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 180.0;
            assert_relative_eq!(
                Point::new(0.0, -0.899320363724538),
                Haversine.destination(origin, bearing, 100_000.0)
            );
        }

        #[test]
        fn west() {
            let origin = Point::new(0.0, 0.0);
            let bearing = 270.0;
            assert_relative_eq!(
                Point::new(-0.8993203637245415, -1.6519568738739197e-16),
                Haversine.destination(origin, bearing, 100_000.0)
            );
        }
    }

    mod distance {
        use super::*;

        #[test]
        fn new_york_to_london() {
            let new_york_city = Point::new(-74.006f64, 40.7128f64);
            let london = Point::new(-0.1278f64, 51.5074f64);

            let distance = Haversine.distance(new_york_city, london);

            assert_relative_eq!(
                5_570_230., // meters
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
            let midpoint = Haversine.point_at_ratio_between(start, end, 0.5);
            assert_relative_eq!(midpoint, Point::new(65.87394172511485, 37.61809316888599));
        }
        #[test]
        fn points_along_line_with_endpoints() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let max_dist = 1000000.0; // meters
            let route = Haversine
                .points_along_line(start, end, max_dist, true)
                .collect::<Vec<_>>();
            assert_eq!(route.len(), 13);
            assert_eq!(route[0], start);
            assert_eq!(route.last().unwrap(), &end);
            assert_relative_eq!(route[1], Point::new(17.882467331860965, 24.435542998803793));
        }
        #[test]
        fn points_along_line_without_endpoints() {
            let start = Point::new(10.0, 20.0);
            let end = Point::new(125.0, 25.0);
            let max_dist = 1000000.0; // meters
            let route = Haversine
                .points_along_line(start, end, max_dist, false)
                .collect::<Vec<_>>();
            assert_eq!(route.len(), 11);
            assert_relative_eq!(route[0], Point::new(17.882467331860965, 24.435542998803793));
        }
    }
}
