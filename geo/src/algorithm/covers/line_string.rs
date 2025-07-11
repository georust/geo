use super::{impl_covers_from_relate, Covers};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

impl<T> Covers<Coord<T>> for LineString<T>
where
    T: GeoFloat,
    Self: Covers<Point<T>>,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.covers(&Point::new(rhs.x, rhs.y))
    }
}
impl_covers_from_relate!(LineString<T>, [Point<T>, MultiPoint<T>]);
impl_covers_from_relate!(LineString<T>, [Line<T>]);
impl_covers_from_relate!(LineString<T>, [ LineString<T>,  MultiLineString<T>]);
impl_covers_from_relate!(LineString<T>, [ Rect<T>, Triangle<T>]);
impl_covers_from_relate!(LineString<T>, [Polygon<T>,  MultiPolygon<T>]);
impl_covers_from_relate!(LineString<T>, [Geometry<T>, GeometryCollection<T>]);

impl<T> Covers<Coord<T>> for MultiLineString<T>
where
    T: GeoFloat,
    Self: Covers<Point<T>>,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.covers(&Point::new(rhs.x, rhs.y))
    }
}
impl_covers_from_relate!(MultiLineString<T>, [Point<T>, MultiPoint<T>]);
impl_covers_from_relate!(MultiLineString<T>, [Line<T>]);
impl_covers_from_relate!(MultiLineString<T>, [ LineString<T>,  MultiLineString<T>]);
impl_covers_from_relate!(MultiLineString<T>, [ Rect<T>, Triangle<T>]);
impl_covers_from_relate!(MultiLineString<T>, [Polygon<T>,  MultiPolygon<T>]);
impl_covers_from_relate!(MultiLineString<T>, [Geometry<T>, GeometryCollection<T>]);
