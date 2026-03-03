use super::{Covers, impl_covers_from_intersects, impl_covers_from_relate};
use crate::{Contains, geometry::*};
use crate::{GeoFloat, GeoNum};

// valid because self is convex geometry
// all exterior pts of RHS intersecting self means self covers RHS
impl_covers_from_intersects!(Point<T>, [
Point<T>, MultiPoint<T>,
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

// valid because RHS is point
impl_covers_from_intersects!(MultiPoint<T>, [Point<T>]);

// use the sliding window comparison implementation of contains
// multipoint has no boundary so they are equivalent
impl<T> Covers<MultiPoint<T>> for MultiPoint<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &MultiPoint<T>) -> bool {
        self.contains(rhs)
    }
}

impl_covers_from_relate!(MultiPoint<T>, [
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::convert::Convert;
    use crate::wkt;

    #[test]
    fn test_rhs_empty() {
        let s: Point<f64> = wkt!(POINT(0 0)).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));

        let s: MultiPoint<f64> = wkt!(MULTIPOINT(0 0, 1 1)).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));
    }

    #[test]
    fn test_lhs_empty() {
        let s: MultiPoint<f64> = MultiPoint::empty();
        assert!(!s.covers(&wkt!(POINT(0 0)).convert()));
        assert!(!s.covers(&wkt!(MULTIPOINT(0 0,1 1)).convert()));
        assert!(!s.covers(&wkt!(LINE(0 0,1 1)).convert()));
        assert!(!s.covers(&wkt!(LINESTRING(0 0,1 1)).convert()));
        assert!(!s.covers(&wkt!(MULTILINESTRING((0 0,1 1),(2 2,3 3))).convert()));
        assert!(!s.covers(&wkt!(POLYGON((0 0,1 1,1 0,0 0))).convert()));
        assert!(!s.covers(&wkt!(MULTIPOLYGON(((0 0,1 0, 1 1,0 1, 0 0)))).convert()));
        assert!(!s.covers(&wkt!(RECT(0 0, 1 1)).convert()));
        assert!(!s.covers(&wkt!(TRIANGLE(0 0, 0 1, 1 1)).convert()));
        assert!(!s.covers(&Into::<Geometry>::into(wkt!(POINT(0 0)).convert())));
        assert!(!s.covers(&wkt!(GEOMETRYCOLLECTION(POINT(0 0))).convert()));
    }
}
