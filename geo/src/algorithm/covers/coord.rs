use super::{Covers, impl_covers_from_intersects};
use crate::GeoNum;
use crate::Intersects;
use crate::geometry::*;

impl<T, G> Covers<Coord<T>> for G
where
    T: GeoNum,
    G: Intersects<Coord<T>>,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.intersects(rhs)
    }
}

// valid because self is convex geometry
// all exterior pts of RHS intersecting self means self covers RHS
impl_covers_from_intersects!(Coord<T>, [
Point<T>, MultiPoint<T>,
Line<T>,
LineString<T>, MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhs_empty() {
        let s: Coord<f64> = Coord::zero();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));
    }
}
