use super::Contains;
use crate::*;
use crate::kernels::*;
use crate::intersects::Intersects;

// ┌──────────────────────────┐
// │ Implementations for Line │
// └──────────────────────────┘

impl<T> Contains<Coordinate<T>> for Line<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        self.intersects(coord)
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
        self.contains(&line.start) && self.contains(&line.end)
    }
}

impl<T> Contains<LineString<T>> for Line<T>
where
    T: HasKernel,
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        linestring.0.iter().all(|c| self.contains(c))
    }
}
