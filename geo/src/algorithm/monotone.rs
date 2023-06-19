mod mono_poly;
use crate::{Coord, GeoNum, Intersects, Polygon};
pub use mono_poly::MonoPoly;

mod segment;
use segment::RcSegment;
pub(crate) use segment::Segment;

mod sweep;
pub(crate) use sweep::SimpleSweep;

mod builder;
pub use builder::monotone_subdivision;

/// A polygon represented as a collection of (disjoin) monotone polygons.
///
/// This structure supports `Intersects<Coord>` trait and is typically faster
/// than the equivalent method on `Polygon`.  
#[derive(Clone, Debug)]
pub struct MonotonicPolygons<T: GeoNum>(Vec<MonoPoly<T>>);

impl<T: GeoNum> MonotonicPolygons<T> {
    pub fn subdivisions(&self) -> &Vec<MonoPoly<T>> {
        &self.0
    }
}
impl<T: GeoNum> From<Polygon<T>> for MonotonicPolygons<T> {
    fn from(poly: Polygon<T>) -> Self {
        Self(monotone_subdivision(poly))
    }
}

impl<T: GeoNum> Intersects<Coord<T>> for MonotonicPolygons<T> {
    fn intersects(&self, other: &Coord<T>) -> bool {
        self.0.iter().any(|mp| mp.intersects(other))
    }
}

#[cfg(test)]
mod tests;
