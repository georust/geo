use crate::{CoordFloat, Point};

// REVIEW: Naming alternatives:
// - LinearReferencing
// - PointAlongLine
// - LineInterpolatePoint (postgis)
// - Interpolate (shapely)
// - Position (geographiclib)
// - Intermediate (georust::geo)
pub trait InterpolatePoint<F: CoordFloat> {
    /// Returns a new Point along a line between two existing points
    ///
    /// See [specific implementations](#implementors) for details.
    fn point_at_ratio_between(start: Point<F>, end: Point<F>, ratio_from_start: F) -> Point<F>;

    // TODO:
    // fn point_at_distance_between(start: Point<F>, end: Point<F>, distance_from_start: F) -> Point<F>;

    /// Interpolates `Point`s along a line between `start` and `end`.
    ///
    /// See [specific implementations](#implementors) for details.
    ///
    /// As many points as necessary will be added such that the distance between points
    /// never exceeds `max_distance`. If the distance between start and end is less than
    /// `max_distance`, no additional points will be included in the output.
    ///
    /// `include_ends`: Should the start and end points be included in the output?
    fn points_along_line(
        start: Point<F>,
        end: Point<F>,
        max_distance: F,
        include_ends: bool,
    ) -> impl Iterator<Item = Point<F>>;
}
