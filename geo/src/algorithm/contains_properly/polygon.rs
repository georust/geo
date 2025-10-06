use super::{ContainsProperly, impl_contains_properly_from_relate};
use crate::CoordsIter;
use crate::HasDimensions;
use crate::Intersects;
use crate::LinesIter;
use crate::coordinate_position::{CoordPos, CoordinatePosition, coord_pos_relative_to_ring};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

impl<T> ContainsProperly<Coord<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Coord<T>) -> bool {
        if self.is_empty() {
            return false;
        }
        self.coordinate_position(rhs) == CoordPos::Inside
    }
}

impl<T> ContainsProperly<Point<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Point<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        self.contains_properly(&rhs.0)
    }
}

impl<T> ContainsProperly<MultiPoint<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &MultiPoint<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        rhs.coords_iter().all(|p| self.contains_properly(&p))
    }
}

impl<T> ContainsProperly<Polygon<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Polygon<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        // no boundary intersection
        if boundary_intersects::<T, Polygon<T>, Polygon<T>>(self, rhs) {
            return false;
        }
        // established that pairwise relation betwwen any two rings is either concentric or disjoint

        polygon_polygon_inner_loop(self, rhs)
    }
}

impl<T> ContainsProperly<MultiPolygon<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &MultiPolygon<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        if boundary_intersects::<T, Polygon<T>, MultiPolygon<T>>(self, rhs) {
            return false;
        }
        // all rings are concentric or disjoint

        rhs.iter()
            .filter(|poly| !poly.is_empty())
            .all(|rhs_poly| polygon_polygon_inner_loop(self, rhs_poly))
    }
}

impl<T> ContainsProperly<Rect<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Rect<T>) -> bool {
        self.contains_properly(&rhs.to_polygon())
    }
}
impl<T> ContainsProperly<Triangle<T>> for Polygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Triangle<T>) -> bool {
        self.contains_properly(&rhs.to_polygon())
    }
}

impl_contains_properly_from_relate!(Polygon<T>, [Line<T>, LineString<T>, MultiLineString<T>,GeometryCollection<T>]);

impl<T> ContainsProperly<Polygon<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Polygon<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        if boundary_intersects::<T, MultiPolygon<T>, Polygon<T>>(self, rhs) {
            return false;
        }
        // all rings are concentric or disjoint

        let Some(rhs_ext_coord) = rhs.exterior().0.first() else {
            return false;
        };

        let mut self_candidates = self
            .0
            .iter()
            .filter(|poly| poly.contains_properly(rhs_ext_coord));

        /* There will be at most one candidate
         *
         * This is because:
         * 1. sub-polygons of a multi-polygon are disjoint
         * 2. therefore, for rhs_poly to intersect multiple polygons of self_poly, it must cross some boundary of sub-polygons of self_poly
         * 3. however, since all rings are either concentric or disjoint, there can be no boundary intersection
         * Therefore rhs_poly can lie in at most one sub-polygon of self_poly
         */
        debug_assert!(self_candidates.clone().count() <= 1);

        let Some(self_candidate) = self_candidates.next() else {
            // disjoint
            return false;
        };

        polygon_polygon_inner_loop(self_candidate, rhs)
    }
}

impl<T> ContainsProperly<MultiPolygon<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &MultiPolygon<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            // there is at least one non-empty polygon in self and rhs
            return false;
        }

        if boundary_intersects::<T, MultiPolygon<T>, MultiPolygon<T>>(self, rhs) {
            return false;
        }
        // all rings are concentric or disjoint

        // every rhs_poly must be covered by at some self_poly to return true
        for rhs_poly in rhs.0.iter().filter(|poly| !poly.is_empty()) {
            let rhs_poly_covered = self
                .0
                .iter()
                .filter(|poly| !poly.is_empty())
                .any(|self_poly| polygon_polygon_inner_loop(self_poly, rhs_poly));

            if !rhs_poly_covered {
                println!("oh");
                return false;
            }
        }

        true
    }
}

impl<T> ContainsProperly<Rect<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Rect<T>) -> bool {
        self.contains_properly(&rhs.to_polygon())
    }
}
impl<T> ContainsProperly<Triangle<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &Triangle<T>) -> bool {
        self.contains_properly(&rhs.to_polygon())
    }
}

impl_contains_properly_from_relate!(MultiPolygon<T>, [Point<T>,MultiPoint<T>,Line<T>, LineString<T>, MultiLineString<T>,GeometryCollection<T>]);

