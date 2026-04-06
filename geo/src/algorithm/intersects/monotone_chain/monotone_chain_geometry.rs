use crate::monotone_chain::MonotoneChainGeometry;
use crate::monotone_chain::geometry::*;
use crate::{GeoNum, Intersects};

impl<'a, G, T> Intersects<G> for MonotoneChainGeometry<'a, T>
where
    G: Intersects<MonotoneChainLineString<'a, T>>
        + Intersects<MonotoneChainMultiLineString<'a, T>>
        + Intersects<MonotoneChainPolygon<'a, T>>
        + Intersects<MonotoneChainMultiPolygon<'a, T>>,
    T: GeoNum + 'a,
{
    fn intersects(&self, rhs: &G) -> bool {
        match self {
            MonotoneChainGeometry::LineString(a) => rhs.intersects(a),
            MonotoneChainGeometry::MultiLineString(a) => rhs.intersects(a),
            MonotoneChainGeometry::Polygon(a) => rhs.intersects(a),
            MonotoneChainGeometry::MultiPolygon(a) => rhs.intersects(a),
        }
    }
}

symmetric_intersects_impl!(MonotoneChainLineString<'a, T>, MonotoneChainGeometry<'a, T>);
symmetric_intersects_impl!(
    MonotoneChainMultiLineString<'a, T>,
    MonotoneChainGeometry<'a, T>
);
symmetric_intersects_impl!(MonotoneChainPolygon<'a, T>, MonotoneChainGeometry<'a, T>);
symmetric_intersects_impl!(
    MonotoneChainMultiPolygon<'a, T>,
    MonotoneChainGeometry<'a, T>
);
