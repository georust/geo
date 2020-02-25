use crate::algorithm::contains::Contains;
use crate::{Line, LineString, Point, Polygon, Rect};
use num_traits::Float;

/// Checks if the geometry A intersects the geometry B.

pub trait Intersects<Rhs = Self> {
    /// Checks if the geometry A intersects the geometry B.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::intersects::Intersects;
    /// use geo::{Coordinate, LineString, Point};
    ///
    /// let linestring = LineString::from(vec![(3., 2.), (7., 6.)]);
    ///
    /// assert!(linestring.intersects(&LineString::from(vec![(3., 4.), (8., 4.)])));
    /// assert!(!linestring.intersects(&LineString::from(vec![(9., 2.), (11., 5.)])));
    /// ```
    fn intersects(&self, rhs: &Rhs) -> bool;
}

impl<T> Intersects<Point<T>> for Line<T>
where
    T: Float,
{
    fn intersects(&self, p: &Point<T>) -> bool {
        let tx = if self.dx() == T::zero() {
            None
        } else {
            Some((p.x() - self.start.x) / self.dx())
        };
        let ty = if self.dy() == T::zero() {
            None
        } else {
            Some((p.y() - self.start.y) / self.dy())
        };
        match (tx, ty) {
            (None, None) => {
                // Degenerate line
                p.0 == self.start
            }
            (Some(t), None) => {
                // Horizontal line
                p.y() == self.start.y && T::zero() <= t && t <= T::one()
            }
            (None, Some(t)) => {
                // Vertical line
                p.x() == self.start.x && T::zero() <= t && t <= T::one()
            }
            (Some(t_x), Some(t_y)) => {
                // All other lines
                (t_x - t_y).abs() <= T::epsilon() && T::zero() <= t_x && t_x <= T::one()
            }
        }
    }
}

impl<T> Intersects<Line<T>> for Point<T>
where
    T: Float,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        line.intersects(self)
    }
}

impl<T> Intersects<Line<T>> for Line<T>
where
    T: Float,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        // Using Cramer's Rule:
        // https://en.wikipedia.org/wiki/Intersection_%28Euclidean_geometry%29#Two_line_segments
        let a1 = self.dx();
        let a2 = self.dy();
        let b1 = -line.dx();
        let b2 = -line.dy();
        let c1 = line.start.x - self.start.x;
        let c2 = line.start.y - self.start.y;

        let d = a1 * b2 - a2 * b1;
        if d == T::zero() {
            let (self_start, self_end) = self.points();
            let (other_start, other_end) = line.points();
            // lines are parallel
            // return true iff at least one endpoint intersects the other line
            self_start.intersects(line)
                || self_end.intersects(line)
                || other_start.intersects(self)
                || other_end.intersects(self)
        } else {
            let s = (c1 * b2 - c2 * b1) / d;
            let t = (a1 * c2 - a2 * c1) / d;
            (T::zero() <= s) && (s <= T::one()) && (T::zero() <= t) && (t <= T::one())
        }
    }
}

impl<T> Intersects<LineString<T>> for Line<T>
where
    T: Float,
{
    fn intersects(&self, linestring: &LineString<T>) -> bool {
        linestring.lines().any(|line| self.intersects(&line))
    }
}

impl<T> Intersects<Line<T>> for LineString<T>
where
    T: Float,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        line.intersects(self)
    }
}

impl<T> Intersects<Polygon<T>> for Line<T>
where
    T: Float,
{
    fn intersects(&self, p: &Polygon<T>) -> bool {
        p.exterior().intersects(self)
            || p.interiors().iter().any(|inner| inner.intersects(self))
            || p.contains(&self.start_point())
            || p.contains(&self.end_point())
    }
}

impl<T> Intersects<Line<T>> for Polygon<T>
where
    T: Float,
{
    fn intersects(&self, line: &Line<T>) -> bool {
        line.intersects(self)
    }
}

