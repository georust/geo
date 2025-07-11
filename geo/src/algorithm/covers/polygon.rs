use super::{impl_covers_from_relate, Covers};
use crate::{geometry::*, Intersects};
use crate::{GeoFloat, GeoNum};

impl<T> Covers<Coord<T>> for Polygon<T>
where
    T: GeoFloat,
    Self: Intersects<Coord<T>>,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.intersects(rhs)
    }
}

impl<T> Covers<Point<T>> for Polygon<T>
where
    T: GeoFloat,
    Self: Intersects<Point<T>>,
{
    fn covers(&self, rhs: &Point<T>) -> bool {
        self.intersects(rhs)
    }
}

impl_covers_from_relate!(Polygon<T>, [ MultiPoint<T>]);
impl_covers_from_relate!(Polygon<T>, [Line<T>]);
impl_covers_from_relate!(Polygon<T>, [ LineString<T>,  MultiLineString<T>]);
impl_covers_from_relate!(Polygon<T>, [ Rect<T>, Triangle<T>]);
impl_covers_from_relate!(Polygon<T>, [Polygon<T>,  MultiPolygon<T>]);
impl_covers_from_relate!(Polygon<T>, [Geometry<T>, GeometryCollection<T>]);

impl<T> Covers<Coord<T>> for MultiPolygon<T>
where
    T: GeoFloat,
    Self: Covers<Point<T>>,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.covers(&Point::new(rhs.x, rhs.y))
    }
}
impl_covers_from_relate!(MultiPolygon<T>, [Point<T>, MultiPoint<T>]);
impl_covers_from_relate!(MultiPolygon<T>, [Line<T>]);
impl_covers_from_relate!(MultiPolygon<T>, [ LineString<T>,  MultiLineString<T>]);
impl_covers_from_relate!(MultiPolygon<T>, [ Rect<T>, Triangle<T>]);
impl_covers_from_relate!(MultiPolygon<T>, [Polygon<T>,  MultiPolygon<T>]);
impl_covers_from_relate!(MultiPolygon<T>, [Geometry<T>, GeometryCollection<T>]);
