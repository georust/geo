use super::{impl_contains_from_relate, impl_contains_geometry_for, Contains};
use crate::dimensions::Dimensions;
use crate::{geometry::*, BoundingRect, CoordsIter, HasDimensions};
use crate::{kernels::Kernel, GeoFloat, GeoNum, Intersects, LinesIter, Orientation};
// ┌──────────────────────────────┐
// │ Implementations for Triangle │
// └──────────────────────────────┘

impl<T> Contains<Coord<T>> for Triangle<T>
where
    T: GeoNum,
{
    fn contains(&self, coord: &Coord<T>) -> bool {
        // leverageing robust predicates
        self.to_lines()
            .map(|l| T::Ker::orient2d(l.start, l.end, *coord))
            .windows(2)
            .all(|win| win[0] == win[1] && win[0] != Orientation::Collinear)

        // // neglecting robust prdicates, hence faster
        // let p0x = self.0.x.to_f64().unwrap();
        // let p0y = self.0.y.to_f64().unwrap();
        // let p1x = self.1.x.to_f64().unwrap();
        // let p1y = self.1.y.to_f64().unwrap();
        // let p2x = self.2.x.to_f64().unwrap();
        // let p2y = self.2.y.to_f64().unwrap();

        // let px = coord.x.to_f64().unwrap();
        // let py = coord.y.to_f64().unwrap();

        // let a = 0.5 * (-p1y * p2x + p0y * (-p1x + p2x) + p0x * (p1y - p2y) + p1x * p2y);

        // let sign = a.signum();

        // let s = (p0y * p2x - p0x * p2y + (p2y - p0y) * px + (p0x - p2x) * py) * sign;
        // let t = (p0x * p1y - p0y * p1x + (p0y - p1y) * px + (p1x - p0x) * py) * sign;

        // s > 0. && t > 0. && (s + t) < 2. * a * sign
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

impl<T> Contains<MultiPoint<T>> for Triangle<T>
where
    T: GeoNum,
    LineString<T>: Contains<MultiPoint<T>>,
    Point<T>: Contains<MultiPoint<T>>,
{
    fn contains(&self, rhs: &MultiPoint<T>) -> bool {
        if rhs.is_empty() {
            return false;
        }
        match (self.dimensions(), rhs.dimensions()) {
            (Dimensions::TwoDimensional, Dimensions::ZeroDimensional) => {
                // all intersect and at least one within
                rhs.coords_iter().all(|c| self.intersects(&c))
                    && rhs.coords_iter().any(|c| self.contains(&c))
            }
            (Dimensions::OneDimensional, _) => {
                LineString::from_iter(self.coords_iter()).contains(rhs)
            }
            (Dimensions::ZeroDimensional, _) => Point::from(self.0).contains(rhs),

            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
            (_, Dimensions::OneDimensional) => unreachable!("MultiPoint cannot be 1 dimensional"),
            (_, Dimensions::TwoDimensional) => unreachable!("MultiPoint cannot be 2 dimensional"),
        }
    }
}

impl<T> Contains<Line<T>> for Triangle<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>>,
    Triangle<T>: Intersects<Coord<T>> + HasDimensions,
    LineString<T>: Contains<Line<T>>,
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
            (Dimensions::ZeroDimensional, _) => Point::from(self.0).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
            (_, Dimensions::TwoDimensional) => unreachable!("Line cannot be 2 dimensional"),
        }
    }
}

impl<T> Contains<LineString<T>> for Triangle<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>>,
    Triangle<T>: Intersects<Coord<T>>,
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
            (Dimensions::ZeroDimensional, _) => Point::from(self.0).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
            (_, Dimensions::TwoDimensional) => unreachable!("LineString cannot be 2 dimensional"),
        }
    }
}

impl<T> Contains<MultiLineString<T>> for Triangle<T>
where
    T: GeoFloat,
    Line<T>: Contains<Line<T>>,
    Triangle<T>: Intersects<Coord<T>>,
{
    fn contains(&self, rhs: &MultiLineString<T>) -> bool {
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
            (Dimensions::ZeroDimensional, _) => Point::from(self.0).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
            (_, Dimensions::TwoDimensional) => unreachable!("LineString cannot be 2 dimensional"),
        }
    }
}

