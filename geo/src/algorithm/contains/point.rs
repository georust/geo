use super::Contains;
use crate::*;

// ┌────────────────────────────────┐
// │ Implementations for Point      │
// └────────────────────────────────┘

impl<T> Contains<Coordinate<T>> for Point<T>
where
    T: CoordinateType,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        &self.0 == coord
    }
}

impl<T> Contains<Point<T>> for Point<T>
where
    T: CoordinateType,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

// ┌────────────────────────────────┐
// │ Implementations for MultiPoint │
// └────────────────────────────────┘

impl<T> Contains<Coordinate<T>> for MultiPoint<T>
where
    T: CoordinateType,
{
    fn contains(&self, coord: &Coordinate<T>) -> bool {
        self.0.iter().any(|point| point.contains(coord))
    }
}

impl<T> Contains<Point<T>> for MultiPoint<T>
where
    T: CoordinateType,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}
