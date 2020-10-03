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
rhs_pt_from_coord_intersects_impl!(Coordinate<T>);