impl<T> Intersects<LineString<T>> for LineString<T>
where
    T: Float,
{
    // See: https://github.com/brandonxiang/geojson-python-utils/blob/33b4c00c6cf27921fb296052d0c0341bd6ca1af2/geojson_utils.py
    fn intersects(&self, linestring: &LineString<T>) -> bool {
        if self.0.is_empty() || linestring.0.is_empty() {
            return false;
        }
        for a in self.lines() {
            for b in linestring.lines() {
                let u_b = b.dy() * a.dx() - b.dx() * a.dy();
                if u_b == T::zero() {
                    continue;
                }
                let ua_t = b.dx() * (a.start.y - b.start.y) - b.dy() * (a.start.x - b.start.x);
                let ub_t = a.dx() * (a.start.y - b.start.y) - a.dy() * (a.start.x - b.start.x);
                let u_a = ua_t / u_b;
                let u_b = ub_t / u_b;
                if (T::zero() <= u_a)
                    && (u_a <= T::one())
                    && (T::zero() <= u_b)
                    && (u_b <= T::one())
                {
                    return true;
                }
            }
        }
        false
    }
}

impl<T> Intersects<LineString<T>> for Polygon<T>
where
    T: Float,
{
    fn intersects(&self, linestring: &LineString<T>) -> bool {
        // line intersects inner or outer polygon edge
        if self.exterior().intersects(linestring)
            || self
                .interiors()
                .iter()
                .any(|inner| inner.intersects(linestring))
        {
            true
        } else {
            // or if it's contained in the polygon
            linestring.points_iter().any(|point| self.contains(&point))
        }
    }
}

impl<T> Intersects<Polygon<T>> for LineString<T>
where
    T: Float,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        polygon.intersects(self)
    }
}

// helper function for intersection check
fn value_in_range<T>(value: T, min: T, max: T) -> bool
where
    T: Float,
{
    (value >= min) && (value <= max)
}

impl<T> Intersects<Rect<T>> for Rect<T>
where
    T: Float,
{
    fn intersects(&self, bounding_rect: &Rect<T>) -> bool {
        // line intersects inner or outer polygon edge
        if bounding_rect.contains(self) {
            false
        } else {
            let x_overlap = value_in_range(
                self.min().x,
                bounding_rect.min().x,
                bounding_rect.min().x + bounding_rect.width(),
            ) || value_in_range(
                bounding_rect.min().x,
                self.min().x,
                self.min().x + self.width(),
            );

            let y_overlap = value_in_range(
                self.min().y,
                bounding_rect.min().y,
                bounding_rect.min().y + bounding_rect.height(),
            ) || value_in_range(
                bounding_rect.min().y,
                self.min().y,
                self.min().y + self.height(),
            );

            x_overlap && y_overlap
        }
    }
}

impl<T> Intersects<Polygon<T>> for Rect<T>
where
    T: Float,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        polygon.intersects(self)
    }
}

impl<T> Intersects<Rect<T>> for Polygon<T>
where
    T: Float,
{
    fn intersects(&self, bounding_rect: &Rect<T>) -> bool {
        let p = Polygon::new(
            LineString::from(vec![
                (bounding_rect.min().x, bounding_rect.min().y),
                (bounding_rect.min().x, bounding_rect.max().y),
                (bounding_rect.max().x, bounding_rect.max().y),
                (bounding_rect.max().x, bounding_rect.min().y),
                (bounding_rect.min().x, bounding_rect.min().y),
            ]),
            vec![],
        );
        self.intersects(&p)
    }
}

impl<T> Intersects<Polygon<T>> for Polygon<T>
where
    T: Float,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        // self intersects (or contains) any line in polygon
        self.intersects(polygon.exterior()) ||
            polygon.interiors().iter().any(|inner_line_string| self.intersects(inner_line_string)) ||
            // self is contained inside polygon
            polygon.intersects(self.exterior())
    }
}

#[cfg(test)]
mod test {
    use crate::algorithm::intersects::Intersects;
    use crate::{line_string, polygon, Coordinate, Line, LineString, Point, Polygon, Rect};

