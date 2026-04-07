use crate::geometry::*;
use crate::intersects::has_disjoint_bboxes;
use crate::monotone_chain::MonotoneChainSegment;
use crate::{BoundingRect, GeoNum, Intersects};

macro_rules! intersects_MonotoneChainSegment {
    ( $k:ty) => {
        impl<'a: 'caller, 'caller, T> $crate::Intersects<$k> for MonotoneChainSegment<'a, T>
        where
            $k: BoundingRect<T>,
            LineString<T>: Intersects<$k>,
            Line<T>: Intersects<$k>,
            Coord<T>: Intersects<$k>,
            T: GeoNum,
        {
            fn intersects(&self, rhs: &$k) -> bool {
                // PERF: this recursivee function recalculates the
                //  bounding box of rhs on each loop
                if has_disjoint_bboxes(self, rhs) {
                    return false;
                }

                match self.ls().len() {
                    0 => false,
                    1 => self.ls()[0].intersects(rhs),
                    2 => {
                        // PERF: One potential speed optimization here is to use specialized function
                        // with pre-requisite of bounding box intersection.
                        // e.g. Line intersect Line could become just the orientation checks
                        Line::<T>::new(self.ls()[0], self.ls()[1]).intersects(rhs)
                    }
                    _ => {
                        if let (l, Some(r)) = self.divide() {
                            l.intersects(rhs) || r.intersects(rhs)
                        } else {
                            unreachable!("divide() should always return Some for n > 2");
                        }
                    }
                }
            }
        }
    };
}

intersects_MonotoneChainSegment!(Coord<T>);
symmetric_intersects_impl!(Coord<T>, MonotoneChainSegment<'a, T>);

intersects_MonotoneChainSegment!(Line<T>);
symmetric_intersects_impl!(Line<T>, MonotoneChainSegment<'a, T>);

impl<'a, T> Intersects<MonotoneChainSegment<'a, T>> for MonotoneChainSegment<'a, T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &MonotoneChainSegment<'a, T>) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        // handle degenerate cases
        if self.ls().len() == 1 {
            return self.ls()[0].intersects(rhs);
        }
        if rhs.ls().len() == 1 {
            return rhs.ls()[0].intersects(self);
        }

        // recurse with binary split on both sides
        match (self.divide(), rhs.divide()) {
            ((a, Some(b)), (c, Some(d))) => {
                a.intersects(&c) || a.intersects(&d) || b.intersects(&c) || b.intersects(&d)
            }
            ((a, Some(b)), (_c, None)) => a.intersects(rhs) || b.intersects(rhs),
            ((_a, None), (c, Some(d))) => self.intersects(&c) || self.intersects(&d),

            ((a, None), (c, None)) => Line::<T>::new(a.ls()[0], a.ls()[1])
                .intersects(&Line::<T>::new(c.ls()[0], c.ls()[1])),
        }
    }
}
