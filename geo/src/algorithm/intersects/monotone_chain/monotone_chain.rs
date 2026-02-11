use crate::geometry::*;
use crate::intersects::has_disjoint_bboxes;
use crate::monotone_chain::{MonotoneChain, MonotoneChainSegment};
use crate::{BoundingRect, GeoNum, Intersects};

impl<'a, T> Intersects<Coord<T>> for MonotoneChain<'a, T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        self.segment_iter().any(|seg| seg.intersects(rhs))
    }
}
symmetric_intersects_impl!(Coord<T>, MonotoneChain<'a, T>);

impl<'a, T> Intersects<Line<T>> for MonotoneChain<'a, T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Line<T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        self.segment_iter().any(|seg| seg.intersects(rhs))
    }
}
symmetric_intersects_impl!(Line<T>, MonotoneChain<'a, T>);

impl<'a, T> Intersects<MonotoneChain<'a, T>> for MonotoneChain<'a, T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &MonotoneChain<'a, T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        self.segment_iter().any(|seg| seg.intersects(rhs))
    }
}

impl<'a, T> Intersects<MonotoneChainSegment<'a, T>> for MonotoneChain<'a, T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &MonotoneChainSegment<'a, T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        self.segment_iter().any(|seg| seg.intersects(rhs))
    }
}

symmetric_intersects_impl!(MonotoneChainSegment<'a, T>, MonotoneChain<'a, T>);
