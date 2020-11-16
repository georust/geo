use super::Intersects;
use crate::*;

// Blanket implementation from Coordinate<T>
impl<T, G> Intersects<G> for Point<T>
where
    T: CoordinateType,
    Coordinate<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.0.intersects(rhs)
    }
}

// Blanket implementation from Point<T>
impl<T, G> Intersects<G> for MultiPoint<T>
where
    T: CoordinateType,
    Point<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.iter().any(|p| p.intersects(rhs))
    }
}

symmetric_intersects_impl!(Coordinate<T>, MultiPoint<T>);
symmetric_intersects_impl!(Line<T>, MultiPoint<T>);
symmetric_intersects_impl!(Polygon<T>, MultiPoint<T>);
