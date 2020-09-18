use super::Intersects;
use crate::kernels::*;
use crate::*;

impl<T, G> Intersects<G> for Triangle<T>
where
    T: CoordinateType,
    Polygon<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.clone().to_polygon().intersects(rhs)
    }
}
symmetric_intersects_impl!(Coordinate<T>, Triangle<T>, HasKernel);
symmetric_intersects_impl!(Point<T>, Triangle<T>, HasKernel);
symmetric_intersects_impl!(Line<T>, Triangle<T>, HasKernel);
symmetric_intersects_impl!(LineString<T>, Triangle<T>, HasKernel);
symmetric_intersects_impl!(Polygon<T>, Triangle<T>, HasKernel);
