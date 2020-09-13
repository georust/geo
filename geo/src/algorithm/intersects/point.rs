use super::Intersects;
use crate::*;

impl<T> Intersects<Coordinate<T>> for Point<T>
where
    T: CoordinateType,
{
    fn intersects(&self, rhs: &Coordinate<T>) -> bool {
        &self.0 == rhs
    }
}

impl<T> Intersects<Point<T>> for Point<T>
where
    T: CoordinateType,
{
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self == rhs
    }
}
