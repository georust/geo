use super::{impl_covers_convex_poly, Covers};
use crate::{geometry::*, CoordsIter, Intersects};
use crate::{GeoNum};


impl<T> Covers<Coord<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.intersects(rhs)
    }
}

impl<T> Covers<Point<T>> for Triangle<T>
where
    T: GeoNum,

{
    fn covers(&self, rhs: &Point<T>) -> bool {
        self.intersects(rhs)
    }
}

impl_covers_convex_poly!(Triangle<T>, [MultiPoint<T>]);
impl_covers_convex_poly!(Triangle<T>, [Line<T>, LineString<T>, MultiLineString<T>]);
impl_covers_convex_poly!(Triangle<T>, [ Rect<T>, Triangle<T>]);
impl_covers_convex_poly!(Triangle<T>, [Polygon<T>,  MultiPolygon<T>]);
impl_covers_convex_poly!(Triangle<T>, [Geometry<T>, GeometryCollection<T>]);
