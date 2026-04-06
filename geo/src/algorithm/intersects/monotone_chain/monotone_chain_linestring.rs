use crate::MonotoneChains;
use crate::monotone_chain::geometry::*;
use crate::{GeoNum, HasDimensions, Intersects};

chains_intersects_impl!(
    MonotoneChainLineString<'a, T>,
    MonotoneChainLineString<'a, T>
);
chains_intersects_impl!(
    MonotoneChainLineString<'a, T>,
    MonotoneChainMultiLineString<'a, T>
);

impl<'a, T: GeoNum> Intersects<MonotoneChainPolygon<'a, T>> for MonotoneChainLineString<'a, T> {
    fn intersects(&self, rhs: &MonotoneChainPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        rhs.chains().any(|c| self.chain().intersects(c))
            || rhs.geometry().intersects(&self.geometry().0[0])
    }
}

impl<'a, T: GeoNum> Intersects<MonotoneChainMultiPolygon<'a, T>>
    for MonotoneChainLineString<'a, T>
{
    fn intersects(&self, rhs: &MonotoneChainMultiPolygon<'a, T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        rhs.chains().any(|c| self.chain().intersects(c))
            || rhs.geometry().intersects(&self.geometry().0[0])
    }
}
