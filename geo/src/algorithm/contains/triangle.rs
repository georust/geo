use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::geometry::*;
use crate::{GeoFloat, GeoNum};

// ┌──────────────────────────────┐
// │ Implementations for Triangle │
// └──────────────────────────────┘

impl<T> Contains<Coord<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        let ls = LineString::new(vec![self.0, self.1, self.2, self.0]);
        use crate::utils::{coord_pos_relative_to_ring, CoordPos};
        coord_pos_relative_to_ring(*coord, &ls) == CoordPos::Inside
    }
}

impl<T> Contains<Point<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn contains(&self, point: &Point<T>) -> bool {
        self.contains(&point.0)
    }
}

impl_contains_from_relate!(Triangle<T>, [Line<T>, LineString<T>, Polygon<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Rect<T>, Triangle<T>]);
impl_contains_geometry_for!(Triangle<T>);
