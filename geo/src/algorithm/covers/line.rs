use super::{Covers, impl_covers_from_intersects};
use crate::GeoNum;
use crate::geometry::*;

// valid because self is convex geometry
// all exterior pts of RHS intersecting self means self covers RHS
impl_covers_from_intersects!(Line<T>, [
Point<T>, MultiPoint<T>,
Line<T>,
LineString<T>, MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T> ,
Geometry<T>, GeometryCollection<T>
]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::convert::Convert;
    use crate::wkt;

    #[test]
    fn test_rhs_empty() {
        let s: Line<f64> = wkt!(LINE(0 0,1 1)).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));
    }
}
