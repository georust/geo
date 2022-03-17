use super::Contains;
use crate::{Coordinate, GeoNum, LineString, Point, Triangle};

// ┌──────────────────────────────┐
// │ Implementations for Triangle │
// └──────────────────────────────┘

impl<T> Contains<Coordinate<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        let ls = LineString(vec![self.0, self.1, self.2, self.0]);
        use crate::utils::{coord_pos_relative_to_ring, CoordPos};
        coord_pos_relative_to_ring(*coord, &ls) == CoordPos::Inside
    }
}

impl<T> Contains<Point<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}
