use num_traits::{Float, ToPrimitive};

use algorithm::euclidean_distance::EuclideanDistance;
use algorithm::intersects::Intersects;
use {Bbox, CoordinateType, Line, LineString, MultiPolygon, Point, Polygon, COORD_PRECISION};

///  Checks if the geometry A is completely inside the B geometry
pub trait Contains<Rhs = Self> {
    /// Checks if `rhs` is completely contained within `self`.
    ///
    /// ```
    /// use geo::{Coordinate, Point, LineString, Polygon};
    /// use geo::algorithm::contains::Contains;
    ///
    /// let p = |x, y| Point(Coordinate { x: x, y: y });
    /// let v = Vec::new();
    /// let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
    /// let poly = Polygon::new(linestring.clone(), v);
    ///
    /// //Point in Point
    /// assert!(p(2., 0.).contains(&p(2., 0.)));
    ///
    /// //Point in Linestring
    /// assert!(linestring.contains(&p(2., 0.)));
    ///
    /// //Point in Polygon
    /// assert!(poly.contains(&p(1., 1.)));
    ///
    /// ```
    ///
    fn contains(&self, rhs: &Rhs) -> bool;
}

impl<T> Contains<Point<T>> for Point<T>
where
    T: Float + ToPrimitive,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.euclidean_distance(p).to_f32().unwrap() < COORD_PRECISION
    }
}

impl<T> Contains<Point<T>> for LineString<T>
where
    T: Float,
{
    fn contains(&self, p: &Point<T>) -> bool {
        // LineString without points
        if self.0.is_empty() {
            return false;
        }
        // LineString with one point equal p
        if self.0.len() == 1 {
            return self.0[0].contains(p);
        }
        // check if point is a vertex
        if self.0.contains(p) {
            return true;
        }
        for line in self.lines() {
            if ((line.start.y() == line.end.y()) && (line.start.y() == p.y())
                && (p.x() > line.start.x().min(line.end.x()))
                && (p.x() < line.start.x().max(line.end.x())))
                || ((line.start.x() == line.end.x()) && (line.start.x() == p.x())
                    && (p.y() > line.start.y().min(line.end.y()))
                    && (p.y() < line.start.y().max(line.end.y())))
            {
                return true;
            }
        }
        false
    }
}

impl<T> Contains<Point<T>> for Line<T>
where
    T: Float,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.intersects(p)
    }
}

impl<T> Contains<Line<T>> for Line<T>
where
    T: Float,
{
    fn contains(&self, line: &Line<T>) -> bool {
        self.contains(&line.start) & self.contains(&line.end)
    }
}

impl<T> Contains<LineString<T>> for Line<T>
where
    T: Float,
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        linestring.0.iter().all(|pt| self.contains(pt))
    }
}

impl<T> Contains<Line<T>> for LineString<T>
where
    T: Float,
{
    fn contains(&self, line: &Line<T>) -> bool {
        let (p0, p1) = (line.start, line.end);
        let mut look_for: Option<Point<T>> = None;
        for segment in self.lines() {
            if look_for.is_none() {
                // If segment contains an endpoint of line, we mark the other endpoint as the
                // one we are looking for.
                if segment.contains(&p0) {
                    look_for = Some(p1);
                } else if segment.contains(&p1) {
                    look_for = Some(p0);
                }
            }
            if let Some(p) = look_for {
                // If we are looking for an endpoint, we need to either find it, or show that we
                // should continue to look for it
                if segment.contains(&p) {
                    // If the segment contains the endpoint we are looking for we are done
                    return true;
                } else if !line.contains(&segment.end) {
                    // If not, and the end of the segment is not on the line, we should stop
                    // looking
                    look_for = None
                }
            }
        }
        false
    }
}

/// The position of a `Point` with respect to a `LineString`
#[derive(PartialEq, Clone, Debug)]
pub(crate) enum PositionPoint {
    OnBoundary,
    Inside,
    Outside,
}

