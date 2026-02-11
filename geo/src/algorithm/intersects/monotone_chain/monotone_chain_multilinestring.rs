use crate::MonotoneChains;
use crate::geometry::*;
use crate::monotone_chain::geometry::*;
use crate::{GeoNum, HasDimensions, Intersects};

chains_intersects_impl!(
    MonotoneChainMultiLineString<'a, T>,
    MonotoneChainLineString<'a, T>
);
chains_intersects_impl!(
    MonotoneChainMultiLineString<'a, T>,
    MonotoneChainMultiLineString<'a, T>
);

impl<'a, T> Intersects<MonotoneChainPolygon<'a, T>> for MonotoneChainMultiLineString<'a, T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &MonotoneChainPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        self.chains()
            .any(|c| rhs.chains().any(|c2| c.intersects(c2)))
            || self
                .geometry()
                .iter()
                .filter_map(|c| c.0.first())
                .any(|c| c.intersects(rhs.geometry()))
    }
}

impl<'a, T> Intersects<MonotoneChainMultiPolygon<'a, T>> for MonotoneChainMultiLineString<'a, T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &MonotoneChainMultiPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        self.chains()
            .any(|c| rhs.chains().any(|c2| c.intersects(c2)))
            || self
                .geometry()
                .iter()
                .filter_map(|c| c.0.first())
                .any(|c| c.intersects(rhs.geometry()))
    }
}
