use algorithm::contains::Contains;
use num_traits::Float;
use {Bbox, Line, LineString, Point, Polygon};

/// Checks if the geometry A intersects the geometry B.

pub trait Intersects<Rhs = Self> {
    /// Checks if the geometry A intersects the geometry B.
    ///
    /// ```
    /// use geo::{Coordinate, Point, LineString};
    /// use geo::algorithm::intersects::Intersects;
    ///
    /// let p = |x, y| Point(Coordinate { x: x, y: y });
    /// let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
    ///
    /// assert!(linestring.intersects(&LineString(vec![p(3., 4.), p(8., 4.)])));
    /// assert!(!linestring.intersects(&LineString(vec![p(9., 2.), p(11., 5.)])));
    ///
    /// ```
    ///
    fn intersects(&self, rhs: &Rhs) -> bool;
}

impl<T> Intersects<Point<T>> for Line<T>
where
    T: Float,
{
    fn intersects(&self, p: &Point<T>) -> bool {
        let dx = self.end.x() - self.start.x();
        let dy = self.end.y() - self.start.y();
        let tx = if dx == T::zero() {
            None
        } else {
            Some((p.x() - self.start.x()) / dx)
        };
        let ty = if dy == T::zero() {
            None
        } else {
            Some((p.y() - self.start.y()) / dy)
        };
        match (tx, ty) {
            (None, None) => {
                // Degenerate line
                *p == self.start
            }
            (Some(t), None) => {
                // Horizontal line
                p.y() == self.start.y() && T::zero() <= t && t <= T::one()
            }
            (None, Some(t)) => {
                // Vertical line
                p.x() == self.start.x() && T::zero() <= t && t <= T::one()
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
        let (x1, y1, x2, y2) = (self.start.x(), self.start.y(), self.end.x(), self.end.y());
        let (x3, y3, x4, y4) = (line.start.x(), line.start.y(), line.end.x(), line.end.y());
        let a1 = x2 - x1;
        let a2 = y2 - y1;
        let b1 = x3 - x4; // == -(x4 - x3)
        let b2 = y3 - y4; // == -(y4 - y3)
        let c1 = x3 - x1;
        let c2 = y3 - y1;

        let d = a1 * b2 - a2 * b1;
        if d == T::zero() {
            // lines are parallel
            // return true iff at least one endpoint intersects the other line
            self.start.intersects(line) || self.end.intersects(line) || line.start.intersects(self)
                || line.end.intersects(self)
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
        p.exterior.intersects(self) || p.interiors.iter().any(|inner| inner.intersects(self))
            || p.contains(&self.start) || p.contains(&self.end)
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
                let u_b = (b.end.y() - b.start.y()) * (a.end.x() - a.start.x())
                    - (b.end.x() - b.start.x()) * (a.end.y() - a.start.y());
                if u_b == T::zero() {
                    continue;
                }
                let ua_t = (b.end.x() - b.start.x()) * (a.start.y() - b.start.y())
                    - (b.end.y() - b.start.y()) * (a.start.x() - b.start.x());
                let ub_t = (a.end.x() - a.start.x()) * (a.start.y() - b.start.y())
                    - (a.end.y() - a.start.y()) * (a.start.x() - b.start.x());
                let u_a = ua_t / u_b;
                let u_b = ub_t / u_b;
                if (T::zero() <= u_a) && (u_a <= T::one()) && (T::zero() <= u_b)
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
        if self.exterior.intersects(linestring)
            || self.interiors
                .iter()
                .any(|inner| inner.intersects(linestring))
        {
            true
        } else {
            // or if it's contained in the polygon
            linestring.0.iter().any(|point| self.contains(point))
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

impl<T> Intersects<Bbox<T>> for Bbox<T>
where
    T: Float,
{
    fn intersects(&self, bbox: &Bbox<T>) -> bool {
        // line intersects inner or outer polygon edge
        if bbox.contains(self) {
            false
        } else {
            (self.xmin >= bbox.xmin && self.xmin <= bbox.xmax
                || self.xmax >= bbox.xmin && self.xmax <= bbox.xmax)
                && (self.ymin >= bbox.ymin && self.ymin <= bbox.ymax
                    || self.ymax >= bbox.ymin && self.ymax <= bbox.ymax)
        }
    }
}

impl<T> Intersects<Polygon<T>> for Bbox<T>
where
    T: Float,
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        polygon.intersects(self)
    }
}

impl<T> Intersects<Bbox<T>> for Polygon<T>
where
    T: Float,
{
    fn intersects(&self, bbox: &Bbox<T>) -> bool {
        let p = Polygon::new(
            LineString(vec![
                Point::new(bbox.xmin, bbox.ymin),
                Point::new(bbox.xmin, bbox.ymax),
                Point::new(bbox.xmax, bbox.ymax),
                Point::new(bbox.xmax, bbox.ymin),
                Point::new(bbox.xmin, bbox.ymin),
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
        self.intersects(&polygon.exterior) ||
            polygon.interiors.iter().any(|inner_line_string| self.intersects(inner_line_string)) ||
            // self is contained inside polygon
            polygon.intersects(&self.exterior)
    }
}

#[cfg(test)]
mod test {
    use algorithm::intersects::Intersects;
    use {Bbox, Coordinate, Line, LineString, Point, Polygon};
    /// Tests: intersection LineString and LineString
    #[test]
    fn empty_linestring1_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(!LineString(Vec::new()).intersects(&linestring));
    }
    #[test]
    fn empty_linestring2_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(!linestring.intersects(&LineString(Vec::new())));
    }
    #[test]
    fn empty_all_linestring_test() {
        assert!(!LineString::<f64>(Vec::new()).intersects(&LineString(Vec::new())));
    }
    #[test]
    fn intersect_linestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(linestring.intersects(&LineString(vec![p(3., 4.), p(8., 4.)])));
    }
    #[test]
    fn parallel_linestrings_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(!linestring.intersects(&LineString(vec![p(3., 1.), p(7., 5.)])));
    }
    /// Tests: intersection LineString and Polygon
    #[test]
    fn linestring_in_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.intersects(&LineString(vec![p(2., 2.), p(3., 3.)])));
    }
    #[test]
    fn linestring_on_boundary_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(
            LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
            Vec::new(),
        );
        assert!(poly.intersects(&LineString(vec![p(0., 0.), p(5., 0.)])));
        assert!(poly.intersects(&LineString(vec![p(5., 0.), p(5., 6.)])));
        assert!(poly.intersects(&LineString(vec![p(5., 6.), p(0., 6.)])));
        assert!(poly.intersects(&LineString(vec![p(0., 6.), p(0., 0.)])));
    }
    #[test]
    fn intersect_linestring_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(
            LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
            Vec::new(),
        );
        assert!(poly.intersects(&LineString(vec![p(2., 2.), p(6., 6.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(
            LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
            Vec::new(),
        );
        assert!(!poly.intersects(&LineString(vec![p(7., 2.), p(9., 4.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let e = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let v = vec![LineString(vec![
            p(1., 1.),
            p(4., 1.),
            p(4., 4.),
            p(1., 4.),
            p(1., 1.),
        ])];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects(&LineString(vec![p(2., 2.), p(3., 3.)])));
        assert!(poly.intersects(&LineString(vec![p(2., 2.), p(4., 4.)])));
    }
    #[test]
    fn linestring_traverse_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let e = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let v = vec![LineString(vec![
            p(1., 1.),
            p(4., 1.),
            p(4., 4.),
            p(1., 4.),
            p(1., 1.),
        ])];
        let poly = Polygon::new(e, v);
        assert!(poly.intersects(&LineString(vec![p(2., 0.5), p(2., 5.)])));
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
        let p = |x, y| Point(Coordinate { x: x, y: y });

        let e = LineString(vec![
            p(2., 2.),
            p(14., 2.),
            p(14., 8.),
            p(2., 8.),
            p(2., 2.),
        ]);
        let v = vec![
            LineString(vec![p(4., 3.), p(7., 3.), p(7., 6.), p(4., 6.), p(4., 3.)]),
            LineString(vec![
                p(9., 3.),
                p(12., 3.),
                p(12., 6.),
                p(9., 6.),
                p(9., 3.),
            ]),
        ];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects(&LineString(vec![p(5., 4.), p(6., 5.)])));
        assert!(poly.intersects(&LineString(vec![p(11., 2.5), p(11., 7.)])));
        assert!(poly.intersects(&LineString(vec![p(4., 7.), p(6., 7.)])));
        assert!(poly.intersects(&LineString(vec![p(8., 1.), p(8., 9.)])));
    }
    #[test]
    fn polygons_do_not_intersect() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(
            LineString(vec![p(1., 3.), p(3., 3.), p(3., 5.), p(1., 5.), p(1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString(vec![
                p(10., 30.),
                p(30., 30.),
                p(30., 50.),
                p(10., 50.),
                p(10., 30.),
            ]),
            Vec::new(),
        );

        assert!(!p1.intersects(&p2));
        assert!(!p2.intersects(&p1));
    }
    #[test]
    fn polygons_overlap() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(
            LineString(vec![p(1., 3.), p(3., 3.), p(3., 5.), p(1., 5.), p(1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString(vec![p(2., 3.), p(4., 3.), p(4., 7.), p(2., 7.), p(2., 3.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygon_contained() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(
            LineString(vec![p(1., 3.), p(4., 3.), p(4., 6.), p(1., 6.), p(1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString(vec![p(2., 4.), p(3., 4.), p(3., 5.), p(2., 5.), p(2., 4.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygons_conincident() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(
            LineString(vec![p(1., 3.), p(4., 3.), p(4., 6.), p(1., 6.), p(1., 3.)]),
            Vec::new(),
        );
        let p2 = Polygon::new(
            LineString(vec![p(1., 3.), p(4., 3.), p(4., 6.), p(1., 6.), p(1., 3.)]),
            Vec::new(),
        );

        assert!(p1.intersects(&p2));
        assert!(p2.intersects(&p1));
    }
    #[test]
    fn polygon_intersects_bbox_test() {
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
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(
            LineString(vec![
                p(0., 0.),
                p(12., 0.),
                p(12., 8.),
                p(0., 8.),
                p(0., 0.),
            ]),
            vec![LineString(vec![
                p(7., 4.),
                p(11., 4.),
                p(11., 7.),
                p(7., 7.),
                p(7., 4.),
            ])],
        );
        let b1 = Bbox {
            xmin: 11.0,
            xmax: 13.0,
            ymin: 1.0,
            ymax: 2.0,
        };
        let b2 = Bbox {
            xmin: 2.0,
            xmax: 8.0,
            ymin: 2.0,
            ymax: 5.0,
        };
        let b3 = Bbox {
            xmin: 8.0,
            xmax: 10.0,
            ymin: 5.0,
            ymax: 6.0,
        };
        let b4 = Bbox {
            xmin: 1.0,
            xmax: 3.0,
            ymin: 1.0,
            ymax: 3.0,
        };
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
    fn bbox_test() {
        let bbox_xl = Bbox {
            xmin: -100.,
            xmax: 100.,
            ymin: -200.,
            ymax: 200.,
        };
        let bbox_sm = Bbox {
            xmin: -10.,
            xmax: 10.,
            ymin: -20.,
            ymax: 20.,
        };
        let bbox_s2 = Bbox {
            xmin: 0.,
            xmax: 20.,
            ymin: 0.,
            ymax: 30.,
        };
        assert_eq!(false, bbox_xl.intersects(&bbox_sm));
        assert_eq!(false, bbox_sm.intersects(&bbox_xl));
        assert_eq!(true, bbox_sm.intersects(&bbox_s2));
        assert_eq!(true, bbox_s2.intersects(&bbox_sm));
    }
    #[test]
    fn point_intersects_line_test() {
        let p0 = Point::new(2., 4.);
        // vertical line
        let line1 = Line::new(Point::new(2., 0.), Point::new(2., 5.));
        // point on line, but outside line segment
        let line2 = Line::new(Point::new(0., 6.), Point::new(1.5, 4.5));
        // point on line
        let line3 = Line::new(Point::new(0., 6.), Point::new(3., 3.));
        // point above line with positive slope
        let line4 = Line::new(Point::new(1., 2.), Point::new(5., 3.));
        // point below line with positive slope
        let line5 = Line::new(Point::new(1., 5.), Point::new(5., 6.));
        // point above line with negative slope
        let line6 = Line::new(Point::new(1., 2.), Point::new(5., -3.));
        // point below line with negative slope
        let line7 = Line::new(Point::new(1., 6.), Point::new(5., 5.));
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
        let line0 = Line::new(Point::new(0., 0.), Point::new(3., 4.));
        let line1 = Line::new(Point::new(2., 0.), Point::new(2., 5.));
        let line2 = Line::new(Point::new(0., 7.), Point::new(5., 4.));
        let line3 = Line::new(Point::new(0., 0.), Point::new(-3., -4.));
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
        let line0 = Line::new(Point::new(0., 0.), Point::new(3., 4.));
        let linestring0 = LineString(vec![
            Point::new(0., 1.),
            Point::new(1., 0.),
            Point::new(2., 0.),
        ]);
        let linestring1 = LineString(vec![
            Point::new(0.5, 0.2),
            Point::new(1., 0.),
            Point::new(2., 0.),
        ]);
        assert!(line0.intersects(&linestring0));
        assert!(!line0.intersects(&linestring1));
        assert!(linestring0.intersects(&line0));
        assert!(!linestring1.intersects(&line0));
    }
    #[test]
    fn line_intersects_polygon_test() {
        let line0 = Line::new(Point::new(0.5, 0.5), Point::new(2., 1.));
        let poly0 = Polygon::new(
            LineString(vec![
                Point::new(0., 0.),
                Point::new(1., 2.),
                Point::new(1., 0.),
                Point::new(0., 0.),
            ]),
            vec![],
        );
        let poly1 = Polygon::new(
            LineString(vec![
                Point::new(1., -1.),
                Point::new(2., -1.),
                Point::new(2., -2.),
                Point::new(1., -1.),
            ]),
            vec![],
        );
        // line contained in the hole
        let poly2 = Polygon::new(
            LineString(vec![
                Point::new(-1., -1.),
                Point::new(-1., 10.),
                Point::new(10., -1.),
                Point::new(-1., -1.),
            ]),
            vec![LineString(vec![
                Point::new(0., 0.),
                Point::new(3., 4.),
                Point::new(3., 0.),
                Point::new(0., 0.),
            ])],
        );
        assert!(line0.intersects(&poly0));
        assert!(poly0.intersects(&line0));

        assert!(!line0.intersects(&poly1));
        assert!(!poly1.intersects(&line0));

        assert!(!line0.intersects(&poly2));
        assert!(!poly2.intersects(&line0));
    }
}
