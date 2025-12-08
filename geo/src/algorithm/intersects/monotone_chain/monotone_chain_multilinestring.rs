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

delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, Coord<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, Point<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, MultiPoint<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, Line<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, LineString<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, MultiLineString<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, Polygon<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, MultiPolygon<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, Rect<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, Triangle<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, Geometry<T>);
delegate_intersects_impl!(MonotoneChainMultiLineString<'a, T>, GeometryCollection<T>);

// commented out if they are implemented by blanket impl in main `Intersects` trait
symmetric_intersects_impl!(Coord<T>, MonotoneChainMultiLineString<'a, T>);
// symmetric_intersects_impl!(Point<T>,MonotoneChainMultiLineString<'a, T>);
// symmetric_intersects_impl!(MultiPoint<T>,MonotoneChainMultiLineString<'a, T>);

symmetric_intersects_impl!(Line<T>, MonotoneChainMultiLineString<'a, T>);
symmetric_intersects_impl!(LineString<T>, MonotoneChainMultiLineString<'a, T>);
// symmetric_intersects_impl!(MultiLineString<T>,MonotoneChainMultiLineString<'a, T>);

symmetric_intersects_impl!(Polygon<T>, MonotoneChainMultiLineString<'a, T>);
// symmetric_intersects_impl!(MultiPolygon<T>,MonotoneChainMultiLineString<'a, T>);
symmetric_intersects_impl!(Rect<T>, MonotoneChainMultiLineString<'a, T>);
symmetric_intersects_impl!(Triangle<T>, MonotoneChainMultiLineString<'a, T>);

// symmetric_intersects_impl!(Geometry<T>,MonotoneChainMultiLineString<'a, T>);
// symmetric_intersects_impl!(GeometryCollection<T>,MonotoneChainMultiLineString<'a, T>);
