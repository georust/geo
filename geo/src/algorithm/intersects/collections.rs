use super::Intersects;
use crate::*;

impl<T, G> Intersects<G> for Geometry<T>
where
    T: CoordinateType,
    Point<T>: Intersects<G>,
    MultiPoint<T>: Intersects<G>,
    Line<T>: Intersects<G>,
    LineString<T>: Intersects<G>,
    MultiLineString<T>: Intersects<G>,
    Triangle<T>: Intersects<G>,
    Rect<T>: Intersects<G>,
    Polygon<T>: Intersects<G>,
    MultiPolygon<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        match self {
            Geometry::Point(geom) => geom.intersects(rhs),
            Geometry::MultiPoint(geom) => geom.intersects(rhs),
            Geometry::Line(geom) => geom.intersects(rhs),
            Geometry::LineString(geom) => geom.intersects(rhs),
            Geometry::MultiLineString(geom) => geom.intersects(rhs),
            Geometry::Triangle(geom) => geom.intersects(rhs),
            Geometry::Rect(geom) => geom.intersects(rhs),
            Geometry::Polygon(geom) => geom.intersects(rhs),
            Geometry::MultiPolygon(geom) => geom.intersects(rhs),
            Geometry::GeometryCollection(geom) => geom.intersects(rhs),
        }
    }
}
symmetric_intersects_impl!(Coordinate<T>, Geometry<T>);
symmetric_intersects_impl!(Line<T>, Geometry<T>);
symmetric_intersects_impl!(Rect<T>, Geometry<T>);
symmetric_intersects_impl!(Polygon<T>, Geometry<T>);

impl<T, G> Intersects<G> for GeometryCollection<T>
where
    T: CoordinateType,
    Geometry<T>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.iter().any(|geom| geom.intersects(rhs))
    }
}
symmetric_intersects_impl!(Coordinate<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Line<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Rect<T>, GeometryCollection<T>);
symmetric_intersects_impl!(Polygon<T>, GeometryCollection<T>);
