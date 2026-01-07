use crate::geometry::*;
use crate::intersects::has_disjoint_bboxes;
use crate::monotone_chain::{MonotoneChain, MonotoneChainSegment};
use crate::{BoundingRect, GeoNum, Intersects};

//TODO: check if order of montone chains vs monotone chain segments affects performance

impl<'a, G, T: GeoNum> Intersects<G> for MonotoneChain<'a, T>
where
    G: BoundingRect<T>,
    MonotoneChainSegment<'a, T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        self.segment_iter().any(|seg| seg.intersects(rhs))
    }
}

symmetric_intersects_impl!(MonotoneChainSegment<'a, T>, MonotoneChain<'a, T>);

// commented out if they are implemented by blanket impl in main `Intersects` trait
symmetric_intersects_impl!(Coord<T>, MonotoneChain<'a, T>);
// symmetric_intersects_impl!(Point<T>,MonotoneChain<'a, T>);
// symmetric_intersects_impl!(MultiPoint<T>,MonotoneChain<'a, T>);

symmetric_intersects_impl!(Line<T>, MonotoneChain<'a, T>);
symmetric_intersects_impl!(LineString<T>, MonotoneChain<'a, T>);
// symmetric_intersects_impl!(MultiLineString<T>,MonotoneChain<'a, T>);

symmetric_intersects_impl!(Polygon<T>, MonotoneChain<'a, T>);
// symmetric_intersects_impl!(MultiPolygon<T>,MonotoneChain<'a, T>);
symmetric_intersects_impl!(Rect<T>, MonotoneChain<'a, T>);
symmetric_intersects_impl!(Triangle<T>, MonotoneChain<'a, T>);

// symmetric_intersects_impl!(Geometry<T>,MonotoneChain<'a, T>);
// symmetric_intersects_impl!(GeometryCollection<T>,MonotoneChain<'a, T>);