//------------------------------------------------------------------------------
// Util functions
//------------------------------------------------------------------------------

/// Return true if the boundary of lhs intersects any of the boundaries of rhs
/// where lhs and rhs are both polygons/multipolygons
fn boundary_intersects<'a, T, G1, G2>(lhs: &'a G1, rhs: &'a G2) -> bool
where
    T: GeoNum,
    G1: LinesIter<'a, Scalar = T>,
    G2: LinesIter<'a, Scalar = T>,
    Line<T>: Intersects<Line<T>>,
{
    lhs.lines_iter()
        .flat_map(|self_l| rhs.lines_iter().map(move |rhs_l| (self_l, rhs_l)))
        .any(|(self_l, rhs_l)| self_l.intersects(&rhs_l))
}

/// Given two non-empty polygons with no intersecting boundaries,
/// Return true if first polygon completely contains second polygon
fn polygon_polygon_inner_loop<T>(self_poly: &Polygon<T>, rhs_poly: &Polygon<T>) -> bool
where
    T: GeoNum,
{
    debug_assert!(!self_poly.is_empty() && !rhs_poly.is_empty());
    debug_assert!(
        self_poly
            .lines_iter()
            .flat_map(|s| rhs_poly.lines_iter().map(move |s2| (s, s2)))
            .all(|(s, s2)| !s.intersects(&s2))
    );

    let Some(rhs_ext_coord) = rhs_poly.exterior().0.first() else {
        return false;
    };

    if !self_poly.contains_properly(rhs_ext_coord) {
        // is disjoint
        return false;
    }

    // if there exits a self_hole which is not inside a rhs_hole
    // then there must be some point of rhs which does not lie on the interior of self
    // and hence self does not contains_properly rhs
    for self_hole in self_poly.interiors() {
        // empty hole is always covered
        let Some(self_hole_first_coord) = self_hole.0.first() else {
            continue;
        };

        // hole outside of RHS does not affect intersection
        if coord_pos_relative_to_ring(*self_hole_first_coord, rhs_poly.exterior())
            != CoordPos::Inside
        {
            continue;
        }

        // if any RHS point is inside self_hole, then we fail the contains_properly test
        // since all rings are either concentric or disjoint, we can check using representative point
        let self_hole_pt_in_rhs_hole = rhs_poly
            .interiors()
            .iter()
            .map(|rhs_ring| coord_pos_relative_to_ring(*self_hole_first_coord, rhs_ring))
            .any(|pos| pos == CoordPos::Inside);
        if !self_hole_pt_in_rhs_hole {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use crate::wkt;
    use crate::{ContainsProperly, Convert, Relate};
    use crate::{LineString, MultiPolygon, Polygon};

    #[test]
    fn test_contains_properly_donut() {
        let poly1: Polygon<f64> =
            wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0),(6 3,6 6,3 6,3 3,6 3))}.convert();
        let poly2: Polygon<f64> =
            wkt! {POLYGON((8 1,8 8,1 8,1 1,8 1),(7 2,7 7,2 7,2 2,7 2))}.convert();

        assert_eq!(
            poly1.contains_properly(&poly2),
            poly1.relate(&poly2).is_contains_properly()
        );
        assert!(poly1.contains_properly(&poly2));
    }

    #[test]
    fn test_contains_properly_donut2() {
        let poly1: Polygon<f64> =
            wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0),(8 7,8 8,7 8,7 7,8 7))}.convert();
        let poly2: Polygon<f64> =
            wkt! {POLYGON((6 1,6 6,1 6,1 1,6 1),(3 2,3 3,2 3,2 2,3 2))}.convert();

        assert_eq!(
            poly1.contains_properly(&poly2),
            poly1.relate(&poly2).is_contains_properly()
        );
        assert!(poly1.contains_properly(&poly2));
    }

    #[test]
    fn test_contains_properly_in_donut_hole() {
        let poly1: Polygon<f64> =
            wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0),(6 3,6 6,3 6,3 3,6 3))}.convert();
        let poly2: Polygon<f64> =
            wkt! {POLYGON((7 4,7 7,4 7,4 4,7 4),(6 5,6 6,5 6,5 5,6 5))}.convert();
        let poly3: Polygon<f64> = wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0))}.convert();

        assert_eq!(
            poly1.contains_properly(&poly2),
            poly1.relate(&poly2).is_contains_properly()
        );
        assert!(!poly1.contains_properly(&poly2));

        assert_eq!(
            poly1.contains_properly(&poly3),
            poly1.relate(&poly3).is_contains_properly()
        );
        assert!(!poly1.contains_properly(&poly3));
    }

    #[test]
    fn test_contains_properly_degenerate_cross_boundary() {
        let degenerate_poly: Polygon<f64> = wkt! {POLYGON((2 2,8 8,2 2))}.convert();
        let ls: LineString<f64> = wkt! {LINESTRING(2 2,8 8)}.convert();
        let mp: MultiPolygon<f64> =
            wkt! {MULTIPOLYGON(((5 1,5 5,1 5,1 1,5 1)),((9 5,9 9,5 9,5 5,9 5)))}.convert();

        assert_eq!(
            mp.contains_properly(&ls),
            mp.relate(&ls).is_contains_properly()
        );
        assert_eq!(
            mp.contains_properly(&degenerate_poly),
            mp.relate(&degenerate_poly).is_contains_properly()
        );
    }

    #[test]
    fn test_contains_properly_donut_multi_multi() {
        let poly1: MultiPolygon<f64> =
            wkt! {MULTIPOLYGON(((9 0,9 9,0 9,0 0,9 0),(6 3,6 6,3 6,3 3,6 3)))}.convert();
        let poly2: MultiPolygon<f64> =
            wkt! {MULTIPOLYGON(((8 1,8 8,1 8,1 1,8 1),(7 2,7 7,2 7,2 2,7 2)))}.convert();

        assert_eq!(
            poly1.contains_properly(&poly2),
            poly1.relate(&poly2).is_contains_properly()
        );
        assert!(poly1.contains_properly(&poly2));
    }

    #[test]
    fn test_contains_properly_donut_multi_poly() {
        let mp: MultiPolygon<f64> = wkt!{MULTIPOLYGON(((9 0,9 9,0 9,0 0,9 0),(8 1,8 8,1 8,1 1,8 1)),((7 2,7 7,2 7,2 2,7 2)))}.convert();
        let poly2: Polygon<f64> = wkt! {POLYGON((6 3,6 6,3 6,3 3,6 3))}.convert();

        assert_eq!(
            mp.contains_properly(&poly2),
            mp.relate(&poly2).is_contains_properly()
        );
        assert!(mp.contains_properly(&poly2));
    }

    #[test]
    fn test_mp_with_empty_part() {
        let empty_p: Polygon<f64> = wkt! {POLYGON(EMPTY)};
        let empty_mp: MultiPolygon<f64> = wkt! {MULTIPOLYGON(EMPTY)};
        let p_outer: Polygon<f64> = wkt! {POLYGON((0 0,0 9,9 9,9 0,0 0))}.convert();
        let p_inner: Polygon<f64> = wkt! {POLYGON((1 1,1 8,8 8,8 1,1 1))}.convert();

        let mp = MultiPolygon::new(vec![p_inner.clone(), empty_p.clone()]);
        let mp2 = MultiPolygon::new(vec![empty_p.clone()]);

        assert_eq!(
            p_outer.contains_properly(&empty_p),
            p_outer.relate(&empty_p).is_contains_properly()
        );
        assert_eq!(
            p_outer.contains_properly(&empty_mp),
            p_outer.relate(&empty_mp).is_contains_properly()
        );
        // mp with only empty parts is empty ==> should fail
        assert_eq!(
            p_outer.contains_properly(&mp2),
            p_outer.relate(&mp2).is_contains_properly()
        );

        assert_eq!(
            p_outer.contains_properly(&p_inner),
            p_outer.relate(&p_inner).is_contains_properly()
        );
        assert_eq!(
            p_outer.contains_properly(&mp),
            p_outer.relate(&mp).is_contains_properly()
        );
    }

    #[test]
    fn aa() {
        let p1: Polygon<f64> = wkt! {
              POLYGON(
        (40 60, 420 60, 420 320, 40 320, 40 60),
        (200 140, 160 220, 260 200, 200 140))
              }
        .convert();
        let p2: Polygon<f64> = wkt! {
        POLYGON(
          (80 100, 360 100, 360 280, 80 280, 80 100))
        }
        .convert();

        assert_eq!(
            p1.contains_properly(&p2),
            p1.relate(&p2).is_contains_properly()
        );
        assert_eq!(
            p2.contains_properly(&p1),
            p2.relate(&p1).is_contains_properly()
        );
    }
}
