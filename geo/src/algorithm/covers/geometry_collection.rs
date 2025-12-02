use super::{Covers, impl_covers_from_relate, impl_covers_geometry_for};
use crate::covers::impl_covers_from_intersects;
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

impl_covers_from_intersects!(GeometryCollection<T>,[Point<T>,MultiPoint<T>]);
impl_covers_from_relate!(GeometryCollection<T>, [
Line<T>,
LineString<T>, MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>, MultiPolygon<T>,
GeometryCollection<T>
]);
impl_covers_geometry_for!(GeometryCollection<T>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::convert::Convert;
    use crate::wkt;

    #[test]
    fn test_rhs_empty() {
        let s: GeometryCollection<f64> = wkt!(GEOMETRYCOLLECTION(POINT(0 0))).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));
    }

    #[test]
    fn test_lhs_empty() {
        let s: GeometryCollection<f64> = GeometryCollection::empty();

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
