use num_traits::{Float, ToPrimitive};

use types::{COORD_PRECISION, Point, Line, LineString, Polygon, MultiPolygon, Bbox};
use algorithm::boundingbox::BoundingBox;
use algorithm::intersects::Intersects;
use algorithm::distance::Distance;

///  Checks if the geometry A is completely inside the B geometry.

pub trait Contains<Rhs = Self> {
    ///  Checks if the geometry A is completely inside the B geometry.
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
    where T: Float + ToPrimitive
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.distance(p).to_f32().unwrap() < COORD_PRECISION
    }
}

impl<T> Contains<Point<T>> for LineString<T>
    where T: Float
{
    fn contains(&self, p: &Point<T>) -> bool {
        let vect = &self.0;
        // LineString without points
        if vect.is_empty() {
            return false;
        }
        // LineString with one point equal p
        if vect.len() == 1 {
            return vect[0].contains(p);
        }
        // check if point is a vertex
        if vect.contains(p) {
            return true;
        }
        for ps in vect.windows(2) {
            if ((ps[0].y() == ps[1].y()) && (ps[0].y() == p.y()) &&
                (p.x() > ps[0].x().min(ps[1].x())) &&
                (p.x() < ps[0].x().max(ps[1].x()))) ||
               ((ps[0].x() == ps[1].x()) && (ps[0].x() == p.x()) &&
                (p.y() > ps[0].y().min(ps[1].y())) &&
                (p.y() < ps[0].y().max(ps[1].y()))) {
                return true;
            }
        }
        false
    }
}

impl<T> Contains<Point<T>> for Line<T>
    where T: Float
{
    fn contains(&self, p: &Point<T>) -> bool {
        // if the point is not in the bounding box, it's not on the line
        if !self.bbox().map_or(false, |b| b.contains(p)) {
            return false;
        }
        let (a, b) = self.0;
        // handle the special case where the line is vertical
        if a.x() == b.x() {
            return (p.x() - a.x()).to_f32().unwrap() <= COORD_PRECISION;
        }
        // solve a linear equation
        let slope = (a.y() - b.y()) / (a.x() - b.x());
        let intercept = a.y() - slope * a.x();
        (p.y() - (slope * p.x() + intercept)).abs()
                                             .to_f32()
                                             .unwrap() <= COORD_PRECISION
    }
}

#[derive(PartialEq, Clone, Debug)]
enum PositionPoint {
    OnBoundary,
    Inside,
    Outside,
}

