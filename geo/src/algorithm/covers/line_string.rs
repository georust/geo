use super::{Covers, impl_covers_from_intersects, impl_covers_from_relate};
use crate::{Contains, HasDimensions, geometry::*};
use crate::{GeoFloat, GeoNum};

// valid because RHS is point or multipoint
// all exterior pts of RHS intersecting self means self covers RHS
impl_covers_from_intersects!(LineString<T>, [Point<T>, MultiPoint<T>]);

impl<T> Covers<Line<T>> for LineString<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &Line<T>) -> bool {
        if rhs.start == rhs.end {
            // handle edge case where line might sit within a linestring's boundary
            self.covers(&rhs.start)
        } else {
            // remaining scenarios are eqivalent to contains
            self.contains(rhs)
        }
    }
}

impl<T> Covers<LineString<T>> for LineString<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &LineString<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        rhs.lines().all(|l| self.covers(&l))
    }
}

impl_covers_from_relate!(LineString<T>, [
MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

// valid because RHS is point or multipoint
// all exterior pts of RHS intersecting self means self covers RHS
impl_covers_from_intersects!(MultiLineString<T>, [Point<T>, MultiPoint<T>]);

impl_covers_from_relate!(MultiLineString<T>, [
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
        let s: LineString<f64> = wkt!(LINESTRING(0 0,1 1)).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));

        let s: MultiLineString<f64> = wkt!(MULTILINESTRING((0 0,1 1))).convert();
        assert!(!s.covers(&LineString::empty()));
        assert!(!s.covers(&Polygon::empty()));
        assert!(!s.covers(&MultiPoint::empty()));
        assert!(!s.covers(&MultiLineString::empty()));
        assert!(!s.covers(&MultiPolygon::empty()));
        assert!(!s.covers(&GeometryCollection::empty()));
    }

    #[test]
    fn test_lhs_empty() {
        let s: LineString<f64> = LineString::empty();
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

        let s: MultiLineString<f64> = MultiLineString::empty();
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

    #[test]
    fn differ_from_contains() {
        // test that the edge case is covered by custom implementation

        let ls: LineString<f64> = wkt!(LINESTRING(0 0, 1 1)).convert();
        let ls_s: LineString<f64> = wkt!(LINESTRING(0 0, 0 0)).convert();
        let ls_e: LineString<f64> = wkt!(LINESTRING(1 1, 1 1)).convert();
        let ln_s: Line<f64> = wkt!(LINE(0 0, 0 0)).convert();
        let ln_e: Line<f64> = wkt!(LINE(1 1, 1 1)).convert();
        let pt_s: Point<f64> = wkt!(POINT(0 0)).convert();
        let pt_e: Point<f64> = wkt!(POINT(1 1)).convert();

        assert!(ls.covers(&ls_s));
        assert!(ls.covers(&ls_e));
        assert!(ls.covers(&ln_s));
        assert!(ls.covers(&ln_e));
        assert!(ls.covers(&pt_s));
        assert!(ls.covers(&pt_e));

        assert!(!ls.contains(&ls_s));
        assert!(!ls.contains(&ls_e));
        assert!(!ls.contains(&ln_s));
        assert!(!ls.contains(&ln_e));
        assert!(!ls.contains(&pt_s));
        assert!(!ls.contains(&pt_e));
    }
}
