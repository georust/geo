use super::Intersects;
use crate::*;

// Blanket implementation using self.lines().any().
impl<T, G> Intersects<G> for LineString<T>
where
    T: CoordNum,
    Line<T>: Intersects<G>,
{
    fn intersects(&self, geom: &G) -> bool {
        self.lines().any(|l| l.intersects(geom))
    }
}
symmetric_intersects_impl!(Coordinate<T>, LineString<T>);
symmetric_intersects_impl!(Line<T>, LineString<T>);
symmetric_intersects_impl!(Rect<T>, LineString<T>);

// Blanket implementation from LineString<T>
impl<T, G> Intersects<G> for MultiLineString<T>
where
    T: CoordNum,
    LineString<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.iter().any(|p| p.intersects(rhs))
    }
}

symmetric_intersects_impl!(Point<T>, MultiLineString<T>);
symmetric_intersects_impl!(Line<T>, MultiLineString<T>);
symmetric_intersects_impl!(Rect<T>, MultiLineString<T>);
