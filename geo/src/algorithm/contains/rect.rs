use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::dimensions::Dimensions;
use crate::{geometry::*, Area, CoordsIter, HasDimensions, Intersects, LinesIter};
use crate::{CoordFloat, CoordNum, GeoFloat, GeoNum};

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
    T: GeoNum,
    Line<T> : Contains<Coord<T>>+Contains<Line<T>>,
    Rect<T> : Contains<Line<T>>+Contains<Coord<T>>,
{
    fn contains(&self, other: &Rect<T>) -> bool {
        match(self.dimensions(), other.dimensions()) {
            (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                // standard case
                self.min().x <= other.min().x
                    && self.max().x >= other.max().x
                    && self.min().y <= other.min().y
                    && self.max().y >= other.max().y
            }
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
               self.contains(&Line::<T>::new(other.min(), other.max()))
            }
            (Dimensions::TwoDimensional, Dimensions::ZeroDimensional) => self.contains(&other.min()),
            // individual cases for OneDimensional because Line contains Rect is currently a Relate Trait impl
             (Dimensions::OneDimensional, Dimensions::TwoDimensional) => false,
                 (Dimensions::OneDimensional, Dimensions::OneDimensional) => {
                Line::new(self.min(), self.max()).contains(&Line::new(other.min(), other.max()))
                }
            (Dimensions::OneDimensional, Dimensions::ZeroDimensional) => {
                Line::<T>::new(self.min(), self.max()).contains(&other.min())
                }
            (Dimensions::ZeroDimensional, _) => Point::<T>::from(self.min()).contains(other),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
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

impl<T> Contains<Line<T>> for Rect<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>>,
    Rect<T>: Intersects<Coord<T>>,
{
    fn contains(&self, rhs: &Line<T>) -> bool {
        match (self.dimensions(), rhs.dimensions()) {
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                // standard case
                self.intersects(&rhs.start)
                    && self.intersects(&rhs.end)
                    && (self.contains(&rhs.start)
                        || self.contains(&rhs.end)
                        || !self.lines_iter().any(|edge| edge.contains(rhs)))
            }
            (Dimensions::TwoDimensional, Dimensions::ZeroDimensional) => self.contains(&rhs.start),
            (Dimensions::OneDimensional, _) => {
                LineString::from_iter(self.coords_iter()).contains(rhs)
            }
            (Dimensions::ZeroDimensional, _) => Point::from(self.min()).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
            (_, Dimensions::TwoDimensional) => unreachable!("Line cannot be 2 dimensional"),
        }
    }
}

impl<T> Contains<Triangle<T>> for Rect<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>> + Contains<LineString<T>> + Contains<Triangle<T>>,
    Rect<T>: Intersects<Coord<T>>,
    Point<T>: Contains<Triangle<T>>,
{
    fn contains(&self, rhs: &Triangle<T>) -> bool {
        // in non-degenerate cases, all three triangle corners intersecting the rectangle implies one edge crosses the rectangle
        match (self.dimensions(), rhs.dimensions()) {
            (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                // standard case
                self.intersects(&rhs.0) && self.intersects(&rhs.1) && self.intersects(&rhs.2)
            }
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                self.contains(&LineString::from_iter(rhs.coords_iter()))
            }
            (Dimensions::TwoDimensional, Dimensions::ZeroDimensional) => self.contains(&rhs.0),
            (Dimensions::OneDimensional, _) => Line::new(self.min(), self.max()).contains(rhs),
            (Dimensions::ZeroDimensional, _) => Point::from(self.min()).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
        }
    }
}

impl<T> Contains<LineString<T>> for Rect<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>>,
    Rect<T>: Intersects<Coord<T>>,
{
    fn contains(&self, rhs: &LineString<T>) -> bool {
        match (self.dimensions(), rhs.dimensions()) {
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                // standard case
                // self intersects all points
                rhs.coords_iter().all(|c| self.intersects(&c))
                // either a point
                &&( rhs.coords_iter().any(|c| self.contains(&c))
                // or there exists a line which does not line on any of the self's edges
                || rhs.lines_iter().any(|rhs_edge| !self.lines_iter().any(|edge| edge.contains(&rhs_edge)))
            )
            }
            (Dimensions::TwoDimensional, Dimensions::ZeroDimensional) => self.contains(&rhs.0[0]),
            (Dimensions::OneDimensional, _) => {
                LineString::from_iter(self.coords_iter()).contains(rhs)
            }
            (Dimensions::ZeroDimensional, _) => Point::from(self.min()).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
            (_, Dimensions::TwoDimensional) => unreachable!("LineString cannot be 2 dimensional"),
        }
    }
}

impl_contains_from_relate!(Rect<T>, [ MultiPoint<T>, MultiLineString<T>, MultiPolygon<T>, GeometryCollection<T> ]);
impl_contains_geometry_for!(Rect<T>);

