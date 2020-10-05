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

// The other side of this is handled via a blanket impl.
impl<T> Intersects<Point<T>> for Coordinate<T>
where
    T: CoordinateType,
{
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self == &rhs.0
    }
}
