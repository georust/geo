use super::Intersects;
use crate::kernels::*;
use crate::*;

impl<T> Intersects<Coordinate<T>> for LineString<T>
where
    T: HasKernel,
{
    fn intersects(&self, coord: &Coordinate<T>) -> bool {
        self.lines().any(|l| coord.intersects(&l))
    }
}
symmetric_intersects_impl!(Coordinate<T>, LineString<T>, HasKernel);

impl<T> Intersects<Point<T>> for LineString<T>
where
    T: HasKernel,
{
    fn intersects(&self, point: &Point<T>) -> bool {
        self.intersects(&point.0)
    }
}
symmetric_intersects_impl!(Point<T>, LineString<T>, HasKernel);

impl<T> Intersects<Line<T>> for LineString<T>
where
    T: HasKernel,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        self.lines().any(|l| line.intersects(&l))
    }
}
symmetric_intersects_impl!(Line<T>, LineString<T>, HasKernel);

impl<T> Intersects<LineString<T>> for LineString<T>
where
    T: HasKernel,
{
    fn intersects(&self, linestring: &LineString<T>) -> bool {
        if self.0.is_empty() || linestring.0.is_empty() {
            return false;
        }
        for a in self.lines() {
            for b in linestring.lines() {
                if a.intersects(&b) {
                    return true;
                }
            }
        }

        false
    }
}
