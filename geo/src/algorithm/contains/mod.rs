/// Checks if `rhs` is completely contained within `self`.
/// More formally, the interior of `rhs` has non-empty
/// (set-theoretic) intersection but neither the interior,
/// nor the boundary of `rhs` intersects the exterior of
/// `self`. In other words, the [DE-9IM] intersection matrix
/// of `(rhs, self)` is `T*F**F***`.
///
/// [DE-9IM]: https://en.wikipedia.org/wiki/DE-9IM
///
/// # Examples
///
/// ```
/// use geo::algorithm::contains::Contains;
/// use geo::{line_string, point, Polygon};
///
/// let line_string = line_string![
///     (x: 0., y: 0.),
///     (x: 2., y: 0.),
///     (x: 2., y: 2.),
///     (x: 0., y: 2.),
///     (x: 0., y: 0.),
/// ];
///
/// let polygon = Polygon::new(line_string.clone(), vec![]);
///
/// // Point in Point
/// assert!(point!(x: 2., y: 0.).contains(&point!(x: 2., y: 0.)));
///
/// // Point in Linestring
/// assert!(line_string.contains(&point!(x: 2., y: 0.)));
///
/// // Point in Polygon
/// assert!(polygon.contains(&point!(x: 1., y: 1.)));
/// ```
pub trait Contains<Rhs = Self> {
    fn contains(&self, rhs: &Rhs) -> bool;
}

mod geometry;
mod line;
mod line_string;
mod point;
mod polygon;
mod rect;
mod triangle;

// ┌───────┐
// │ Tests │
// └───────┘

#[cfg(test)]
mod test {
    use crate::algorithm::contains::Contains;
    use crate::line_string;
    use crate::{Coordinate, Line, LineString, MultiPolygon, Point, Polygon, Rect, Triangle};

