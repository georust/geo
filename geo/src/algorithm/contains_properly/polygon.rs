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

        // if any point of rhs.exterior lies within self.exterior, then all points of rhs exterior lie within self.exterior
        let Some(rhs_ext_coord) = rhs.exterior().0.first() else {
            return false;
        };

        //check for disjoint
        if !coord_in_ring(rhs_ext_coord, self.exterior()) {
            return false;
        }

        // if there exits a self_hole which is not inside a rhs_hole
        // then there must be some point of rhs which does not lie on the interior of self
        // and hence self does not contains_properly rhs
        for self_hole in self.interiors() {
            // if self_hole is empty, then it is covered by rhs
            if !is_covered_hole(self_hole, rhs) {
                return false;
            }
        }
        true
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
        rhs.iter().all(|poly| self.contains_properly(poly))
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

        // check for disjoint within the loop
        // rhs will lie in at most one polygon of self
        // filtering by intersects is sufficient to identify this polygon
        // if there is no intersection, then rhs lies in no polygon of self and is disjoint
        // if there are multiple intersections, then self must have been self intersecting
        let mut is_disjoint = true;

        let candidates = self
            .0
            .iter()
            .filter(|poly| poly.contains_properly(rhs_ext_coord));

        // there should at most one candidate

        for self_poly in candidates {
            is_disjoint = false;

            for self_hole in self_poly.interiors() {
                if !is_covered_hole(self_hole, rhs) {
                    return false;
                }
            }
        }

        !is_disjoint
    }
}
impl<T> ContainsProperly<MultiPolygon<T>> for MultiPolygon<T>
where
    T: GeoNum,
{
    fn contains_properly(&self, rhs: &MultiPolygon<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        if boundary_intersects::<T, MultiPolygon<T>, MultiPolygon<T>>(self, rhs) {
            return false;
        }

        for rhs_poly in rhs.0.iter() {
            // if any point of rhs exterior lies within self.exterior, then all points of rhs exterior lie within self.exterior
            let Some(rhs_ext_coord) = rhs_poly.exterior().0.first() else {
                return false;
            };
            // check for disjoint per rhs_poly
            let mut is_disjoint = true;
            let candidates = self
                .0
                .iter()
                .filter(|poly| poly.contains_properly(rhs_ext_coord));

            for self_poly in candidates {
                is_disjoint = false;
                for self_hole in self_poly.interiors() {
                    if !is_covered_hole(self_hole, rhs_poly) {
                        return false;
                    }
                }
            }

            if is_disjoint {
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

/// Returns true if self_hole is inside an RHS hole
/// ~ if self_hole is not inside an rhs hole, then part of RHS is outside of self  
fn is_covered_hole<T>(self_hole: &LineString<T>, rhs: &Polygon<T>) -> bool
where
    T: GeoNum,
{
    // empty hole is always covered
    let Some(self_hole_first_coord) = self_hole.0.first() else {
        return true;
    };
    // no hole in rhs means hole is covered
    if rhs.interiors().is_empty() {
        return true;
    }

    // hole outside of RHS does not affect intersection
    if coord_pos_relative_to_ring(*self_hole_first_coord, rhs.exterior()) != CoordPos::Inside {
        return true;
    }

    // since all rings are either concentric or disjoint, we can check using represenative point
    rhs.interiors()
        .iter()
        .map(|ring| coord_pos_relative_to_ring(*self_hole_first_coord, ring))
        .any(|pos| pos == CoordPos::Inside)
}

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

fn coord_in_ring<T>(coord: &Coord<T>, ring: &LineString<T>) -> bool
where
    T: GeoNum,
{
    coord_pos_relative_to_ring(*coord, ring) == CoordPos::Inside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Convert;
    use crate::wkt;
    use crate::{MultiPolygon, Polygon};

    #[test]
    fn test_contains_properly_donut() {
        let poly1: Polygon<f64> =
            wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0),(6 3,6 6,3 6,3 3,6 3))}.convert();
        let poly2: Polygon<f64> =
            wkt! {POLYGON((8 1,8 8,1 8,1 1,8 1),(7 2,7 7,2 7,2 2,7 2))}.convert();

        assert!(poly1.contains_properly(&poly2));
    }

    #[test]
    fn test_contains_properly_donut2() {
        let poly1: Polygon<f64> =
            wkt! {POLYGON((9 0,9 9,0 9,0 0,9 0),(8 7,8 8,7 8,7 7,8 7))}.convert();
        let poly2: Polygon<f64> =
            wkt! {POLYGON((6 1,6 6,1 6,1 1,6 1),(3 2,3 3,2 3,2 2,3 2))}.convert();

        assert!(poly1.contains_properly(&poly2));
    }

    #[test]
    fn test_contains_properly_donut_multi_multi() {
        let poly1: MultiPolygon<f64> =
            wkt! {MULTIPOLYGON(((9 0,9 9,0 9,0 0,9 0),(6 3,6 6,3 6,3 3,6 3)))}.convert();
        let poly2: MultiPolygon<f64> =
            wkt! {MULTIPOLYGON(((8 1,8 8,1 8,1 1,8 1),(7 2,7 7,2 7,2 2,7 2)))}.convert();

        assert!(poly1.contains_properly(&poly2));
    }

    #[test]
    fn test_contains_properly_donut_multi_poly() {
        let mp: MultiPolygon<f64> = wkt!{MULTIPOLYGON(((9 0,9 9,0 9,0 0,9 0),(8 1,8 8,1 8,1 1,8 1)),((7 2,7 7,2 7,2 2,7 2)))}.convert();
        let poly2: Polygon<f64> = wkt! {POLYGON((6 3,6 6,3 6,3 3,6 3))}.convert();

        assert!(mp.contains_properly(&poly2));
    }
}
