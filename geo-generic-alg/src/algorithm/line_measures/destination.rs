use geo_types::{CoordFloat, Point};

/// Calculate the destination point from an origin point, given a bearing and a distance.
pub trait Destination<F: CoordFloat> {
    /// Returns a new point having travelled the `distance` along a line
    /// from the `origin` point with the given `bearing`.
    ///
    /// See [specific implementations](#implementors) for details.
    ///
    /// # Units
    ///
    /// - `origin`: Point where the units of x/y depend on the [trait implementation](#implementors).
    /// - `bearing`: degrees, where: North: 0°, East: 90°, South: 180°, West: 270°
    /// - `distance`: depends on the [trait implementation](#implementors).
    /// - returns: Point where the units of x/y depend on the [trait implementation](#implementors).
    ///
    /// # Examples
    ///
    /// ```
    /// # use approx::assert_relative_eq;
    /// use geo::{Haversine, Rhumb, Geodesic, Destination, Point};
    ///
    /// let point = Point::new(0.0, 0.0);
    ///
    /// assert_relative_eq!(Haversine.destination(point, 45.0, 111_111.0), Point::new(0.706607921147679, 0.7065541919063233));
    /// assert_relative_eq!(Geodesic.destination(point, 45.0, 111_111.0), Point::new(0.7058183774535367, 0.7105205988658333));
    /// assert_relative_eq!(Rhumb.destination(point, 45.0, 111_111.0), Point::new(0.706590011673029, 0.7065721019258285));
    /// ```
    ///
    /// [`metric_spaces`]: super::metric_spaces
    fn destination(&self, origin: Point<F>, bearing: F, distance: F) -> Point<F>;
}
