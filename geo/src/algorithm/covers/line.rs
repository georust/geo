use super::{impl_covers_from_relate, Covers};
use crate::{geometry::*, Intersects};
use crate::{GeoFloat, GeoNum};

impl<T> Covers<Coord<T>> for Line<T>
where
    T: GeoFloat,
    Self: Covers<Point<T>>,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.covers(&Point::new(rhs.x, rhs.y))
    }
}

impl<T> Covers<Point<T>> for Line<T>
where
    T: GeoFloat,
{
    fn covers(&self, rhs: &Point<T>) -> bool {
        self.intersects(rhs)
    }
}
impl_covers_from_relate!(Line<T>, [MultiPoint<T>]);
impl_covers_from_relate!(Line<T>, [Line<T>]);
impl_covers_from_relate!(Line<T>, [ LineString<T>,  MultiLineString<T>]);
impl_covers_from_relate!(Line<T>, [ Rect<T>, Triangle<T>]);
impl_covers_from_relate!(Line<T>, [Polygon<T>,  MultiPolygon<T>]);
impl_covers_from_relate!(Line<T>, [Geometry<T>, GeometryCollection<T>]);
