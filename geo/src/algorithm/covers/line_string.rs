use super::{Covers, impl_covers_from_intersects, impl_covers_from_relate};
use crate::{Contains, HasDimensions, geometry::*};
use crate::{GeoFloat, GeoNum};

impl_covers_from_intersects!(LineString<T>, [Point<T>, MultiPoint<T>]);

impl<T> Covers<Line<T>> for LineString<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &Line<T>) -> bool {
        if rhs.start == rhs.end {
            self.covers(&rhs.start)
        } else {
            self.contains(rhs)
        }
    }
}

impl<T> Covers<LineString<T>> for LineString<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &LineString<T>) -> bool {
        if self.is_empty() || rhs.is_empty() {
            return false;
        }
        rhs.lines().all(|l| self.covers(&l))
    }
}

impl_covers_from_relate!(LineString<T>, [
MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

impl_covers_from_intersects!(MultiLineString<T>, [Point<T>, MultiPoint<T>]);
impl_covers_from_relate!(MultiLineString<T>, [
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);
