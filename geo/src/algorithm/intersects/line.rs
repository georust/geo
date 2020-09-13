use super::{point_in_rect, Intersects};
use crate::kernels::*;
use crate::*;

impl<T> Intersects<Coordinate<T>> for Line<T>
where
    T: HasKernel,
{
    fn intersects(&self, rhs: &Coordinate<T>) -> bool {
        T::Ker::orient2d(self.start, self.end, *rhs) == Orientation::Collinear
            && point_in_rect(*rhs, self.start, self.end)
    }
}

impl<T> Intersects<Point<T>> for Line<T>
where
    T: HasKernel,
{
    fn intersects(&self, p: &Point<T>) -> bool {
        self.intersects(&p.0)
    }
}

impl<T> Intersects<Line<T>> for Line<T>
where
    T: HasKernel,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        if self.start == self.end {
            return line.intersects(&self.start);
        }
        let check_1_1 = T::Ker::orient2d(self.start, self.end, line.start);
        let check_1_2 = T::Ker::orient2d(self.start, self.end, line.end);

        if check_1_1 != check_1_2 {
            let check_2_1 = T::Ker::orient2d(line.start, line.end, self.start);
            let check_2_2 = T::Ker::orient2d(line.start, line.end, self.end);

            check_2_1 != check_2_2
        } else if check_1_1 == Orientation::Collinear {
            // Special case: collinear line segments.

            // Equivalent to the point-line intersection
            // impl., but removes the calls to the kernel
            // predicates.
            point_in_rect(line.start, self.start, self.end)
                || point_in_rect(line.end, self.start, self.end)
                || point_in_rect(self.end, line.start, line.end)
                || point_in_rect(self.end, line.start, line.end)
        } else {
            false
        }
    }
}
