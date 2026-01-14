//! Implements `ContainsProperly` for MonotoneChainPolygon and MonotoneChainMultiPolygon

use super::boundary_intersects;
use crate::algorithm::contains_properly::polygon_polygon_inner_loop;
use crate::monotone_chain::geometry::*;
use crate::{ContainsProperly, GeoNum, HasDimensions};

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
