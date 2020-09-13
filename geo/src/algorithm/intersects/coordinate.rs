use super::Intersects;
use crate::*;

impl<T> Intersects<Coordinate<T>> for Coordinate<T>
where
    T: CoordinateType,
{
    fn intersects(&self, rhs: &Coordinate<T>) -> bool {
        self == rhs
    }
}
