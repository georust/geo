use super::{impl_covers_from_relate, impl_covers_geometry_for, Covers};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

impl<T> Covers<Coord<T>> for GeometryCollection<T>
where
    T: GeoFloat,
{
    fn covers(&self, rhs: &Coord<T>) -> bool {
        self.iter().any(|geometry| geometry.covers(rhs))
    }
}

impl<T> Covers<Point<T>> for GeometryCollection<T>
where
    T: GeoFloat,
{
    fn covers(&self, point: &Point<T>) -> bool {
        self.covers(&point.0)
    }
}

impl_covers_from_relate!(GeometryCollection<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_covers_geometry_for!(GeometryCollection<T>);
