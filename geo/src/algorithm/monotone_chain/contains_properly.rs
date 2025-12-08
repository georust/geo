//! Implements `ContainsProperly` for MonotoneChainPolygon and MonotoneChainMultiPolygon

use super::{MonotoneChainMultiPolygon, MonotoneChainPolygon, MonotoneChains};
use crate::algorithm::contains_properly::polygon_polygon_inner_loop;
use crate::{ContainsProperly, GeoNum, HasDimensions, Intersects};

impl<'a, T: GeoNum> ContainsProperly<MonotoneChainPolygon<'a, T>> for MonotoneChainPolygon<'a, T> {
    fn contains_properly(&self, rhs: &MonotoneChainPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        // no boundary intersection
        if boundary_intersects::<T, MonotoneChainPolygon<'a, T>, MonotoneChainPolygon<'a, T>>(
            self, rhs,
        ) {
            return false;
        }
        // established that pairwise relation between any two rings is either concentric or disjoint

        polygon_polygon_inner_loop(self.geometry(), rhs.geometry())
    }
}

impl<'a, T: GeoNum> ContainsProperly<MonotoneChainMultiPolygon<'a, T>>
    for MonotoneChainPolygon<'a, T>
{
    fn contains_properly(&self, rhs: &MonotoneChainMultiPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        if boundary_intersects::<T, MonotoneChainPolygon<'a, T>, MonotoneChainMultiPolygon<'a, T>>(
            self, rhs,
        ) {
            return false;
        }
        // all rings are concentric or disjoint

        rhs.geometry()
            .iter()
            .filter(|poly| !poly.is_empty())
            .all(|rhs_poly| polygon_polygon_inner_loop(self.geometry(), rhs_poly))
    }
}

impl<'a, T: GeoNum> ContainsProperly<MonotoneChainPolygon<'a, T>>
    for MonotoneChainMultiPolygon<'a, T>
{
    fn contains_properly(&self, rhs: &MonotoneChainPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        if boundary_intersects::<T, MonotoneChainMultiPolygon<'a, T>, MonotoneChainPolygon<'a, T>>(
            self, rhs,
        ) {
            return false;
        }
        // all rings are concentric or disjoint

        let Some(rhs_ext_coord) = rhs.geometry().exterior().0.first() else {
            return false;
        };

        let mut self_candidates = self
            .geometry()
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

        polygon_polygon_inner_loop(self_candidate, rhs.geometry())
    }
}

impl<'a, T: GeoNum> ContainsProperly<MonotoneChainMultiPolygon<'a, T>>
    for MonotoneChainMultiPolygon<'a, T>
{
    fn contains_properly(&self, rhs: &MonotoneChainMultiPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            // there is at least one non-empty polygon in self and rhs
            return false;
        }

        if boundary_intersects::<
            T,
            MonotoneChainMultiPolygon<'a, T>,
            MonotoneChainMultiPolygon<'a, T>,
        >(self, rhs)
        {
            return false;
        }
        // all rings are concentric or disjoint

        // every rhs_poly must be covered by at some self_poly to return true
        for rhs_poly in rhs.geometry().0.iter().filter(|poly| !poly.is_empty()) {
            let rhs_poly_covered = self
                .geometry()
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

/// Return true if the boundary of lhs intersects any of the boundaries of rhs
/// where lhs and rhs are both polygons/multipolygons
/// This is a short circuit version of boundary_intersects which doesn't use the monotone algorithm
fn boundary_intersects<'a: 'caller, 'caller, T, G1, G2>(lhs: &'caller G1, rhs: &'caller G2) -> bool
where
    T: GeoNum + 'a,
    G1: MonotoneChains<'a, 'caller, T>,
    G2: MonotoneChains<'a, 'caller, T>,
{
    // use monotone when larger
    let rhs_arr = rhs.chains().collect::<Vec<_>>();

    lhs.chains()
        .any(|lhs| rhs_arr.iter().any(|rhs| lhs.intersects(*rhs)))
}
