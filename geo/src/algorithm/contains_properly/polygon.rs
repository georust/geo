use super::{ContainsProperly, impl_contains_properly_from_relate};
use crate::coordinate_position::{CoordPos, CoordinatePosition, coord_pos_relative_to_ring};
use crate::geometry::*;
use crate::{BoundingRect, CoordsIter, HasDimensions, Intersects, LinesIter};
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
        if self.is_empty() {
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
        // established that pairwise relation between any two rings is either concentric or disjoint

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
    Rect<T>: Intersects<Rect<T>>,
{
    let rhs_bbox_cache = rhs
        .lines_iter()
        .map(|l| (l, l.bounding_rect()))
        .collect::<Vec<(Line<T>, Rect<T>)>>();

    return lhs
        .lines_iter()
        .map(|l| (l, l.bounding_rect()))
        .any(|(l, l_bbox)| {
            rhs_bbox_cache
                .iter()
                .any(|(r, r_bbox)| l_bbox.intersects(r_bbox) && l.intersects(r))
        });
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
    use crate::{MultiPolygon, Point, Polygon};

    // basic pairwise test
    #[test]
    fn test_poly_for_poly() {
        let base: Polygon<f64> =
            wkt! {POLYGON((90 0,90 90,0 90,0 0,90 0),(60 30,60 60,30 60,30 30,60 30))}.convert();

        let donut_inside: Polygon<f64> =
            wkt! {POLYGON((80 10,80 80,10 80,10 10,80 10),(70 20,70 70,20 70,20 20,70 20))}
                .convert();
        let fully_inside: Polygon<f64> = wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10))}.convert();
        let fully_inside2: Polygon<f64> =
            wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10),(19 11,19 19,11 19,11 11,19 11))}
                .convert();
        let in_hole: Polygon<f64> = wkt! {POLYGON((50 40,50 50,40 50,40 40,50 40))}.convert();
        let disjoint: Polygon<f64> =
            wkt! {POLYGON((150 140,150 150,140 150,140 140,150 140))}.convert();

        assert_eq!(
            base.contains_properly(&donut_inside),
            base.relate(&donut_inside).is_contains_properly()
        );
        assert!(base.contains_properly(&donut_inside));
        assert_eq!(
            base.contains_properly(&fully_inside),
            base.relate(&fully_inside).is_contains_properly()
        );
        assert!(base.contains_properly(&fully_inside));

        assert_eq!(
            base.contains_properly(&fully_inside2),
            base.relate(&fully_inside).is_contains_properly()
        );
        assert!(base.contains_properly(&fully_inside2));

        assert_eq!(
            base.contains_properly(&in_hole),
            base.relate(&in_hole).is_contains_properly()
        );
        assert!(!base.contains_properly(&in_hole));

        assert_eq!(
            base.contains_properly(&disjoint),
            base.relate(&disjoint).is_contains_properly()
        );
        assert!(!base.contains_properly(&disjoint));
    }

    #[test]
    fn test_multipoly_for_poly() {
        let base: Polygon<f64> =
            wkt! {POLYGON((90 0,90 90,0 90,0 0,90 0),(60 30,60 60,30 60,30 30,60 30))}.convert();

        let donut_inside: Polygon<f64> =
            wkt! {POLYGON((80 10,80 80,10 80,10 10,80 10),(70 20,70 70,20 70,20 20,70 20))}
                .convert();
        let fully_inside: Polygon<f64> = wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10))}.convert();
        let fully_inside2: Polygon<f64> =
            wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10),(19 11,19 19,11 19,11 11,19 11))}
                .convert();
        let in_hole: Polygon<f64> = wkt! {POLYGON((50 40,50 50,40 50,40 40,50 40))}.convert();
        let disjoint: Polygon<f64> =
            wkt! {POLYGON((150 140,150 150,140 150,140 140,150 140))}.convert();

        let mp1 = MultiPolygon::new(vec![
            donut_inside.clone(),
            fully_inside.clone(),
            fully_inside2.clone(),
        ]);
        let mp2 = MultiPolygon::new(vec![
            donut_inside.clone(),
            fully_inside.clone(),
            in_hole.clone(),
        ]);
        let mp3 = MultiPolygon::new(vec![
            donut_inside.clone(),
            fully_inside.clone(),
            disjoint.clone(),
        ]);

        assert_eq!(
            base.contains_properly(&mp1),
            base.relate(&mp1).is_contains_properly()
        );
        assert!(base.contains_properly(&mp1));

        assert_eq!(
            base.contains_properly(&mp2),
            base.relate(&mp2).is_contains_properly()
        );
        assert!(!base.contains_properly(&mp2));

        assert_eq!(
            base.contains_properly(&mp3),
            base.relate(&mp3).is_contains_properly()
        );
        assert!(!base.contains_properly(&mp3));
    }

    // test against a MultiPolygon of two concentric donuts
    #[test]
    fn test_poly_for_multipoly() {
        let base: Polygon<f64> =
            wkt! {POLYGON((90 0,90 90,0 90,0 0,90 0),(60 30,60 60,30 60,30 30,60 30))}.convert();
        let base_2: Polygon<f64> =
            wkt! {POLYGON((59 31,59 59,31 59,31 31,59 31),(55 35,55 55,35 55,35 35,55 35))}
                .convert();

        // two concentric donuts
        let mp_base = MultiPolygon::new(vec![base.clone(), base_2.clone()]);

        // should succeed
        let donut_inside: Polygon<f64> =
            wkt! {POLYGON((80 10,80 80,10 80,10 10,80 10),(70 20,70 70,20 70,20 20,70 20))}
                .convert();
        let fully_inside: Polygon<f64> = wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10))}.convert();
        let fully_inside2: Polygon<f64> =
            wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10),(19 11,19 19,11 19,11 11,19 11))}
                .convert();
        // should fail
        let in_hole: Polygon<f64> = wkt! {POLYGON((50 40,50 50,40 50,40 40,50 40))}.convert();
        let disjoint: Polygon<f64> =
            wkt! {POLYGON((150 140,150 150,140 150,140 140,150 140))}.convert();

        assert_eq!(
            mp_base.contains_properly(&donut_inside),
            mp_base.relate(&donut_inside).is_contains_properly()
        );
        assert!(mp_base.contains_properly(&donut_inside));

        assert_eq!(
            mp_base.contains_properly(&fully_inside),
            mp_base.relate(&fully_inside).is_contains_properly()
        );
        assert!(mp_base.contains_properly(&fully_inside));

        assert_eq!(
            mp_base.contains_properly(&fully_inside2),
            mp_base.relate(&fully_inside).is_contains_properly()
        );
        assert!(mp_base.contains_properly(&fully_inside2));

        assert_eq!(
            mp_base.contains_properly(&in_hole),
            mp_base.relate(&in_hole).is_contains_properly()
        );
        assert!(!mp_base.contains_properly(&in_hole));

        assert_eq!(
            mp_base.contains_properly(&disjoint),
            mp_base.relate(&disjoint).is_contains_properly()
        );
        assert!(!mp_base.contains_properly(&disjoint));
    }

    // test against a MultiPolygon of two concentric donuts
    #[test]
    fn test_multipoly_for_multipoly() {
        let base: Polygon<f64> =
            wkt! {POLYGON((90 0,90 90,0 90,0 0,90 0),(60 30,60 60,30 60,30 30,60 30))}.convert();
        let base_2: Polygon<f64> =
            wkt! {POLYGON((59 31,59 59,31 59,31 31,59 31),(55 35,55 55,35 55,35 35,55 35))}
                .convert();
        // two concentric donuts
        let mp_base = MultiPolygon::new(vec![base.clone(), base_2.clone()]);

        // should succeed
        let donut_inside: Polygon<f64> =
            wkt! {POLYGON((80 10,80 80,10 80,10 10,80 10),(70 20,70 70,20 70,20 20,70 20))}
                .convert();
        let fully_inside: Polygon<f64> = wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10))}.convert();
        let fully_inside2: Polygon<f64> =
            wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10),(19 11,19 19,11 19,11 11,19 11))}
                .convert();
        // should fail
        let in_hole: Polygon<f64> = wkt! {POLYGON((50 40,50 50,40 50,40 40,50 40))}.convert();
        let disjoint: Polygon<f64> =
            wkt! {POLYGON((150 140,150 150,140 150,140 140,150 140))}.convert();

        let mp1 = MultiPolygon::new(vec![
            donut_inside.clone(),
            fully_inside.clone(),
            fully_inside2.clone(),
        ]);
        let mp2 = MultiPolygon::new(vec![
            donut_inside.clone(),
            fully_inside.clone(),
            fully_inside2.clone(),
            in_hole.clone(),
        ]);
        let mp3 = MultiPolygon::new(vec![
            donut_inside.clone(),
            fully_inside.clone(),
            fully_inside2.clone(),
            disjoint.clone(),
        ]);

        assert_eq!(
            mp_base.contains_properly(&mp1),
            mp_base.relate(&mp1).is_contains_properly()
        );
        assert!(mp_base.contains_properly(&mp1));

        assert_eq!(
            mp_base.contains_properly(&mp2),
            mp_base.relate(&mp2).is_contains_properly()
        );
        assert!(!mp_base.contains_properly(&mp2));

        assert_eq!(
            mp_base.contains_properly(&mp3),
            mp_base.relate(&mp3).is_contains_properly()
        );
        assert!(!mp_base.contains_properly(&mp3));
    }

    #[test]
    fn empty_parts() {
        let base: Polygon<f64> =
            wkt! {POLYGON((90 0,90 90,0 90,0 0,90 0),(60 30,60 60,30 60,30 30,60 30))}.convert();
        let base_2: Polygon<f64> =
            wkt! {POLYGON((59 31,59 59,31 59,31 31,59 31),(55 35,55 55,35 55,35 35,55 35))}
                .convert();
        // two concentric donuts
        let mp_base = MultiPolygon::new(vec![base.clone(), base_2.clone()]);

        let fully_inside: Polygon<f64> = wkt! {POLYGON((20 10,20 20,10 20,10 10,20 10))}.convert();
        let disjoint: Polygon<f64> =
            wkt! {POLYGON((150 140,150 150,140 150,140 140,150 140))}.convert();

        let empty_poly = Polygon::empty();
        let empty_mp1 = MultiPolygon::empty();
        let empty_mp2 = MultiPolygon::from(vec![Polygon::empty()]);

        let mp1 = MultiPolygon::new(vec![fully_inside.clone(), Polygon::empty()]);
        let mp2 = MultiPolygon::new(vec![disjoint.clone(), Polygon::empty()]);

        // empty Polygon
        assert!(!base.contains_properly(&empty_poly));
        assert!(!mp_base.contains_properly(&empty_poly));
        // empty MultiPolygon
        assert!(!base.contains_properly(&empty_mp1));
        assert!(!base.contains_properly(&empty_mp2));
        assert!(!mp_base.contains_properly(&empty_mp1));
        assert!(!mp_base.contains_properly(&empty_mp2));

        // multipolygon with empty part
        assert!(base.contains_properly(&mp1));
        assert!(mp_base.contains_properly(&mp1));
        assert!(!base.contains_properly(&mp2));
        assert!(!mp_base.contains_properly(&mp2));
    }

    // degenerate polygon should always return false
    // because it would be intersecting the boundary of the degenerate polygon
    #[test]
    fn test_degenerate_self() {
        let degenerate_poly_as_pt: Polygon<f64> = wkt! {POLYGON((0 0, 0 0, 0 0, 0 0))}.convert();
        let pt: Point<f64> = wkt! {POINT(0 0)}.convert();
        assert!(!degenerate_poly_as_pt.relate(&pt).is_contains_properly());
    }

    #[test]
    fn test_degenerate_other() {
        let base: Polygon<f64> = wkt! {POLYGON((90 0,90 90,0 90,0 0,90 0))}.convert();
        let degenerate_poly_as_pt: Polygon<f64> = wkt! {POLYGON((1 1, 1 1, 1 1, 1 1))}.convert();
        let degenerate_poly_as_ls: Polygon<f64> = wkt! {POLYGON((1 1, 2 2, 1 1))}.convert();
        assert!(base.relate(&degenerate_poly_as_pt).is_contains_properly());
        assert!(base.relate(&degenerate_poly_as_ls).is_contains_properly());
    }
}
