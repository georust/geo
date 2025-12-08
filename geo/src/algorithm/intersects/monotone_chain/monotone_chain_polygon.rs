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

delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, Coord<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, Point<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, MultiPoint<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, Line<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, LineString<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, MultiLineString<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, Polygon<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, MultiPolygon<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, Rect<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, Triangle<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, Geometry<T>);
delegate_intersects_impl!(MonotoneChainPolygon<'a, T>, GeometryCollection<T>);

// commented out if they are implemented by blanket impl in main `Intersects` trait
symmetric_intersects_impl!(Coord<T>, MonotoneChainPolygon<'a, T>);
// symmetric_intersects_impl!(Point<T>,MonotoneChainPolygon<'a, T>);
// symmetric_intersects_impl!(MultiPoint<T>,MonotoneChainPolygon<'a, T>);

symmetric_intersects_impl!(Line<T>, MonotoneChainPolygon<'a, T>);
symmetric_intersects_impl!(LineString<T>, MonotoneChainPolygon<'a, T>);
// symmetric_intersects_impl!(MultiLineString<T>,MonotoneChainPolygon<'a, T>);

symmetric_intersects_impl!(Polygon<T>, MonotoneChainPolygon<'a, T>);
// symmetric_intersects_impl!(MultiPolygon<T>,MonotoneChainPolygon<'a, T>);
symmetric_intersects_impl!(Rect<T>, MonotoneChainPolygon<'a, T>);
symmetric_intersects_impl!(Triangle<T>, MonotoneChainPolygon<'a, T>);

// symmetric_intersects_impl!(Geometry<T>,MonotoneChainPolygon<'a, T>);
// symmetric_intersects_impl!(GeometryCollection<T>,MonotoneChainPolygon<'a, T>);
