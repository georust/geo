use super::Contains;
use crate::kernels::*;
use crate::*;

// ┌──────────────────────────────┐
// │ Implementations for Triangle │
// └──────────────────────────────┘

impl<T> Contains<Coordinate<T>> for Triangle<T>
where
    T: HasKernel,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        let ls = LineString(vec![self.0, self.1, self.2, self.0]);
        use utils::*;
        coord_pos_relative_to_ring(*coord, &ls) == CoordPos::Inside
    }
}

impl<T> Contains<Point<T>> for Triangle<T>
where
    T: HasKernel,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}
