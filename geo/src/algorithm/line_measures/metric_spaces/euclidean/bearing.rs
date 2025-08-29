use crate::CoordFloat;
use crate::Euclidean;
use crate::Point;
use crate::algorithm::Bearing;
use num_traits::FromPrimitive;

impl<F: CoordFloat + FromPrimitive> Bearing<F> for Euclidean {
    /// Returns the bearing from `origin` to `destination` in degrees along a straight line.
    ///
    /// # Units
    ///
    /// - `origin`, `destination`: Points where x/y are cartesian coordinates
    /// - returns: degrees, measured clockwise from the positive Y-axis direction
    ///   - 0° = positive Y direction (typically North)
    ///   - 90° = positive X direction (typically East)
    ///   - 180° = negative Y direction (typically South)
    ///   - 270° = negative X direction (typically West)
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Euclidean, Point, Bearing};
    ///
    /// let origin = Point::new(0.0, 0.0);
    /// let destination = Point::new(1.0, 1.0);
    /// let bearing = Euclidean.bearing(origin, destination);
    /// // NorthEast
    /// assert_relative_eq!(bearing, 45.0, epsilon = 1.0e-2);
    /// ```
    fn bearing(&self, origin: Point<F>, destination: Point<F>) -> F {
        let three_sixty =
            F::from(360.0).expect("Numeric type to be constructable from primitive 360");

        let dx = destination.x() - origin.x();
        let dy = destination.y() - origin.y();

        // Calculate angle from north (using dx first gives angle from north)
        let angle = dx.atan2(dy);

        // Convert to degrees and normalize to 0-360
        let mut bearing = angle.to_degrees();
        bearing = (bearing + three_sixty) % three_sixty;

        bearing
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::Bearing;
    use crate::{Euclidean, Point};

    #[test]
    fn test_bearing() {
        let origin = Point::new(10.0, 10.0);
        let north = Point::new(10.0, 11.0);
        let south = Point::new(10.0, 9.0);
        let east = Point::new(11.0, 10.0);
        let west = Point::new(9.0, 10.0);

        let bearing = Euclidean.bearing(origin, north);
        assert_eq!(bearing, 0.0);
        let bearing = Euclidean.bearing(origin, south);
        assert_eq!(bearing, 180.0);
        let bearing = Euclidean.bearing(origin, east);
        assert_eq!(bearing, 90.0);
        let bearing = Euclidean.bearing(origin, west);
        assert_eq!(bearing, 270.0);
    }

    #[test]
    fn test_self_bearing() {
        let origin = Point::new(0.0, 0.0);
        let bearing = Euclidean.bearing(origin, origin);
        assert_eq!(bearing, 0.0);
    }
}
