use crate::MonotoneChains;
use crate::geometry::*;
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

// commented out if they are implemented by blanket impl in main `Intersects` trait

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, Coord<T>);
symmetric_intersects_impl!(Coord<T>, MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, Point<T>);
// symmetric_intersects_impl!(Point<T>,MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, MultiPoint<T>);
// symmetric_intersects_impl!(MultiPoint<T>,MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, Line<T>);
symmetric_intersects_impl!(Line<T>, MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, LineString<T>);
symmetric_intersects_impl!(LineString<T>, MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, MultiLineString<T>);
// symmetric_intersects_impl!(MultiLineString<T>,MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, Polygon<T>);
symmetric_intersects_impl!(Polygon<T>, MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, MultiPolygon<T>);
// symmetric_intersects_impl!(MultiPolygon<T>,MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, Rect<T>);
symmetric_intersects_impl!(Rect<T>, MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, Triangle<T>);
symmetric_intersects_impl!(Triangle<T>, MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, Geometry<T>);
// symmetric_intersects_impl!(Geometry<T>,MonotoneChainLineString<'a, T>);

delegate_intersects_impl!(MonotoneChainLineString<'a, T>, GeometryCollection<T>);
// symmetric_intersects_impl!(GeometryCollection<T>,MonotoneChainLineString<'a, T>);
