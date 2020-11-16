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
impl<G, T> Contains<G> for MultiPoint<T>
where
    T: CoordinateType,
    Point<T>: Contains<G>,
{
    fn contains(&self, rhs: &G) -> bool {
        self.iter().any(|p| p.contains(rhs))
    }
}