fn get_position<T>(p: &Point<T>, linestring: &LineString<T>) -> PositionPoint
    where T: Float
{
    // See: http://www.ecse.rpi.edu/Homepages/wrf/Research/Short_Notes/pnpoly.html
    //      http://geospatialpython.com/search
    //         ?updated-min=2011-01-01T00:00:00-06:00&updated-max=2012-01-01T00:00:00-06:00&max-results=19
    // Return the position of the point relative to a linestring

    let vect = &linestring.0;
    // LineString without points
    if vect.is_empty() {
        return PositionPoint::Outside;
    }
    // Point is on linestring
    if linestring.contains(p) {
        return PositionPoint::OnBoundary;
    }

    let mut xints = T::zero();
    let mut crossings = 0;
    for ps in vect.windows(2) {
        if p.y() > ps[0].y().min(ps[1].y()) {
            if p.y() <= ps[0].y().max(ps[1].y()) {
                if p.x() <= ps[0].x().max(ps[1].x()) {
                    if ps[0].y() != ps[1].y() {
                        xints = (p.y() - ps[0].y()) * (ps[1].x() - ps[0].x()) /
                                (ps[1].y() - ps[0].y()) + ps[0].x();
                    }
                    if (ps[0].x() == ps[1].x()) || (p.x() <= xints) {
                        crossings += 1;
                    }
                }
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
    where T: Float
{
    fn contains(&self, p: &Point<T>) -> bool {
        match get_position(p, &self.exterior) {
            PositionPoint::OnBoundary => false,
            PositionPoint::Outside => false,
            _ => self.interiors.iter().all(|ls| get_position(p, ls) == PositionPoint::Outside),
        }
    }
}

impl<T> Contains<Point<T>> for MultiPolygon<T>
    where T: Float
{
    fn contains(&self, p: &Point<T>) -> bool {
        self.0.iter().any(|poly| poly.contains(p))
    }
}

impl<T> Contains<LineString<T>> for Polygon<T>
    where T: Float
{
    fn contains(&self, linestring: &LineString<T>) -> bool {
        // All points of LineString must be in the polygon ?
        if linestring.0.iter().all(|point| self.contains(point)) {
            !self.intersects(linestring)
        } else {
            false
        }
    }
}


impl<T> Contains<Point<T>> for Bbox<T>
    where T: Float
{
    fn contains(&self, p: &Point<T>) -> bool {
        p.x() >= self.xmin && p.x() <= self.xmax && p.y() >= self.ymin && p.y() <= self.ymax
    }
}

impl<T> Contains<Bbox<T>> for Bbox<T>
    where T: Float
{
    fn contains(&self, bbox: &Bbox<T>) -> bool {
        // All points of LineString must be in the polygon ?
        self.xmin <= bbox.xmin && self.xmax >= bbox.xmax && self.ymin <= bbox.ymin && self.ymax >= bbox.ymax
    }
}


#[cfg(test)]
mod test {
    use types::{Coordinate, Point, Line, LineString, Polygon, MultiPolygon, Bbox};
    use algorithm::contains::Contains;
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
        let inner_linestring = LineString(vec![p(0.5, 0.5),
                                               p(1.5, 0.5),
                                               p(1.5, 1.5),
                                               p(0.0, 1.5),
                                               p(0.0, 0.0)]);
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
        let poly1 = Polygon::new(LineString(vec![p(0., 0.), p(1., 0.), p(1., 1.), p(0., 1.), p(0., 0.)]),
                                 Vec::new());
        let poly2 = Polygon::new(LineString(vec![p(2., 0.), p(3., 0.), p(3., 1.), p(2., 1.), p(2., 0.)]),
                                 Vec::new());
        let multipoly = MultiPolygon(vec![poly1, poly2]);
        assert!(multipoly.contains(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains(&Point::new(1.5, 0.5)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_and_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly1 = Polygon::new(LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
                                 vec![LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 1.)])]);
        let poly2 = Polygon::new(LineString(vec![p(9., 0.), p(14., 0.), p(14., 4.), p(9., 4.), p(9., 0.)]),
                                 Vec::new());

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

        let poly = Polygon::new(LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
                                vec![LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 4.), p(1., 1.)])]);
        assert!(!poly.contains(&LineString(vec![p(2., 2.), p(3., 3.)])));
        assert!(!poly.contains(&LineString(vec![p(2., 2.), p(2., 5.)])));
        assert!(!poly.contains(&LineString(vec![p(3., 0.5), p(3., 5.)])));
    }
    #[test]
    fn bbox_in_inner_bbox_test() {
        let bbox_xl = Bbox { xmin: -100., xmax: 100., ymin: -200., ymax: 200.};
        let bbox_sm = Bbox { xmin: -10., xmax: 10., ymin: -20., ymax: 20.};
        assert_eq!(true, bbox_xl.contains(&bbox_sm));
        assert_eq!(false, bbox_sm.contains(&bbox_xl));
    }
    #[test]
    fn point_in_line_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p0 = p(2., 4.);
        // vertical line
        let line1 = Line((p(2., 0.), p(2., 5.)));
        // point on line, but outside line segment
        let line2 = Line((p(0., 6.), p(1.5, 4.5)));
        // point on line
        let line3 = Line((p(0., 6.), p(3., 3.)));
        // point within precision of line
        let line4 = Line((p(0., 6.00001), p(3., 3.0001)));
        assert!(line1.contains(&p0));
        assert!(!line2.contains(&p0));
        assert!(line3.contains(&p0));
        assert!(line4.contains(&p0));
    }
}
