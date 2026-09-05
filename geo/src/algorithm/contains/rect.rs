use geo_types::CoordFloat;

use super::{Contains, impl_contains_from_relate, impl_contains_geometry_for};
use crate::{Area, CoordsIter, HasDimensions, Intersects, geometry::*};
use crate::{CoordNum, GeoFloat};

// ┌──────────────────────────┐
// │ Implementations for Rect │
// └──────────────────────────┘

impl<T> Contains<Coord<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        geo_types::private_utils::rect_contains_coord(self, coord)
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

// The interior of a rect is the product of its interior on each axis: the open interval
// where the extent is non-zero, and the single value where it is zero. Two rects contain
// one another only if these interiors meet on both axes, which a comparison of the
// bounds alone does not show.
fn axis_interiors_intersect<T: CoordNum>(a_min: T, a_max: T, b_min: T, b_max: T) -> bool {
    match (a_min < a_max, b_min < b_max) {
        (true, true) => {
            let lower = if a_min > b_min { a_min } else { b_min };
            let upper = if a_max < b_max { a_max } else { b_max };
            lower < upper
        }
        (true, false) => a_min < b_min && b_min < a_max,
        (false, true) => b_min < a_min && a_min < b_max,
        (false, false) => a_min == b_min,
    }
}

impl<T> Contains<Rect<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, other: &Rect<T>) -> bool {
        // `other` must not reach outside `self`, and the two interiors must meet. A rect
        // with no width or height is a line or a point, and one that lies along the
        // boundary of `self` is covered but not contained.
        self.min().x <= other.min().x
            && self.max().x >= other.max().x
            && self.min().y <= other.min().y
            && self.max().y >= other.max().y
            && axis_interiors_intersect(self.min().x, self.max().x, other.min().x, other.max().x)
            && axis_interiors_intersect(self.min().y, self.max().y, other.min().y, other.max().y)
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