impl<T> Contains<Rect<T>> for Triangle<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>>,
    Triangle<T>: Intersects<Coord<T>> + Contains<Line<T>> + Contains<Coord<T>>,
    LineString<T>: Contains<Rect<T>>,
{
    fn contains(&self, rhs: &Rect<T>) -> bool {
        // in non-degenerate cases, all four corners intersecting the triangle implies edges crossing the triangle
        match (self.dimensions(), rhs.dimensions()) {
            (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                // standard case
                rhs.coords_iter().all(|c| self.intersects(&c))
            }
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                self.contains(&Line::new(rhs.min(), rhs.max()))
            }
            (Dimensions::TwoDimensional, Dimensions::ZeroDimensional) => self.contains(&rhs.min()),
            (Dimensions::OneDimensional, _) => {
                LineString::from_iter(self.coords_iter()).contains(rhs)
            }
            (Dimensions::ZeroDimensional, _) => Point::from(self.0).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
        }
    }
}

impl<T> Contains<Triangle<T>> for Triangle<T>
where
    T: GeoNum,
    Line<T>: Contains<Line<T>>,
    LineString<T>: Contains<Triangle<T>>,
    Triangle<T>: Intersects<Coord<T>>,
{
    fn contains(&self, rhs: &Triangle<T>) -> bool {
        // bounding box check
        // bounding box self !contains bounding box rhs iff
        // (1) bounding box b is degenerate and lies on bounding box a or (2) some part of b not in a
        // if case (1), then there canno be any part of b within a
        // if case (2), then a cannot contains b
        // and rect contains rect is cheap
        if !self.bounding_rect().contains(&rhs.bounding_rect()) {
            return false;
        }

        match (self.dimensions(), rhs.dimensions()) {
            (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                // standard case
                // all points of RHS must intersect self
                // by being 2d shape, there must be some area within RHS that is not on the boundary of self
                self.intersects(&rhs.0) && self.intersects(&rhs.1) && self.intersects(&rhs.2)
            }
            (Dimensions::TwoDimensional, Dimensions::OneDimensional) => {
                self.contains(&LineString::from_iter(rhs.coords_iter()))
            }
            (Dimensions::TwoDimensional, Dimensions::ZeroDimensional) => self.contains(&rhs.0),
            (Dimensions::OneDimensional, _) => {
                LineString::from_iter(self.coords_iter()).contains(rhs)
            }
            (Dimensions::ZeroDimensional, _) => Point::from(self.0).contains(rhs),
            (Dimensions::Empty, _) => false,
            (_, Dimensions::Empty) => false,
        }
    }
}

impl_contains_from_relate!(Triangle<T>, [Polygon<T>, MultiPolygon<T>, GeometryCollection<T>]);
impl_contains_geometry_for!(Triangle<T>);

#[cfg(test)]
mod tests_linestring {
    use super::*;
    use crate::{coord, line_string};

    #[test]
    fn tri_contains_linestring() {
        let tri = Triangle::new(
            coord! {x:0.,y:0.},
            coord! {x:10.,y:0.},
            coord! {x:10.,y:10.},
        );
        let ls_within = line_string![(x: 3., y: 2.),(x: 5., y: 3.),];
        let ls_boundary = LineString::from_iter(tri.exterior_coords_iter());
        let ls_cross_in = line_string![(x: 1., y: 0.),(x:10., y: 1.),];
        let ls_cross_out = line_string![(x: 0., y: 0.),(x: 10., y: 11.),];

        assert!(tri.contains(&ls_within));
        assert!(!tri.contains(&ls_boundary));
        assert!(tri.contains(&ls_cross_in));
        assert!(!tri.contains(&ls_cross_out));
    }
}

#[cfg(test)]
mod test_triangle {
    use super::*;
    use crate::coord;

    #[test]
    fn triangle_contains_triangle() {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );

        let tri_within = Triangle::new(
            coord! {x:1., y:1.},
            coord! {x:9., y:1.},
            coord! {x:9., y:9.},
        );
        let tri_edges = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:5., y:5.},
        );

        assert!(tri.contains(&tri));
        assert!(tri.contains(&tri_within));
        assert!(tri.contains(&tri_edges));
    }
}

