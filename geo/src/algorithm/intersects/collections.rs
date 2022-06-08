use super::{has_disjoint_bboxes, Intersects};
use crate::BoundingRect;
use crate::*;

impl<T, G> Intersects<G> for Geometry<T>
where
    T: CoordNum,
    Point<T>: Intersects<G>,
    MultiPoint<T>: Intersects<G>,
    Line<T>: Intersects<G>,
    LineString<T>: Intersects<G>,
    MultiLineString<T>: Intersects<G>,
    Triangle<T>: Intersects<G>,
    Rect<T>: Intersects<G>,
    Polygon<T>: Intersects<G>,
    MultiPolygon<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    geometry_delegate_impl! {
        fn intersects(&self, rhs: &G) -> bool;
    }
}
symmetric_intersects_impl!(Coordinate<T>, Geometry<T>);
symmetric_intersects_impl!(Line<T>, Geometry<T>);
symmetric_intersects_impl!(Rect<T>, Geometry<T>);
symmetric_intersects_impl!(Polygon<T>, Geometry<T>);

impl<T, G> Intersects<G> for GeometryCollection<T>
where
    T: CoordNum,
    Geometry<T>: Intersects<G>,
    G: BoundingRect<T>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        self.iter().any(|geom| geom.intersects(rhs))
    }
}
symmetric_intersects_impl!(Coordinate<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Line<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Rect<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Polygon<T>, GeometryCollection<T>);
