use super::Contains;
use crate::*;

// ┌──────────────────────────┐
// │ Implementations for Rect │
// └──────────────────────────┘

impl<T> Contains<Coordinate<T>> for Rect<T>
where
    T: CoordinateType,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        coord.x > self.min().x
            && coord.x < self.max().x
            && coord.y > self.min().y
            && coord.y < self.max().y
    }
}

impl<T> Contains<Point<T>> for Rect<T>
where
    T: CoordinateType,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Rect<T>> for Rect<T>
where
    T: CoordinateType,
{
    fn contains(&self, other: &Rect<T>) -> bool {
        // TODO: check for degenerate rectangle (which is a line or a point)
        // All points of LineString must be in the polygon ?
        self.min().x <= other.min().x
            && self.max().x >= other.max().x
            && self.min().y <= other.min().y
            && self.max().y >= other.max().y
    }
}
