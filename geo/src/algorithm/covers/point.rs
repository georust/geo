use super::{Covers, impl_covers_from_intersects, impl_covers_from_relate};
use crate::{Contains, geometry::*};
use crate::{GeoFloat, GeoNum};

impl_covers_from_intersects!(Point<T>, [
Point<T>, MultiPoint<T>,
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);

impl_covers_from_intersects!(MultiPoint<T>, [Point<T>]);

impl<T> Covers<MultiPoint<T>> for MultiPoint<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &MultiPoint<T>) -> bool {
        // use the sliding comparison implementation
        self.contains(rhs)
    }
}

impl_covers_from_relate!(MultiPoint<T>, [
Line<T>,
LineString<T>,  MultiLineString<T>,
Rect<T>, Triangle<T>,
Polygon<T>,  MultiPolygon<T>,
Geometry<T>, GeometryCollection<T>
]);
