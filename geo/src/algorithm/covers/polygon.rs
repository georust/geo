use super::{Covers, impl_covers_from_intersects, impl_covers_from_relate};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

// valid because RHS is point or multipoint
// all exterior pts of RHS intersecting self means self covers RHS
impl_covers_from_intersects!(Polygon<T>, [Point<T>, MultiPoint<T>]);

impl_covers_from_relate!(Polygon<T>, [
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

// valid because RHS is point or multipoint
// all exterior pts of RHS intersecting self means self covers RHS
impl_covers_from_intersects!(MultiPolygon<T>, [Point<T>, MultiPoint<T>]);

impl_covers_from_relate!(MultiPolygon<T>, [
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
        let s: Polygon<f64> = wkt!(POLYGON((0 0,1 0, 1 1,0 1, 0 0))).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));

        let s: MultiPolygon<f64> = wkt!(MULTIPOLYGON(((0 0,1 0, 1 1,0 1, 0 0)))).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));
    }

    #[test]
    fn test_lhs_empty() {
        let s: Polygon<f64> = Polygon::empty();
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

        let s: MultiPolygon<f64> = MultiPolygon::empty();
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
