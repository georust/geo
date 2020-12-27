use super::Contains;
use crate::intersects::Intersects;
use crate::kernels::*;
use crate::*;

// ┌──────────────────────────┐
// │ Implementations for Line │
// └──────────────────────────┘

impl<T> Contains<Coordinate<T>> for Line<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        if self.start == self.end {
            &self.start == coord
        } else {
            coord != &self.start && coord != &self.end && self.intersects(coord)
        }
    }
}

impl<T> Contains<Point<T>> for Line<T>
where
    T: HasKernel,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Line<T>> for Line<T>
where
    T: HasKernel,
{
    fn contains(&self, line: &Line<T>) -> bool {
        if line.start == line.end {
            self.contains(&line.start)
        } else {
            self.intersects(&line.start) && self.intersects(&line.end)
        }
    }
}

impl<T> Contains<LineString<T>> for Line<T>
where
    T: HasKernel,
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        // Empty linestring has no interior, and not
        // contained in anything.
        if linestring.0.is_empty() {
            return false;
        }

        // The interior of the linestring should have some
        // intersection with the interior of self. Two cases
        // arise:
        //
        // 1. There are at least two distinct points in the
        // linestring. Then, if both intersect, the interior
        // between these two must have non-empty intersection.
        //
        // 2. Otherwise, all the points on the linestring
        // are the same. In this case, the interior is this
        // specific point, and it should be contained in the
        // line.
        let first = linestring.0.first().unwrap();
        let mut all_equal = true;

        // If all the vertices of the linestring intersect
        // self, then the interior or boundary of the
        // linestring cannot have non-empty intersection
        // with the exterior.
        let all_intersects = linestring.0.iter().all(|c| {
            if c != first {
                all_equal = false;
            }
            self.intersects(c)
        });

        all_intersects && (!all_equal || self.contains(first))
    }
}