#[cfg(test)]
mod test_rect {
    use super::*;
    use crate::coord;

    #[test]
    fn triangle_contains_rect() {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:10.},
        );

        let rect_bounds = Rect::new(Point::new(10., 0.), Point::new(5., 5.));
        let rect_within = Rect::new(Point::new(9., 1.), Point::new(4., 4.));

        assert!(tri.contains(&rect_bounds));
        assert!(tri.contains(&rect_within));
    }
}

#[cfg(test)]
mod test_line {
    use super::*;
    use crate::{coord, Line, Point, Relate, Triangle};

    #[test]
    fn triangle_i32() {
        let tri = Triangle::new(coord! {x:0, y:0}, coord! {x:10, y:0}, coord! {x:10, y:5});
        let line_boundary = Line::new(Point::new(2, 1), Point::new(0, 0));
        let line_within = Line::new(Point::new(0, 0), Point::new(10, 1));
        let line_disjoint = Line::new(Point::new(0, -1), Point::new(10, -1));

        assert!(!tri.contains(&line_boundary));
        assert!(tri.contains(&line_within));
        assert!(!tri.contains(&line_disjoint));
    }

    #[test]
    fn triangle2d_contains_line0d() {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:50.},
        );
        let line = Line::new(Point::new(1., 1.), Point::new(1., 1.));

        assert!(tri.contains(&line));
        assert!(tri.relate(&line).is_contains());
    }
    #[test]
    fn triangle1d_contains_line1d() {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:5., y:0.},
        );
        let line = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        assert!(tri.contains(&line));
        assert!(tri.relate(&line).is_contains());
    }

    #[test]
    fn triangle1d_contains_line0d() {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:5., y:0.},
        );
        let line = Line::new(Point::new(1., 0.), Point::new(1., 0.));

        assert!(tri.contains(&line));
        assert!(tri.relate(&line).is_contains());
    }

    #[test]
    fn triangle0d_contains_line0d() {
        // Diverge from Relate Trait here because (0,0) should contain (0,0)

        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:0., y:0.},
            coord! {x:0., y:0.},
        );
        let line = Line::new(Point::new(0., 0.), Point::new(0., 0.));

        assert!(tri.contains(&line));
        assert!(!tri.relate(&line).is_contains());
    }

    #[test]
    fn triangle_contains_line() {
        let tri = Triangle::new(
            coord! {x:0., y:0.},
            coord! {x:10., y:0.},
            coord! {x:10., y:5.},
        );

        let ln_within = Line::new(Point::new(3., 1.), Point::new(9., 3.));
        let ln_in_cross = Line::new(Point::new(0., 0.), Point::new(10., 4.));

        let ln_boundary = Line::new(Point::new(0., 0.), Point::new(10., 0.));
        let ln_boundary_partial = Line::new(Point::new(1., 0.), Point::new(9., 0.));

        let ln_disjoint = Line::new(Point::new(0., 6.), Point::new(10., 6.));
        let ln_out_cross = Line::new(Point::new(0., 0.), Point::new(9., 6.));

        assert!(tri.contains(&ln_within));
        assert!(tri.relate(&ln_within).is_contains());

        assert!(tri.contains(&ln_in_cross));
        assert!(tri.relate(&ln_in_cross).is_contains());

        assert!(!tri.contains(&ln_boundary));
        assert!(!tri.relate(&ln_boundary).is_contains());

        assert!(!tri.contains(&ln_boundary_partial));
        assert!(!tri.relate(&ln_boundary_partial).is_contains());

        assert!(!tri.contains(&ln_disjoint));
        assert!(!tri.relate(&ln_disjoint).is_contains());

        assert!(!tri.contains(&ln_out_cross));
        assert!(!tri.relate(&ln_out_cross).is_contains());
    }

    /**
     *  Implementation of Triangle contains Line requires line contains self to be true
     */
    #[test]
    fn line_contains_self() {
        let ln = Line::new(Point::new(0., 0.), Point::new(10., 0.));
        assert!(ln.contains(&ln));
        assert!(ln.relate(&ln).is_contains());
    }
}