/// Calculate the position of `Point` p relative to a linestring
pub(crate) fn get_position<T>(p: &Point<T>, linestring: &LineString<T>) -> PositionPoint
where
    T: Float,
{
    // See: http://www.ecse.rpi.edu/Homepages/wrf/Research/Short_Notes/pnpoly.html
    //      http://geospatialpython.com/search
    //         ?updated-min=2011-01-01T00:00:00-06:00&updated-max=2012-01-01T00:00:00-06:00&max-results=19

    // LineString without points
    if linestring.0.is_empty() {
        return PositionPoint::Outside;
    }
    // Point is on linestring
    if linestring.contains(p) {
        return PositionPoint::OnBoundary;
    }

    let mut xints = T::zero();
    let mut crossings = 0;
    for line in linestring.lines() {
        if p.y() > line.start.y().min(line.end.y()) && p.y() <= line.start.y().max(line.end.y())
            && p.x() <= line.start.x().max(line.end.x())
        {
            if line.start.y() != line.end.y() {
                xints = (p.y() - line.start.y()) * (line.end.x() - line.start.x())
                    / (line.end.y() - line.start.y()) + line.start.x();
            }
            if (line.start.x() == line.end.x()) || (p.x() <= xints) {
                crossings += 1;
            }
        }
    }
    if crossings % 2 == 1 {
        PositionPoint::Inside
    } else {
        PositionPoint::Outside
    }
}

impl<T> Contains<Point<T>> for Polygon<T>
where
    T: Float,
{
    fn contains(&self, p: &Point<T>) -> bool {
        match get_position(p, &self.exterior) {
            PositionPoint::OnBoundary | PositionPoint::Outside => false,
            _ => self.interiors
                .iter()
                .all(|ls| get_position(p, ls) == PositionPoint::Outside),
        }
    }
}

impl<T> Contains<Point<T>> for MultiPolygon<T>
where
    T: Float,
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.0.iter().any(|poly| poly.contains(p))
    }
}

impl<T> Contains<Line<T>> for Polygon<T>
where
    T: Float,
{
    fn contains(&self, line: &Line<T>) -> bool {
        // both endpoints are contained in the polygon and the line
        // does NOT intersect the exterior or any of the interior boundaries
        self.contains(&line.start) && self.contains(&line.end) && !self.exterior.intersects(line)
            && !self.interiors.iter().any(|inner| inner.intersects(line))
    }
}

impl<T> Contains<Polygon<T>> for Polygon<T>
where
    T: Float,
{
    fn contains(&self, poly: &Polygon<T>) -> bool {
        // decompose poly's exterior ring into Lines, and check each for containment
        poly.exterior.lines().all(|line| self.contains(&line))
    }
}

impl<T> Contains<LineString<T>> for Polygon<T>
where
    T: Float,
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        // All LineString points must be inside the Polygon
        if linestring.0.iter().all(|point| self.contains(point)) {
            // The Polygon interior is allowed to intersect with the LineString
            // but the Polygon's rings are not
            !self.interiors
                .iter()
                .any(|ring| ring.intersects(linestring))
        } else {
            false
        }
    }
}

impl<T> Contains<Point<T>> for Bbox<T>
where
    T: CoordinateType,
{
    fn contains(&self, p: &Point<T>) -> bool {
        p.x() >= self.xmin && p.x() <= self.xmax && p.y() >= self.ymin && p.y() <= self.ymax
    }
}

impl<T> Contains<Bbox<T>> for Bbox<T>
where
    T: CoordinateType,
{
    fn contains(&self, bbox: &Bbox<T>) -> bool {
        // All points of LineString must be in the polygon ?
        self.xmin <= bbox.xmin && self.xmax >= bbox.xmax && self.ymin <= bbox.ymin
            && self.ymax >= bbox.ymax
    }
}

