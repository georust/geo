use crate::geometry::*;
use crate::monotone_chain::geometry::*;
use crate::monotone_chain::{MonotoneChain, MonotoneChains};
use crate::{GeoNum, HasDimensions, Intersects};

symmetric_intersects_impl!(
    MonotoneChainMultiPolygon<'a, T>,
    MonotoneChainLineString<'a, T>
);
symmetric_intersects_impl!(
    MonotoneChainMultiPolygon<'a, T>,
    MonotoneChainMultiLineString<'a, T>
);
symmetric_intersects_impl!(
    MonotoneChainMultiPolygon<'a, T>,
    MonotoneChainPolygon<'a, T>
);

impl<'a, T: GeoNum> Intersects<MonotoneChainMultiPolygon<'a, T>>
    for MonotoneChainMultiPolygon<'a, T>
where
    Coord<T>: Intersects<MultiPolygon<T>>,
    Coord<T>: Intersects<Polygon<T>>,
    MonotoneChain<'a, T>: Intersects<MonotoneChain<'a, T>>,
{
    fn intersects(&self, rhs: &MonotoneChainMultiPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        rhs.geometry()
            .0
            .iter()
            .filter_map(|c| c.exterior().0.first())
            .any(|c| c.intersects(self.geometry()))
            || self
                .geometry()
                .0
                .iter()
                .filter_map(|c| c.exterior().0.first())
                .any(|c| c.intersects(rhs.geometry()))
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
