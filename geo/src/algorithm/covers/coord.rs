use super::{Covers, impl_covers_from_intersects};
use crate::GeoNum;
use crate::Intersects;
use crate::geometry::*;

impl<T, G> Covers<Coord<T>> for G
where
    T: GeoNum,
    G: Intersects<Coord<T>>,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.intersects(rhs)
    }
}

impl_covers_from_intersects!(Coord<T>, [
Point<T>, MultiPoint<T>,
Line<T>,
LineString<T>, MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);
