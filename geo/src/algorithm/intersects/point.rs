use super::Intersects;
use crate::kernels::*;
use crate::*;

impl<T> Intersects<Coordinate<T>> for Point<T>
where
    T: CoordinateType,
{
    fn intersects(&self, rhs: &Coordinate<T>) -> bool {
        self.0.intersects(rhs)
    }
}
symmetric_intersects_impl!(Coordinate<T>, Point<T>, CoordinateType);

impl<T> Intersects<Point<T>> for Point<T>
where
    T: CoordinateType,
{
    fn intersects(&self, rhs: &Point<T>) -> bool {
        self.intersects(&rhs.0)
    }
}

impl<T, G> Intersects<G> for MultiPoint<T>
where
    T: CoordinateType,
    Point<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.0.iter().any(|p| p.intersects(rhs))
    }
}
symmetric_intersects_impl!(Coordinate<T>, MultiPoint<T>, CoordinateType);
symmetric_intersects_impl!(Point<T>, MultiPoint<T>, CoordinateType);
symmetric_intersects_impl!(Line<T>, MultiPoint<T>, HasKernel);
symmetric_intersects_impl!(LineString<T>, MultiPoint<T>, HasKernel);
symmetric_intersects_impl!(Polygon<T>, MultiPoint<T>, HasKernel);
