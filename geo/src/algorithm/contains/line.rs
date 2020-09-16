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

        // A closed linestring contains the start point in
        // its interior and hence, must be within the line
        // too.
        if linestring.is_closed() && !self.contains(&linestring.0[0]) {
            return false;
        }

        linestring.0.iter().all(|c| self.intersects(c))
    }
}