#[cfg(test)]
mod tests_linestring {
    use super::*;
    use crate::{line_string, Point};

    #[test]
    fn rect_contains_linestring() {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ls_within = line_string![(x: 3., y: 2.),(x: 7., y: 5.),];
        let ls_boundary = LineString::from_iter(rect.exterior_coords_iter());
        let ls_cross_in = line_string![(x: 0., y: 1.),(x: 3., y: 5.),];
        let ls_cross_out = line_string![(x: 0., y: 0.),(x: 10., y: 6.),];

        assert!(rect.contains(&ls_within));
        assert!(!rect.contains(&ls_boundary));
        assert!(rect.contains(&ls_cross_in));
        assert!(!rect.contains(&ls_cross_out));
    }
}

#[cfg(test)]
mod tests_triangle {
    use super::*;
    use crate::coord;

    #[test]
    fn rect_contains_triangle() {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 10.));
        let tri_bound = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );
        let tri_within = Triangle::new(
            coord! {x:1., y:1.},
            coord! {x:9., y:1.},
            coord! {x:9., y:9.},
        );
        assert!(rect.contains(&tri_bound));
        assert!(rect.contains(&tri_within));
    }
}


#[cfg(test)]
mod tests_rect {
    use super::*;
    use crate::{ Point, Rect, Relate};

    #[test]
    fn rect2d_contains_line0d() {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let rect_1d = Rect::new(Point::new(0., 0.), Point::new(10., 0.));
        let rect_0d = Rect::new(Point::new(0., 0.), Point::new(0., 0.));

        assert!(rect.contains(&rect));
        assert!(rect.relate(&rect).is_contains());
        
        assert!(!rect.contains(&rect_1d));
        assert!(!rect.relate(&rect_1d).is_contains());
        
        assert!(!rect.contains(&rect_0d));
        assert!(!rect.relate(&rect_0d).is_contains());

    }
}
#[cfg(test)]
mod tests_line {
    use super::*;
    use crate::{Line, Point, Rect, Relate};

    #[test]
    fn rect2d_contains_line0d() {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));
        let ln = Line::new(Point::new(1., 1.), Point::new(1., 1.));
        assert!(rect.contains(&ln));
        assert!(rect.relate(&ln).is_contains());
    }
    #[test]
    fn rect1d_contains_line1d() {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 0.));
        let ln = Line::new(Point::new(1., 0.), Point::new(9., 0.));
        assert!(rect.contains(&ln));
        assert!(rect.relate(&ln).is_contains());
    }
    #[test]
    fn rect1d_contains_line0d() {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 0.));
        let ln = Line::new(Point::new(1., 0.), Point::new(1., 0.));
        assert!(rect.contains(&ln));
        assert!(rect.relate(&ln).is_contains());
    }
    #[test]
    fn rect0d_contains_line0d() {
        //diverge from Relate Trait here because (0,0) should contain (0,0)
        let rect = Rect::new(Point::new(1., 0.), Point::new(1., 0.));
        let ln = Line::new(Point::new(1., 0.), Point::new(1., 0.));
        assert!(rect.contains(&ln));
        assert!(!rect.relate(&ln).is_contains());
    }

    #[test]
    fn rect_contains_line() {
        let rect = Rect::new(Point::new(0., 0.), Point::new(10., 5.));

        let ln_within = Line::new(Point::new(1., 1.), Point::new(9., 4.));
        let ln_in_cross = Line::new(Point::new(0., 0.), Point::new(10., 5.));

        let ln_boundary = Line::new(Point::new(0., 0.), Point::new(10., 0.));
        let ln_boundary_partial = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        let ln_disjoint = Line::new(Point::new(0., 6.), Point::new(10., 6.));
        let ln_out_cross = Line::new(Point::new(0., 0.), Point::new(10., 10.));

        assert!(rect.contains(&ln_within));
        assert!(rect.relate(&ln_within).is_contains());

        assert!(rect.contains(&ln_in_cross));
        assert!(rect.relate(&ln_in_cross).is_contains());

        assert!(!rect.contains(&ln_boundary));
        assert!(!rect.relate(&ln_boundary).is_contains());

        assert!(!rect.contains(&ln_boundary_partial));
        assert!(!rect.relate(&ln_boundary_partial).is_contains());

        assert!(!rect.contains(&ln_disjoint));
        assert!(!rect.relate(&ln_disjoint).is_contains());

        assert!(!rect.contains(&ln_out_cross));
        assert!(!rect.relate(&ln_out_cross).is_contains());
    }

    /**
     *  Implementation of Rect contains Line requires line contains self to be true
     */
    #[test]
    fn line_contains_self() {
        let ln = Line::new(Point::new(0., 0.), Point::new(10., 0.));
        assert!(ln.contains(&ln));
        assert!(ln.relate(&ln).is_contains());
    }
}