    #[test]
    // see https://github.com/georust/geo/issues/452
    fn linestring_contains_point() {
        let line_string = LineString::from(vec![(0., 0.), (3., 3.)]);
        let point_on_line = Point::new(1., 1.);
        assert!(line_string.contains(&point_on_line));
    }
    #[test]
    // V doesn't contain rect because two of its edges intersect with V's exterior boundary
    fn polygon_does_not_contain_polygon() {
        let v = Polygon::new(
            vec![
                (150., 350.),
                (100., 350.),
                (210., 160.),
                (290., 350.),
                (250., 350.),
                (200., 250.),
                (150., 350.),
            ]
            .into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (250., 310.),
                (150., 310.),
                (150., 280.),
                (250., 280.),
                (250., 310.),
            ]
            .into(),
            vec![],
        );
        assert_eq!(!v.contains(&rect), true);
    }
    #[test]
    // V contains rect because all its vertices are contained, and none of its edges intersect with V's boundaries
    fn polygon_contains_polygon() {
        let v = Polygon::new(
            vec![
                (150., 350.),
                (100., 350.),
                (210., 160.),
                (290., 350.),
                (250., 350.),
                (200., 250.),
                (150., 350.),
            ]
            .into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (185., 237.),
                (220., 237.),
                (220., 220.),
                (185., 220.),
                (185., 237.),
            ]
            .into(),
            vec![],
        );
        assert_eq!(v.contains(&rect), true);
    }
    #[test]
    // LineString is fully contained
    fn linestring_fully_contained_in_polygon() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![],
        );
        let ls = LineString::from(vec![(3.0, 0.5), (3.0, 3.5)]);
        assert_eq!(poly.contains(&ls), true);
    }
    /// Tests: Point in LineString
    #[test]
    fn empty_linestring_test() {
        let linestring = LineString(Vec::new());
        assert!(!linestring.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn linestring_point_is_vertex_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.)]);
        // Note: the end points of a linestring are not
        // considered to be "contained"
        assert!(linestring.contains(&Point::new(2., 0.)));
        assert!(!linestring.contains(&Point::new(0., 0.)));
        assert!(!linestring.contains(&Point::new(2., 2.)));
    }
    #[test]
    fn linestring_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.)]);
        assert!(linestring.contains(&Point::new(1., 0.)));
    }
    /// Tests: Point in Polygon
    #[test]
    fn empty_polygon_test() {
        let linestring = LineString(Vec::new());
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_one_point_test() {
        let linestring = LineString::from(vec![(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(3., 1.)));
    }
    #[test]
    fn polygon_with_one_point_is_vertex_test() {
        let linestring = LineString::from(vec![(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_point_on_boundary_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(1., 0.)));
        assert!(!poly.contains(&Point::new(2., 1.)));
        assert!(!poly.contains(&Point::new(1., 2.)));
        assert!(!poly.contains(&Point::new(0., 1.)));
    }
    #[test]
    fn point_in_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(1., 1.)));
    }
    #[test]
    fn point_in_polygon_with_ray_passing_through_a_vertex_test() {
        let linestring = LineString::from(vec![(1., 0.), (0., 1.), (-1., 0.), (0., -1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(0., 0.)));
    }
    #[test]
    fn point_in_polygon_with_ray_passing_through_a_vertex_and_not_crossing() {
        let linestring = LineString::from(vec![
            (0., 0.),
            (2., 0.),
            (3., 1.),
            (4., 0.),
            (4., 2.),
            (0., 2.),
            (0., 0.),
        ]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&Point::new(1., 1.)));
    }
    #[test]
    fn point_out_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2.1, 1.)));
        assert!(!poly.contains(&Point::new(1., 2.1)));
        assert!(!poly.contains(&Point::new(2.1, 2.1)));
    }
    #[test]
    fn point_polygon_with_inner_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let inner_linestring = LineString::from(vec![
            [0.5, 0.5],
            [1.5, 0.5],
            [1.5, 1.5],
            [0.0, 1.5],
            [0.0, 0.0],
        ]);
        let poly = Polygon::new(linestring, vec![inner_linestring]);
        assert!(!poly.contains(&Point::new(0.25, 0.25)));
        assert!(!poly.contains(&Point::new(1., 1.)));
        assert!(!poly.contains(&Point::new(1.5, 1.5)));
        assert!(!poly.contains(&Point::new(1.5, 1.)));
    }

    /// Tests: Point in MultiPolygon
    #[test]
    fn empty_multipolygon_test() {
        let multipoly = MultiPolygon(Vec::new());
        assert!(!multipoly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (1., 0.), (1., 1.), (0., 1.), (0., 0.)]),
            Vec::new(),
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(2., 0.), (3., 0.), (3., 1.), (2., 1.), (2., 0.)]),
            Vec::new(),
        );
        let multipoly = MultiPolygon(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains(&Point::new(1.5, 0.5)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_and_inner_test() {
        let poly1 = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![LineString::from(vec![
                (1., 1.),
                (4., 1.),
                (4., 4.),
                (1., 1.),
            ])],
        );
        let poly2 = Polygon::new(
            LineString::from(vec![(9., 0.), (14., 0.), (14., 4.), (9., 4.), (9., 0.)]),
            Vec::new(),
        );

        let multipoly = MultiPolygon(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(3., 5.)));
        assert!(multipoly.contains(&Point::new(12., 2.)));
        assert!(!multipoly.contains(&Point::new(3., 2.)));
        assert!(!multipoly.contains(&Point::new(7., 2.)));
    }
    /// Tests: LineString in Polygon
    #[test]
    fn linestring_in_polygon_with_linestring_is_boundary_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring.clone(), Vec::new());
        assert!(!poly.contains(&linestring.clone()));
        assert!(!poly.contains(&LineString::from(vec![(0., 0.), (2., 0.)])));
        assert!(!poly.contains(&LineString::from(vec![(2., 0.), (2., 2.)])));
        assert!(!poly.contains(&LineString::from(vec![(0., 2.), (0., 0.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let linestring = LineString::from(vec![(0., 0.), (2., 0.), (2., 2.), (0., 2.), (0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&LineString::from(vec![(1., 1.), (3., 0.)])));
        assert!(!poly.contains(&LineString::from(vec![(3., 0.), (5., 2.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            vec![LineString::from(vec![
                (1., 1.),
                (4., 1.),
                (4., 4.),
                (1., 4.),
                (1., 1.),
            ])],
        );
        assert!(!poly.contains(&LineString::from(vec![(2., 2.), (3., 3.)])));
        assert!(!poly.contains(&LineString::from(vec![(2., 2.), (2., 5.)])));
        assert!(!poly.contains(&LineString::from(vec![(3., 0.5), (3., 5.)])));
    }
    #[test]
    fn bounding_rect_in_inner_bounding_rect_test() {
        let bounding_rect_xl = Rect::new(
            Coordinate { x: -100., y: -200. },
            Coordinate { x: 100., y: 200. },
        );
        let bounding_rect_sm = Rect::new(
            Coordinate { x: -10., y: -20. },
            Coordinate { x: 10., y: 20. },
        );
        assert_eq!(true, bounding_rect_xl.contains(&bounding_rect_sm));
        assert_eq!(false, bounding_rect_sm.contains(&bounding_rect_xl));
    }
    #[test]
    fn point_in_line_test() {
        let c = |x, y| Coordinate { x, y };
        let p0 = c(2., 4.);
        // vertical line
        let line1 = Line::new(c(2., 0.), c(2., 5.));
        // point on line, but outside line segment
        let line2 = Line::new(c(0., 6.), c(1.5, 4.5));
        // point on line
        let line3 = Line::new(c(0., 6.), c(3., 3.));
        assert!(line1.contains(&Point(p0)));
        assert!(!line2.contains(&Point(p0)));
        assert!(line3.contains(&Point(p0)));
    }
    #[test]
    fn line_in_line_test() {
        let c = |x, y| Coordinate { x, y };
        let line0 = Line::new(c(0., 1.), c(3., 4.));
        // first point on line0, second not
        let line1 = Line::new(c(1., 2.), c(2., 2.));
        // co-linear, but extends past the end of line0
        let line2 = Line::new(c(1., 2.), c(4., 5.));
        // contained in line0
        let line3 = Line::new(c(1., 2.), c(3., 4.));
        assert!(!line0.contains(&line1));
        assert!(!line0.contains(&line2));
        assert!(line0.contains(&line3));
    }
    #[test]
    fn linestring_in_line_test() {
        let line = Line::from([(0, 10), (30, 40)]);
        // linestring0 in line
        let linestring0 = LineString::from(vec![(01, 11), (10, 20), (15, 25)]);
        // linestring1 starts and ends in line, but wanders in the middle
        let linestring1 = LineString::from(vec![(01, 11), (20, 20), (15, 25)]);
        // linestring2 is co-linear, but extends beyond line
        let linestring2 = LineString::from(vec![(01, 11), (10, 20), (40, 50)]);
        // no part of linestring3 is contained in line
        let linestring3 = LineString::from(vec![(11, 11), (20, 20), (25, 25)]);
        // a linestring with singleton interior on the boundary of the line
        let linestring4 = LineString::from(vec![(0, 10), (0, 10), (0, 10)]);
        // a linestring with singleton interior that is contained in the line
        let linestring5 = LineString::from(vec![(1, 11), (1, 11), (1, 11)]);
        assert!(line.contains(&linestring0));
        assert!(!line.contains(&linestring1));
        assert!(!line.contains(&linestring2));
        assert!(!line.contains(&linestring3));
        assert!(!line.contains(&linestring4));
        assert!(line.contains(&linestring5));
    }
    #[test]
    fn line_in_polygon_test() {
        let c = |x, y| Coordinate { x, y };
        let line = Line::new(c(0, 10), c(30, 40));
        let linestring0 = line_string![c(-10, 0), c(50, 0), c(50, 50), c(0, 50), c(-10, 0)];
        let poly0 = Polygon::new(linestring0, Vec::new());
        let linestring1 = line_string![c(0, 0), c(0, 20), c(20, 20), c(20, 0), c(0, 0)];
        let poly1 = Polygon::new(linestring1, Vec::new());
        assert!(poly0.contains(&line));
        assert!(!poly1.contains(&line));
    }
    #[test]
    #[ignore]
    fn line_in_polygon_edgecases_test() {
        // Some DE-9IM edge cases for checking line is
        // inside polygon The end points of the line can be
        // on the boundary of the polygon but we don't allow
        // that yet.
        let c = |x, y| Coordinate { x, y };
        // A non-convex polygon
        let linestring0 = line_string![c(0, 0), c(1, 1), c(1, -1), c(-1, -1), c(-1, 1)];
        let poly = Polygon::new(linestring0, Vec::new());

        assert!(poly.contains(&Line::new(c(0, 0), c(1, -1))));
        assert!(poly.contains(&Line::new(c(-1, 1), c(1, -1))));
        assert!(!poly.contains(&Line::new(c(-1, 1), c(1, 1))));
    }
    #[test]
    fn line_in_linestring_edgecases() {
        let c = |x, y| Coordinate { x, y };
        use crate::line_string;
        let mut ls = line_string![c(0, 0), c(1, 0), c(0, 1), c(-1, 0)];
        assert!(!ls.contains(&Line::from([(0, 0), (0, 0)])));
        ls.close();
        assert!(ls.contains(&Line::from([(0, 0), (0, 0)])));
        assert!(ls.contains(&Line::from([(-1, 0), (1, 0)])));
    }
    #[test]
    fn line_in_linestring_test() {
        let line0 = Line::from([(1., 1.), (2., 2.)]);
        // line0 is completely contained in the second segment
        let linestring0 = LineString::from(vec![(0., 0.5), (0.5, 0.5), (3., 3.)]);
        // line0 is contained in the last three segments
        let linestring1 = LineString::from(vec![
            (0., 0.5),
            (0.5, 0.5),
            (1.2, 1.2),
            (1.5, 1.5),
            (3., 3.),
        ]);
        // line0 endpoints are contained in the linestring, but the fourth point is off the line
        let linestring2 = LineString::from(vec![
            (0., 0.5),
            (0.5, 0.5),
            (1.2, 1.2),
            (1.5, 0.),
            (2., 2.),
            (3., 3.),
        ]);
        assert!(linestring0.contains(&line0));
        assert!(linestring1.contains(&line0));
        assert!(!linestring2.contains(&line0));
    }

    #[test]
    fn integer_bounding_rects() {
        let p: Point<i32> = Point::new(10, 20);
        let bounding_rect: Rect<i32> =
            Rect::new(Coordinate { x: 0, y: 0 }, Coordinate { x: 100, y: 100 });
        assert!(bounding_rect.contains(&p));
        assert!(!bounding_rect.contains(&Point::new(-10, -10)));

        let smaller_bounding_rect: Rect<i32> =
            Rect::new(Coordinate { x: 10, y: 10 }, Coordinate { x: 20, y: 20 });
        assert!(bounding_rect.contains(&smaller_bounding_rect));
    }

    #[test]
    fn triangle_not_contains_point_on_edge() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 0.0);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_on_vertex() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(2.0, 0.0);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_contains_point_inside() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 0.5);
        assert!(t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_above() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(1.0, 1.5);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_not_contains_point_below() {
        let t = Triangle::from([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0)]);
        let p = Point::new(-1.0, 0.5);
        assert!(!t.contains(&p));
    }

    #[test]
    fn triangle_contains_neg_point() {
        let t = Triangle::from([(0.0, 0.0), (-2.0, 0.0), (-2.0, -2.0)]);
        let p = Point::new(-1.0, -0.5);
        assert!(t.contains(&p));
    }

    #[test]
    // https://github.com/georust/geo/issues/473
    fn triangle_contains_collinear_points() {
        let origin: Coordinate<f64> = (0., 0.).into();
        let tri = Triangle(origin, origin, origin);
        let pt: Point<f64> = (0., 1.23456).into();
        assert!(!tri.contains(&pt));
        let pt: Point<f64> = (0., 0.).into();
        assert!(!tri.contains(&pt));
        let origin: Coordinate<f64> = (0., 0.).into();
        let tri = Triangle((1., 1.).into(), origin, origin);
        let pt: Point<f64> = (1., 1.).into();
        assert!(!tri.contains(&pt));
        let pt: Point<f64> = (0.5, 0.5).into();
        assert!(!tri.contains(&pt));
    }
}
