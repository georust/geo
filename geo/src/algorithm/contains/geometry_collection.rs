use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

impl<T> Contains<Coord<T>> for GeometryCollection<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        self.iter().any(|geometry| geometry.contains(coord))
    }
}

impl<T> Contains<Point<T>> for GeometryCollection<T>
where
    T: GeoNum,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}

impl_contains_from_relate!(GeometryCollection<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(GeometryCollection<T>);
