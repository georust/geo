use crate::geometry::*;
use crate::monotone_chain::geometry::*;
use crate::monotone_chain::{MonotoneChain, MonotoneChains};
use crate::{GeoNum, HasDimensions, Intersects};

symmetric_intersects_impl!(MonotoneChainPolygon<'a, T>, MonotoneChainLineString<'a, T>);
symmetric_intersects_impl!(
    MonotoneChainPolygon<'a, T>,
    MonotoneChainMultiLineString<'a, T>
);

impl<'a, T: GeoNum> Intersects<MonotoneChainPolygon<'a, T>> for MonotoneChainPolygon<'a, T>
where
    Coord<T>: Intersects<Polygon<T>>,
    MonotoneChain<'a, T>: Intersects<MonotoneChain<'a, T>>,
{
    fn intersects(&self, rhs: &MonotoneChainPolygon<'a, T>) -> bool
where {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }

        self.geometry().exterior().0[0].intersects(rhs.geometry())
            || rhs.geometry().exterior().0[0].intersects(self.geometry())
            || self
                .exterior_chains()
                .any(|c| rhs.exterior_chains().any(|c2| c.intersects(c2)))
            || self
                .interior_chains()
                .any(|c| rhs.exterior_chains().any(|c2| c.intersects(c2)))
            || self
                .exterior_chains()
                .any(|c| rhs.interior_chains().any(|c2| c.intersects(c2)))
    }
}

impl<'a, T: GeoNum> Intersects<MonotoneChainMultiPolygon<'a, T>> for MonotoneChainPolygon<'a, T>
where
    Coord<T>: Intersects<MultiPolygon<T>>,
    Coord<T>: Intersects<Polygon<T>>,
    MonotoneChain<'a, T>: Intersects<MonotoneChain<'a, T>>,
{
    fn intersects(&self, rhs: &MonotoneChainMultiPolygon<'a, T>) -> bool
where {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        rhs.geometry()
            .0
            .iter()
            .filter_map(|c| c.exterior().0.first())
            .any(|c| c.intersects(self.geometry()))
            || self.geometry().exterior().0[0].intersects(rhs.geometry())
            || self
                .exterior_chains()
                .any(|c| rhs.exterior_chains().any(|c2| c.intersects(c2)))
            || self
                .interior_chains()
                .any(|c| rhs.exterior_chains().any(|c2| c.intersects(c2)))
            || self
                .exterior_chains()
                .any(|c| rhs.interior_chains().any(|c2| c.intersects(c2)))
    }
}
