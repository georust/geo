use super::{has_disjoint_bboxes, Intersects};
use crate::*;

impl<T> Intersects<Coord<T>> for Rect<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        rhs.x >= self.min().x
            && rhs.y >= self.min().y
            && rhs.x <= self.max().x
            && rhs.y <= self.max().y
    }
}

symmetric_intersects_impl!(Rect<T>, LineString<T>);
symmetric_intersects_impl!(Rect<T>, MultiLineString<T>);

// Same logic as Polygon<T>: Intersects<Line<T>>, but avoid
// an allocation.
impl<T> Intersects<Line<T>> for Rect<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Line<T>) -> bool {
        let lb = self.min();
        let rt = self.max();
        let lt = Coord::from((lb.x, rt.y));
        let rb = Coord::from((rt.x, lb.y));
        // If either rhs.{start,end} lies inside Rect, then true
        self.intersects(&rhs.start)
            || self.intersects(&rhs.end)
            || Line::new(lt, rt).intersects(rhs)
            || Line::new(rt, rb).intersects(rhs)
            || Line::new(lb, rb).intersects(rhs)
            || Line::new(lt, lb).intersects(rhs)
    }
}

symmetric_intersects_impl!(Rect<T>, Point<T>);
symmetric_intersects_impl!(Rect<T>, MultiPoint<T>);

symmetric_intersects_impl!(Rect<T>, Polygon<T>);
symmetric_intersects_impl!(Rect<T>, MultiPolygon<T>);

impl<T> Intersects<Rect<T>> for Rect<T>
where
    T: CoordNum,
{
    fn intersects(&self, other: &Rect<T>) -> bool {
        if self.max().x < other.min().x {
            return false;
        }

        if self.max().y < other.min().y {
            return false;
        }

        if self.min().x > other.max().x {
            return false;
        }

        if self.min().y > other.max().y {
            return false;
        }

        true
    }
}

impl<T> Intersects<Triangle<T>> for Rect<T>
where
    T: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T>) -> bool {
        // sufficient to show that any of these are true:
        // some corner of the triangle intersects the rectangle
        // some corner of the rectangle intersects the triangle
        // some edge of triangle intersects edge of rectangle

        if has_disjoint_bboxes(self, rhs) {
            return false;
        }

        rhs.coords_iter().any(|p| self.intersects(&p))
            || self.coords_iter().any(|p| rhs.intersects(&p))
            || rhs.lines_iter().any(|l| self.intersects(&l))
    }
}