    /// Tests: intersection LineString and LineString
    #[test]
    fn empty_linestring1_test() {
        let linestring = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        assert!(!line_string![].intersects(&linestring));
    }
    #[test]
    fn empty_linestring2_test() {
        let linestring = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        assert!(!linestring.intersects(&LineString(Vec::new())));
    }
    #[test]
    fn empty_all_linestring_test() {
        let empty: LineString<f64> = line_string![];
        assert!(!empty.intersects(&empty));
    }
    #[test]
    fn intersect_linestring_test() {
        let ls1 = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        let ls2 = line_string![(x: 3., y: 4.), (x: 8., y: 4.)];
        assert!(ls1.intersects(&ls2));
    }
    #[test]
    fn parallel_linestrings_test() {
        let ls1 = line_string![(x: 3., y: 2.), (x: 7., y: 6.)];
        let ls2 = line_string![(x: 3., y: 1.), (x: 7., y: 5.)];
        assert!(!ls1.intersects(&ls2));
    }
    /// Tests: intersection LineString and Polygon
    #[test]
    fn linestring_in_polygon_test() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 5., y: 0.),
            (x: 5., y: 6.),
            (x: 0., y: 6.),
            (x: 0., y: 0.),
        ];
        let ls = line_string![(x: 2., y: 2.), (x: 3., y: 3.)];
        assert!(poly.intersects(&ls));
    }
    #[test]
    fn linestring_on_boundary_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            Vec::new(),
        );
        assert!(poly.intersects(&LineString::from(vec![(0., 0.), (5., 0.)])));
        assert!(poly.intersects(&LineString::from(vec![(5., 0.), (5., 6.)])));
        assert!(poly.intersects(&LineString::from(vec![(5., 6.), (0., 6.)])));
        assert!(poly.intersects(&LineString::from(vec![(0., 6.), (0., 0.)])));
    }
    #[test]
    fn intersect_linestring_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            Vec::new(),
        );
        assert!(poly.intersects(&LineString::from(vec![(2., 2.), (6., 6.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]),
            Vec::new(),
        );
        assert!(!poly.intersects(&LineString::from(vec![(7., 2.), (9., 4.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let e = LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]);
        let v = vec![LineString::from(vec![
            (1., 1.),
            (4., 1.),
            (4., 4.),
            (1., 4.),
            (1., 1.),
        ])];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects(&LineString::from(vec![(2., 2.), (3., 3.)])));
        assert!(poly.intersects(&LineString::from(vec![(2., 2.), (4., 4.)])));
    }
    #[test]
    fn linestring_traverse_polygon_test() {
        let e = LineString::from(vec![(0., 0.), (5., 0.), (5., 6.), (0., 6.), (0., 0.)]);
        let v = vec![LineString::from(vec![
            (1., 1.),
            (4., 1.),
            (4., 4.),
            (1., 4.),
            (1., 1.),
        ])];
        let poly = Polygon::new(e, v);
        assert!(poly.intersects(&LineString::from(vec![(2., 0.5), (2., 5.)])));
    }
    #[test]
    fn linestring_in_inner_with_2_inner_polygon_test() {
        //                                        (8,9)
        //     (2,8)                                |                                      (14,8)
        //      ------------------------------------|------------------------------------------
        //      |                                   |                                         |
        //      |     (4,7)            (6,7)        |                                         |
        //      |       ------------------          |                    (11,7)               |
        //      |                                   |                       |                 |
        //      |     (4,6)                (7,6)    |     (9,6)             |     (12,6)      |
        //      |       ----------------------      |       ----------------|---------        |
        //      |       |                    |      |       |               |        |        |
        //      |       |       (6,5)        |      |       |               |        |        |
        //      |       |        /           |      |       |               |        |        |
        //      |       |       /            |      |       |               |        |        |
        //      |       |     (5,4)          |      |       |               |        |        |
        //      |       |                    |      |       |               |        |        |
        //      |       ----------------------      |       ----------------|---------        |
        //      |     (4,3)                (7,3)    |     (9,3)             |     (12,3)      |
        //      |                                   |                    (11,2.5)             |
        //      |                                   |                                         |
        //      ------------------------------------|------------------------------------------
        //    (2,2)                                 |                                      (14,2)
        //                                        (8,1)
        //
        let e = LineString::from(vec![(2., 2.), (14., 2.), (14., 8.), (2., 8.), (2., 2.)]);
        let v = vec![
            LineString::from(vec![(4., 3.), (7., 3.), (7., 6.), (4., 6.), (4., 3.)]),
            LineString::from(vec![(9., 3.), (12., 3.), (12., 6.), (9., 6.), (9., 3.)]),
        ];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects(&LineString::from(vec![(5., 4.), (6., 5.)])));
        assert!(poly.intersects(&LineString::from(vec![(11., 2.5), (11., 7.)])));
        assert!(poly.intersects(&LineString::from(vec![(4., 7.), (6., 7.)])));
        assert!(poly.intersects(&LineString::from(vec![(8., 1.), (8., 9.)])));
    }
    #[test]
    fn polygons_do_not_intersect() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (3., 3.), (3., 5.), (1., 5.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![
                (10., 30.),
                (30., 30.),
                (30., 50.),
                (10., 50.),
                (10., 30.),
            ]),
            Vec::new(),
        );

        assert!(!p1.intersects(&p2));
        assert!(!p2.intersects(&p1));
    }
    #[test]
    fn polygons_overlap() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (3., 3.), (3., 5.), (1., 5.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![(2., 3.), (4., 3.), (4., 7.), (2., 7.), (2., 3.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygon_contained() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (4., 3.), (4., 6.), (1., 6.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![(2., 4.), (3., 4.), (3., 5.), (2., 5.), (2., 4.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygons_conincident() {
        let p1 = Polygon::new(
            LineString::from(vec![(1., 3.), (4., 3.), (4., 6.), (1., 6.), (1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString::from(vec![(1., 3.), (4., 3.), (4., 6.), (1., 6.), (1., 3.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygon_intersects_bounding_rect_test() {
        // Polygon poly =
        //
        // (0,8)               (12,8)
        //  ┌──────────────────────┐
        //  │         (7,7) (11,7) │
        //  │             ┌──────┐ │
        //  │             │      │ │
        //  │             │(hole)│ │
        //  │             │      │ │
        //  │             │      │ │
        //  │             └──────┘ │
        //  │         (7,4) (11,4) │
        //  │                      │
        //  │                      │
        //  │                      │
        //  │                      │
        //  │                      │
        //  └──────────────────────┘
        // (0,0)               (12,0)
        let poly = Polygon::new(
            LineString::from(vec![(0., 0.), (12., 0.), (12., 8.), (0., 8.), (0., 0.)]),
            vec![LineString::from(vec![
                (7., 4.),
                (11., 4.),
                (11., 7.),
                (7., 7.),
                (7., 4.),
            ])],
        );
        let b1 = Rect::new(Coordinate { x: 11., y: 1. }, Coordinate { x: 13., y: 2. });
        let b2 = Rect::new(Coordinate { x: 2., y: 2. }, Coordinate { x: 8., y: 5. });
        let b3 = Rect::new(Coordinate { x: 8., y: 5. }, Coordinate { x: 10., y: 6. });
        let b4 = Rect::new(Coordinate { x: 1., y: 1. }, Coordinate { x: 3., y: 3. });
        // overlaps
        assert!(poly.intersects(&b1));
        // contained in exterior, overlaps with hole
        assert!(poly.intersects(&b2));
        // completely contained in the hole
        assert!(!poly.intersects(&b3));
        // completely contained in the polygon
        assert!(poly.intersects(&b4));
        // conversely,
        assert!(b1.intersects(&poly));
        assert!(b2.intersects(&poly));
        assert!(!b3.intersects(&poly));
        assert!(b4.intersects(&poly));
    }
    #[test]
    fn bounding_rect_test() {
        let bounding_rect_xl = Rect::new(
            Coordinate { x: -100., y: -200. },
            Coordinate { x: 100., y: 200. },
        );
        let bounding_rect_sm = Rect::new(
            Coordinate { x: -10., y: -20. },
            Coordinate { x: 10., y: 20. },
        );
        let bounding_rect_s2 =
            Rect::new(Coordinate { x: 0., y: 0. }, Coordinate { x: 20., y: 30. });
        // confirmed using GEOS
        assert_eq!(true, bounding_rect_xl.intersects(&bounding_rect_sm));
        assert_eq!(false, bounding_rect_sm.intersects(&bounding_rect_xl));
        assert_eq!(true, bounding_rect_sm.intersects(&bounding_rect_s2));
        assert_eq!(true, bounding_rect_s2.intersects(&bounding_rect_sm));
    }
    #[test]
    fn point_intersects_line_test() {
        let p0 = Point::new(2., 4.);
        // vertical line
        let line1 = Line::from([(2., 0.), (2., 5.)]);
        // point on line, but outside line segment
        let line2 = Line::from([(0., 6.), (1.5, 4.5)]);
        // point on line
        let line3 = Line::from([(0., 6.), (3., 3.)]);
        // point above line with positive slope
        let line4 = Line::from([(1., 2.), (5., 3.)]);
        // point below line with positive slope
        let line5 = Line::from([(1., 5.), (5., 6.)]);
        // point above line with negative slope
        let line6 = Line::from([(1., 2.), (5., -3.)]);
        // point below line with negative slope
        let line7 = Line::from([(1., 6.), (5., 5.)]);
        assert!(line1.intersects(&p0));
        assert!(p0.intersects(&line1));
        assert!(!line2.intersects(&p0));
        assert!(!p0.intersects(&line2));
        assert!(line3.intersects(&p0));
        assert!(p0.intersects(&line3));
        assert!(!line4.intersects(&p0));
        assert!(!p0.intersects(&line4));
        assert!(!line5.intersects(&p0));
        assert!(!p0.intersects(&line5));
        assert!(!line6.intersects(&p0));
        assert!(!p0.intersects(&line6));
        assert!(!line7.intersects(&p0));
        assert!(!p0.intersects(&line7));
    }
    #[test]
    fn line_intersects_line_test() {
        let line0 = Line::from([(0., 0.), (3., 4.)]);
        let line1 = Line::from([(2., 0.), (2., 5.)]);
        let line2 = Line::from([(0., 7.), (5., 4.)]);
        let line3 = Line::from([(0., 0.), (-3., -4.)]);
        assert!(line0.intersects(&line0));
        assert!(line0.intersects(&line1));
        assert!(!line0.intersects(&line2));
        assert!(line0.intersects(&line3));

        assert!(line1.intersects(&line0));
        assert!(line1.intersects(&line1));
        assert!(!line1.intersects(&line2));
        assert!(!line1.intersects(&line3));

        assert!(!line2.intersects(&line0));
        assert!(!line2.intersects(&line1));
        assert!(line2.intersects(&line2));
        assert!(!line1.intersects(&line3));
    }
    #[test]
    fn line_intersects_linestring_test() {
        let line0 = Line::from([(0., 0.), (3., 4.)]);
        let linestring0 = LineString::from(vec![(0., 1.), (1., 0.), (2., 0.)]);
        let linestring1 = LineString::from(vec![(0.5, 0.2), (1., 0.), (2., 0.)]);
        assert!(line0.intersects(&linestring0));
        assert!(!line0.intersects(&linestring1));
        assert!(linestring0.intersects(&line0));
        assert!(!linestring1.intersects(&line0));
    }
    #[test]
    fn line_intersects_polygon_test() {
        let line0 = Line::from([(0.5, 0.5), (2., 1.)]);
        let poly0 = Polygon::new(
            LineString::from(vec![(0., 0.), (1., 2.), (1., 0.), (0., 0.)]),
            vec![],
        );
        let poly1 = Polygon::new(
            LineString::from(vec![(1., -1.), (2., -1.), (2., -2.), (1., -1.)]),
            vec![],
        );
        // line contained in the hole
        let poly2 = Polygon::new(
            LineString::from(vec![(-1., -1.), (-1., 10.), (10., -1.), (-1., -1.)]),
            vec![LineString::from(vec![
                (0., 0.),
                (3., 4.),
                (3., 0.),
                (0., 0.),
            ])],
        );
        assert!(line0.intersects(&poly0));
        assert!(poly0.intersects(&line0));

        assert!(!line0.intersects(&poly1));
        assert!(!poly1.intersects(&line0));

        assert!(!line0.intersects(&poly2));
        assert!(!poly2.intersects(&line0));
    }
    #[test]
    // See https://github.com/georust/geo/issues/419
    fn rect_test_419() {
        let a = Rect::new(
            Coordinate {
                x: 9.228515625,
                y: 46.83013364044739,
            },
            Coordinate {
                x: 9.2724609375,
                y: 46.86019101567026,
            },
        );
        let b = Rect::new(
            Coordinate {
                x: 9.17953,
                y: 46.82018,
            },
            Coordinate {
                x: 9.26309,
                y: 46.88099,
            },
        );
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }
}
