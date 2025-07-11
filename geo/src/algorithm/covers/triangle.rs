use super::{Covers, impl_covers_from_intersects};
use crate::GeoNum;
use crate::{CoordsIter, Intersects, geometry::*};

impl<T> Covers<Polygon<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &Polygon<T>) -> bool {
        rhs.exterior_coords_iter().all(|c| self.intersects(&c))
    }
}

impl<T> Covers<MultiPolygon<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn covers(&self, rhs: &MultiPolygon<T>) -> bool {
        rhs.exterior_coords_iter().all(|c| self.intersects(&c))
    }
}

impl_covers_from_intersects!(Triangle<T>, [
Point<T>,MultiPoint<T>,
Line<T>,
LineString<T>, MultiLineString<T>,
Rect<T>, Triangle<T>,
Geometry<T>, GeometryCollection<T>
]);
