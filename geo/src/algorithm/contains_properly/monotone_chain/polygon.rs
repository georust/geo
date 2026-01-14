use super::boundary_intersects;
use crate::algorithm::contains_properly::polygon_polygon_inner_loop;
use crate::monotone_chain::geometry::*;
use crate::{ContainsProperly, GeoNum, HasDimensions};

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