#[cfg(test)]
mod test {
    use algorithm::contains::Contains;
    use {Bbox, Coordinate, Line, LineString, MultiPolygon, Point, Polygon};
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
            ].into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (250., 310.),
                (150., 310.),
                (150., 280.),
                (250., 280.),
                (250., 310.),
            ].into(),
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
            ].into(),
            vec![],
        );
        let rect = Polygon::new(
            vec![
                (185., 237.),
                (220., 237.),
                (220., 220.),
                (185., 220.),
                (185., 237.),
            ].into(),
            vec![],
        );
        assert_eq!(v.contains(&rect), true);
    }
    #[test]
    // LineString is fully contained
    fn linestring_fully_contained_in_polygon() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(
            LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
            vec![],
        );
        let ls = LineString(vec![Point::new(3.0, 0.5), Point::new(3.0, 3.5)]);
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
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.)]);
        assert!(linestring.contains(&p(2., 2.)));
    }
    #[test]
    fn linestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.)]);
        assert!(linestring.contains(&p(1., 0.)));
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
        let linestring = LineString(vec![Point::new(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(3., 1.)));
    }
    #[test]
    fn polygon_with_one_point_is_vertex_test() {
        let linestring = LineString(vec![Point::new(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_point_on_boundary_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&p(1., 0.)));
        assert!(!poly.contains(&p(2., 1.)));
        assert!(!poly.contains(&p(1., 2.)));
        assert!(!poly.contains(&p(0., 1.)));
    }
    #[test]
    fn point_in_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains(&p(1., 1.)));
    }
    #[test]
    fn point_out_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&p(2.1, 1.)));
        assert!(!poly.contains(&p(1., 2.1)));
        assert!(!poly.contains(&p(2.1, 2.1)));
    }
    #[test]
    fn point_polygon_with_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let inner_linestring = LineString(vec![
            p(0.5, 0.5),
            p(1.5, 0.5),
            p(1.5, 1.5),
            p(0.0, 1.5),
            p(0.0, 0.0),
        ]);
        let poly = Polygon::new(linestring, vec![inner_linestring]);
        assert!(poly.contains(&p(0.25, 0.25)));
        assert!(!poly.contains(&p(1., 1.)));
        assert!(!poly.contains(&p(1.5, 1.5)));
        assert!(!poly.contains(&p(1.5, 1.)));
    }
    /// Tests: Point in MultiPolygon
    #[test]
    fn empty_multipolygon_test() {
        let multipoly = MultiPolygon(Vec::new());
        assert!(!multipoly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly1 = Polygon::new(
            LineString(vec![p(0., 0.), p(1., 0.), p(1., 1.), p(0., 1.), p(0., 0.)]),
            Vec::new(),
        );
        let poly2 = Polygon::new(
            LineString(vec![p(2., 0.), p(3., 0.), p(3., 1.), p(2., 1.), p(2., 0.)]),
            Vec::new(),
        );
        let multipoly = MultiPolygon(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains(&Point::new(1.5, 0.5)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_and_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly1 = Polygon::new(
            LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
            vec![LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 1.)])],
        );
        let poly2 = Polygon::new(
            LineString(vec![
                p(9., 0.),
                p(14., 0.),
                p(14., 4.),
                p(9., 4.),
                p(9., 0.),
            ]),
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
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring.clone(), Vec::new());
        assert!(!poly.contains(&linestring.clone()));
        assert!(!poly.contains(&LineString(vec![p(0., 0.), p(2., 0.)])));
        assert!(!poly.contains(&LineString(vec![p(2., 0.), p(2., 2.)])));
        assert!(!poly.contains(&LineString(vec![p(0., 2.), p(0., 0.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains(&LineString(vec![p(1., 1.), p(3., 0.)])));
        assert!(!poly.contains(&LineString(vec![p(3., 0.), p(5., 2.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });

        let poly = Polygon::new(
            LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
            vec![LineString(vec![
                p(1., 1.),
                p(4., 1.),
                p(4., 4.),
                p(1., 4.),
                p(1., 1.),
            ])],
        );
        assert!(!poly.contains(&LineString(vec![p(2., 2.), p(3., 3.)])));
        assert!(!poly.contains(&LineString(vec![p(2., 2.), p(2., 5.)])));
        assert!(!poly.contains(&LineString(vec![p(3., 0.5), p(3., 5.)])));
    }
    #[test]
    fn bbox_in_inner_bbox_test() {
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
        assert_eq!(true, bbox_xl.contains(&bbox_sm));
        assert_eq!(false, bbox_sm.contains(&bbox_xl));
    }
    #[test]
    fn point_in_line_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p0 = p(2., 4.);
        // vertical line
        let line1 = Line::new(p(2., 0.), p(2., 5.));
        // point on line, but outside line segment
        let line2 = Line::new(p(0., 6.), p(1.5, 4.5));
        // point on line
        let line3 = Line::new(p(0., 6.), p(3., 3.));
        assert!(line1.contains(&p0));
        assert!(!line2.contains(&p0));
        assert!(line3.contains(&p0));
    }
    #[test]
    fn line_in_line_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let line0 = Line::new(p(0., 1.), p(3., 4.));
        // first point on line0, second not
        let line1 = Line::new(p(1., 2.), p(2., 2.));
        // co-linear, but extends past the end of line0
        let line2 = Line::new(p(1., 2.), p(4., 5.));
        // contained in line0
        let line3 = Line::new(p(1., 2.), p(3., 4.));
        assert!(!line0.contains(&line1));
        assert!(!line0.contains(&line2));
        assert!(line0.contains(&line3));
    }
    #[test]
    fn linestring_in_line_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let line = Line::new(p(0., 1.), p(3., 4.));
        // linestring0 in line
        let linestring0 = LineString(vec![p(0.1, 1.1), p(1., 2.), p(1.5, 2.5)]);
        // linestring1 starts and ends in line, but wanders in the middle
        let linestring1 = LineString(vec![p(0.1, 1.1), p(2., 2.), p(1.5, 2.5)]);
        // linestring2 is co-linear, but extends beyond line
        let linestring2 = LineString(vec![p(0.1, 1.1), p(1., 2.), p(4., 5.)]);
        // no part of linestring3 is contained in line
        let linestring3 = LineString(vec![p(1.1, 1.1), p(2., 2.), p(2.5, 2.5)]);
        assert!(line.contains(&linestring0));
        assert!(!line.contains(&linestring1));
        assert!(!line.contains(&linestring2));
        assert!(!line.contains(&linestring3));
    }
    #[test]
    fn line_in_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let line = Line::new(p(0., 1.), p(3., 4.));
        let linestring0 = LineString(vec![
            p(-1., 0.),
            p(5., 0.),
            p(5., 5.),
            p(0., 5.),
            p(-1., 0.),
        ]);
        let poly0 = Polygon::new(linestring0, Vec::new());
        let linestring1 = LineString(vec![p(0., 0.), p(0., 2.), p(2., 2.), p(2., 0.), p(0., 0.)]);
        let poly1 = Polygon::new(linestring1, Vec::new());
        assert!(poly0.contains(&line));
        assert!(!poly1.contains(&line));
    }
    #[test]
    fn line_in_linestring_test() {
        let line0 = Line::new(Point::new(1., 1.), Point::new(2., 2.));
        // line0 is completely contained in the second segment
        let linestring0 = LineString(vec![
            Point::new(0., 0.5),
            Point::new(0.5, 0.5),
            Point::new(3., 3.),
        ]);
        // line0 is contained in the last three segments
        let linestring1 = LineString(vec![
            Point::new(0., 0.5),
            Point::new(0.5, 0.5),
            Point::new(1.2, 1.2),
            Point::new(1.5, 1.5),
            Point::new(3., 3.),
        ]);
        // line0 endpoints are contained in the linestring, but the fourth point is off the line
        let linestring2 = LineString(vec![
            Point::new(0., 0.5),
            Point::new(0.5, 0.5),
            Point::new(1.2, 1.2),
            Point::new(1.5, 0.),
            Point::new(2., 2.),
            Point::new(3., 3.),
        ]);
        assert!(linestring0.contains(&line0));
        assert!(linestring1.contains(&line0));
        assert!(!linestring2.contains(&line0));
    }

    #[test]
    fn integer_bboxs() {
        let p: Point<i32> = Point::new(10, 20);
        let bbox: Bbox<i32> = Bbox {
            xmin: 0,
            ymin: 0,
            xmax: 100,
            ymax: 100,
        };
        assert!(bbox.contains(&p));
        assert!(!bbox.contains(&Point::new(-10, -10)));

        let smaller_bbox: Bbox<i32> = Bbox {
            xmin: 10,
            ymin: 10,
            xmax: 20,
            ymax: 20,
        };
        assert!(bbox.contains(&smaller_bbox));
    }
}
