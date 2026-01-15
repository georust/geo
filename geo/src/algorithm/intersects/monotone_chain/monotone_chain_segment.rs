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
                const SEARCH_THRESHOLD: usize = 2;

                match self.ls().len() {
                    0 => false,
                    1 => self.ls()[0].intersects(rhs),
                    2 => {
                        // PERF: One potential speed optimization here is to use specialized function
                        // with pre-requisite of bounding box intersection.
                        // e.g. Line intersect Line could become just the orientation checks
                        Line::<T>::new(self.ls()[0], self.ls()[1]).intersects(rhs)
                    }
                    n if n > SEARCH_THRESHOLD => {
                        if let (l, Some(r)) = self.divide() {
                            l.intersects(rhs) || r.intersects(rhs)
                        } else {
                            unreachable!(
                                "divide() should always return Some for n > SEARCH_THRESHOLD"
                            );
                        }
                    }
                    _ => LineString::from_iter(self.ls().iter().cloned()).intersects(rhs),
                }
            }
        }
    };
}

intersects_MonotoneChainSegment!(Coord<T>);
intersects_MonotoneChainSegment!(Point<T>);
intersects_MonotoneChainSegment!(MultiPoint<T>);
intersects_MonotoneChainSegment!(Line<T>);
intersects_MonotoneChainSegment!(LineString<T>);
intersects_MonotoneChainSegment!(MultiLineString<T>);
intersects_MonotoneChainSegment!(Polygon<T>);
intersects_MonotoneChainSegment!(MultiPolygon<T>);
intersects_MonotoneChainSegment!(Rect<T>);
intersects_MonotoneChainSegment!(Triangle<T>);
intersects_MonotoneChainSegment!(Geometry<T>);
intersects_MonotoneChainSegment!(GeometryCollection<T>);

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
        // handle base case
        if self.ls().len() == 2 && rhs.ls().len() == 2 {
            //PERF: optimize this to skip construction and bounding box check
            return Line::<T>::new(self.ls()[0], self.ls()[1])
                .intersects(&Line::<T>::new(rhs.ls()[0], rhs.ls()[1]));
        }

        // recurse with binary split on both sides
        match (self.divide(), rhs.divide()) {
            ((a, Some(b)), (c, Some(d))) => {
                a.intersects(&c) || a.intersects(&d) || b.intersects(&c) || b.intersects(&d)
            }
            ((a, Some(b)), (_c, None)) => a.intersects(rhs) || b.intersects(rhs),
            ((_a, None), (c, Some(d))) => self.intersects(&c) || self.intersects(&d),
            ((_a, None), (_c, None)) => unreachable!("both sides are single segments"),
        }
    }
}

// commented out if they are implemented by blanket impl in main `Intersects` trait
symmetric_intersects_impl!(Coord<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(Point<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(MultiPoint<T>, MonotoneChainSegment<'a, T>);

symmetric_intersects_impl!(Line<T>, MonotoneChainSegment<'a, T>);
symmetric_intersects_impl!(LineString<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(MultiLineString<T>, MonotoneChainSegment<'a, T>);

symmetric_intersects_impl!(Polygon<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(MultiPolygon<T>, MonotoneChainSegment<'a, T>);
symmetric_intersects_impl!(Rect<T>, MonotoneChainSegment<'a, T>);
symmetric_intersects_impl!(Triangle<T>, MonotoneChainSegment<'a, T>);

// symmetric_intersects_impl!(Geometry<T>, MonotoneChainSegment<'a, T>);
// symmetric_intersects_impl!(GeometryCollection<T>, MonotoneChainSegment<'a, T>);
