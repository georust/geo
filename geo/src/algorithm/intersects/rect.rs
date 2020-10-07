use super::{value_in_range, Intersects};
use crate::kernels::*;
use crate::*;

impl<T> Intersects<Coordinate<T>> for Rect<T>
where
    T: CoordinateType,
{
    fn intersects(&self, rhs: &Coordinate<T>) -> bool {
        // Funnily, we don't use point_in_rect, as we know
        // self.min <= self.max.
        let bound_1 = self.min();
        let bound_2 = self.max();
        value_in_range(rhs.x, bound_1.x, bound_2.x) && value_in_range(rhs.y, bound_1.y, bound_2.y)
    }
}
symmetric_intersects_impl!(Coordinate<T>, Rect<T>);
symmetric_intersects_impl!(Rect<T>, Point<T>);
symmetric_intersects_impl!(Rect<T>, MultiPoint<T>);

impl<T> Intersects<Rect<T>> for Rect<T>
where
    T: CoordinateType,
{
    fn intersects(&self, other: &Rect<T>) -> bool {
        let x_overlap = value_in_range(self.min().x, other.min().x, other.max().x)
            || value_in_range(other.min().x, self.min().x, self.max().x);

        let y_overlap = value_in_range(self.min().y, other.min().y, other.max().y)
            || value_in_range(other.min().y, self.min().y, self.max().y);

        x_overlap && y_overlap
    }
}

// Same logic as Polygon<T>: Intersects<Line<T>>, but avoid
// an allocation.
impl<T> Intersects<Line<T>> for Rect<T>
where
    T: HasKernel,
{
    fn intersects(&self, rhs: &Line<T>) -> bool {
        let lt = self.min();
        let rb = self.max();
        let lb = Coordinate::from((lt.x, rb.y));
        let rt = Coordinate::from((rb.x, lt.y));
        // If either rhs.{start,end} lies inside Rect, then true
        self.intersects(&rhs.start)
            || self.intersects(&rhs.end)
            || Line::new(lt, rt).intersects(rhs)
            || Line::new(rt, rb).intersects(rhs)
            || Line::new(lb, rb).intersects(rhs)
            || Line::new(lt, lb).intersects(rhs)
    }
}
symmetric_intersects_impl!(Line<T>, Rect<T>);
