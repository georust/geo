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
    /// [`metric_spaces`]: super::metric_spaces
    fn destination(origin: Point<F>, bearing: F, distance: F) -> Point<F>;
}
