use super::{impl_covers_convex_poly, Covers};
use crate::{geometry::*, CoordsIter, Intersects};
use crate::{GeoNum};


impl<T> Covers<Coord<T>> for Rect<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.intersects(rhs)
    }
}

impl<T> Covers<Point<T>> for Rect<T>
where
    T: GeoNum,

{
    fn covers(&self, rhs: &Point<T>) -> bool {
        self.intersects(rhs)
    }
}

impl_covers_convex_poly!(Rect<T>, [MultiPoint<T>]);
impl_covers_convex_poly!(Rect<T>, [Line<T>, LineString<T>, MultiLineString<T>]);
impl_covers_convex_poly!(Rect<T>, [ Rect<T>, Triangle<T>]);
impl_covers_convex_poly!(Rect<T>, [Polygon<T>,  MultiPolygon<T>]);
impl_covers_convex_poly!(Rect<T>, [Geometry<T>, GeometryCollection<T>]);
