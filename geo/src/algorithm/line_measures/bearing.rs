use geo_types::{CoordFloat, Point};

/// Calculate the bearing between two points.
pub trait Bearing<F: CoordFloat> {
    /// Calculate the bearing from `origin` to `destination` in degrees.
    ///
    /// See [specific implementations](#implementors) for details.
    ///
    /// # Units
    /// - `origin`, `destination`: Point where the units of x/y depend on the [trait implementation](#implementors).
    /// - returns: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, Haversine, Bearing, Geodesic};
    ///
    /// let point_1 = Point::new(0.0, 0.0);
    /// let point_2 = Point::new(0.0, 2.0);
    ///
    /// // Due north
    /// assert_eq!(Haversine.bearing(point_1, point_2), 0.0);
    /// assert_eq!(Geodesic.bearing(point_1, point_2), 0.0);
    /// ```
    fn bearing(&self, origin: Point<F>, destination: Point<F>) -> F;
}
