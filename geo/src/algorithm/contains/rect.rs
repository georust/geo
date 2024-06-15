use geo_types::CoordFloat;

use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::{geometry::*, Area, CoordsIter, HasDimensions, Intersects};
use crate::{CoordNum, GeoFloat};

// ┌──────────────────────────┐
// │ Implementations for Rect │
// └──────────────────────────┘

impl<T> Contains<Coord<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        coord.x > self.min().x
            && coord.x < self.max().x
            && coord.y > self.min().y
            && coord.y < self.max().y
    }
}

impl<T> Contains<Point<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.contains(&p.0)
    }
}

impl<T> Contains<Rect<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, other: &Rect<T>) -> bool {
        // TODO: check for degenerate rectangle (which is a line or a point)
        // All points of LineString must be in the polygon ?
        self.min().x <= other.min().x
            && self.max().x >= other.max().x
            && self.min().y <= other.min().y
            && self.max().y >= other.max().y
    }
}

impl<T> Contains<Polygon<T>> for Rect<T>
where
    T: CoordFloat,
{
    fn contains(&self, rhs: &Polygon<T>) -> bool {
        // the polygon must not be empty
        if rhs.is_empty() {
            return false;
        }

        // none of the polygon's points may lie outside the rectangle
        let mut points_inside = 0;
        for c in rhs.exterior_coords_iter() {
            if !self.intersects(&c) {
                return false;
            }
            if self.contains(&c) {
                points_inside += 1;
            }
        }

        // The polygon must not lie completely inside the rectangle's boundary.
        // In other words: at least one point of the interior of the polygon
        // must lie in the interior of the rectangle. Since we know that the
        // rectangle is convex, we just need make sure that either at least
        // one point of the polygon lies inside the rectangle's interior or
        // that the polygon's interior is not empty, in which case it will
        // definitely intersect with the rectangle's interior.
        if points_inside == 0 && rhs.signed_area().is_zero() {
            return false;
        }

        true
    }
}

impl_contains_from_relate!(Rect<T>, [Line<T>, LineString<T>, MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T>, Triangle<T>]);
impl_contains_geometry_for!(Rect<T>);
