use super::Intersects;
use crate::*;

impl<T, G> Intersects<G> for Triangle<T>
where
    T: CoordNum,
    Polygon<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        (*self).to_polygon().intersects(rhs)
    }
}
symmetric_intersects_impl!(Coordinate<T>, Triangle<T>);
symmetric_intersects_impl!(Line<T>, Triangle<T>);
symmetric_intersects_impl!(Rect<T>, Triangle<T>);
symmetric_intersects_impl!(Polygon<T>, Triangle<T>);
