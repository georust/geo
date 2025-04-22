use super::Intersects;
use crate::*;

impl<T> Intersects<Coord<T>> for Coord<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Coord<T>) -> bool {
        self == rhs
    }
}

// The other side of this is handled via a blanket impl.
impl<T> Intersects<Point<T>> for Coord<T>
where
    T: CoordNum,
{
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self == &rhs.0
    }
}
