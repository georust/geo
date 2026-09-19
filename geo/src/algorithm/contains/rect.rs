use geo_types::CoordFloat;

use super::{Contains, impl_contains_from_relate, impl_contains_geometry_for};
use crate::dimensions::Dimensions;
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

impl<T> Contains<Rect<T>> for Rect<T>
where
    T: CoordNum,
{
    fn contains(&self, other: &Rect<T>) -> bool {
        use Dimensions::*;

        // We need to handle dimensions seperately to properly conform to DE-9IM
        match (self.dimensions(), other.dimensions()) {
            (TwoDimensional, TwoDimensional) => {
                self.min().x <= other.min().x
                    && other.max().x <= self.max().x
                    && self.min().y <= other.min().y
                    && other.max().y <= self.max().y
            }

            (TwoDimensional, OneDimensional) => {
                let inside_bounds = self.min().x <= other.min().x
                    && other.max().x <= self.max().x
                    && self.min().y <= other.min().y
                    && other.max().y <= self.max().y;

                // If the line is not contained at all, early return.
                if !inside_bounds {
                    return false;
                }

                if other.min().x == other.max().x {
                    // Vertical line (| represent the line, ==== for rect)
                    //
                    // DE-9IM does not accept this as contained (as it is on the boundary)
                    // │═════════════╗
                    // │             ║
                    // │             ║
                    // │             ║
                    // │═════════════╝
                    //
                    // however, this is.
                    // ╔═══════│═════╗
                    // ║       │     ║
                    // ║       │     ║
                    // ║       │     ║
                    // ╚═══════│═════╝

                    self.min().x < other.min().x && other.min().x < self.max().x
                } else {
                    // Horizontal line (| represent the line, ==== for rect)
                    //
                    // DE-9IM does not accept this as contained (as it is on the boundary)
                    // ───────────────
                    // ║             ║
                    // ║             ║
                    // ║             ║
                    // ╚═════════════╝
                    //
                    // however, this is.
                    // ╔═════════════╗
                    // ║             ║
                    // ║ ─────────── ║
                    // ║             ║
                    // ╚═════════════╝

                    self.min().y < other.min().y && other.min().y < self.max().y
                }
            }

            (TwoDimensional, ZeroDimensional) => self.contains(&other.min()),

            (OneDimensional, OneDimensional) => {
                if self.min().x == self.max().x {
                    // If the line is vertical we need the other line to also be vertical as well as bounds on y
                    other.min().x == self.min().x
                        && other.max().x == self.max().x
                        && self.min().y <= other.min().y
                        && other.max().y <= self.max().y
                } else {
                    other.min().y == self.min().y
                        && other.max().y == self.max().y
                        && self.min().x <= other.min().x
                        && other.max().x <= self.max().x
                }
            }

            (OneDimensional, ZeroDimensional) => {
                let p = other.min();

                // Need point not on the boundary of the line
                if self.min().x == self.max().x {
                    // Vertical line
                    p.x == self.min().x && self.min().y < p.y && p.y < self.max().y
                } else {
                    // Horizontal line
                    p.y == self.min().y && self.min().x < p.x && p.x < self.max().x
                }
            }

            (ZeroDimensional, ZeroDimensional) => self.min().contains(&other.min()),

            // A geometry cannot contain a higher-dimensional geometry.
            (ZeroDimensional, OneDimensional | TwoDimensional)
            | (OneDimensional, TwoDimensional) => false,

            (Empty, _) | (_, Empty) => unreachable!("Empty dimension, should not be possible."),
        }
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
