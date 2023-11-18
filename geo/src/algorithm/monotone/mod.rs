mod mono_poly;
use crate::{Coord, GeoNum, Intersects, MultiPolygon, Polygon};
pub use mono_poly::MonoPoly;

mod segment;
use segment::RcSegment;
pub(crate) use segment::Segment;

mod sweep;
pub(crate) use sweep::SimpleSweep;

mod builder;
pub use builder::monotone_subdivision;

/// A multi-polygon represented as a collection of (disjoint) monotone polygons.
///
/// This structure is optimized for point-in-polygon queries, and is typically
/// much faster than the equivalent method on `Polygon`.  This is because a
/// single monotone polygon can be tested for intersection with a point in
/// `O(log n)` time, where `n` is the number of vertices in the polygon.  In
/// contrast, the equivalent method on `Polygon` is `O(n)`.  Typically, a
/// polygon can be sub-divided into a small number of monotone polygons, thus
/// providing a significant speed-up.
///
/// # Example
///
/// Construct a `MonotonicPolygons` from a `Polygon`, or a `MultiPolygon` using
/// `MontonicPolygons::from`, and query point intersection via the
/// `Intersects<Coord>` trait.
///
/// ```rust
/// use geo::prelude::*;
/// use geo::{polygon, coord};
///
/// let polygon = polygon![
///     (x: -2., y: 1.),
///     (x: 1., y: 3.),
///     (x: 4., y: 1.),
///     (x: 1., y: -1.),
///     (x: -2., y: 1.),
/// ];
/// let mp = MonotonicPolygons::from(polygon);
/// assert!(mp.intersects(&coord!(x: -2., y: 1.)));
/// ```
#[derive(Clone, Debug)]
pub struct MonotonicPolygons<T: GeoNum>(Vec<MonoPoly<T>>);

impl<T: GeoNum> MonotonicPolygons<T> {
    /// Get a reference to the monotone polygons.
    pub fn subdivisions(&self) -> &Vec<MonoPoly<T>> {
        &self.0
    }

    /// Reduce to inner `Vec` of monotone polygons.
    pub fn into_subdivisions(self) -> Vec<MonoPoly<T>> {
        self.0
    }
}
impl<T: GeoNum> From<Polygon<T>> for MonotonicPolygons<T> {
    fn from(poly: Polygon<T>) -> Self {
        Self(monotone_subdivision([poly]))
    }
}

impl<T: GeoNum> From<MultiPolygon<T>> for MonotonicPolygons<T> {
    fn from(mp: MultiPolygon<T>) -> Self {
        Self(monotone_subdivision(mp.0))
    }
}

impl<T: GeoNum> Intersects<Coord<T>> for MonotonicPolygons<T> {
    fn intersects(&self, other: &Coord<T>) -> bool {
        self.0.iter().any(|mp| mp.intersects(other))
    }
}

#[cfg(test)]
mod tests;
