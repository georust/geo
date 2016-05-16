
use types::{COORD_PRECISION, Point, LineString, Polygon, MultiPolygon};
use algorithm::intersects::Intersects;
use algorithm::distance::Distance;

///  Checks if the geometry A is completely inside the B geometry.

pub trait Contains<RHS = Self> {
    ///  Checks if the geometry A is completely inside the B geometry.
    ///
    /// ```
    /// use geo::{Coordinate, Point, LineString, Polygon};
    /// use geo::algorithm::contains::Contains;
    ///
    /// let p = |x, y| Point(Coordinate { x: x, y: y });
    /// let v = Vec::new();
    /// let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
    /// let poly = Polygon(linestring.clone(), v);
    ///
    /// //Point in Point
    /// assert!(p(2., 0.).contains(&p(2., 0.)));
    ///
    /// //Point in Linstring
    /// assert!(linestring.contains(&p(2., 0.)));
    ///
    /// //Point in Polygon
    /// assert!(poly.contains(&p(1., 1.)));
    ///
    /// ```
    ///
    fn contains(&self, rhs: &RHS) -> bool;
}

impl Contains<Point> for Point {
    fn contains(&self, p: &Point) -> bool {
        self.distance(p) < COORD_PRECISION
    }
}

impl Contains<Point> for LineString {
    fn contains(&self, p: &Point) -> bool {
        let vect = &self.0;
        // LineString without points
        if vect.is_empty() {
            return false;
        }
        // LinString with one point equal p
        if vect.len() == 1 {
            return vect[0].contains(p);
        }
        // check if point is a vertex
        if vect.contains(p) {
            return true;
        }
        for (p1, p2) in vect.iter().zip(vect[1..].iter()) {
            if ((p1.lat() == p2.lat()) && (p1.lat() == p.lat()) &&
                (p.lng() > p1.lng().min(p2.lng())) &&
                (p.lng() < p1.lng().max(p2.lng()))) ||
               ((p1.lng() == p2.lng()) && (p1.lng() == p.lng()) &&
                (p.lat() > p1.lat().min(p2.lat())) &&
                (p.lat() < p1.lat().max(p2.lat()))) {
                return true;
            }
        }
        return false;
    }
}
#[derive(PartialEq, Clone, Debug)]
enum PositionPoint {
    OnBoundary,
    Inside,
    Outside,
}

fn get_position(p: &Point, linestring: &LineString) -> PositionPoint {
    // See: http://www.ecse.rpi.edu/Homepages/wrf/Research/Short_Notes/pnpoly.html
    // 	 http://geospatialpython.com/search
    // 		?updated-min=2011-01-01T00:00:00-06:00&updated-max=2012-01-01T00:00:00-06:00&max-results=19
    // Return the position of the point relative to a linestring

    let vect = &linestring.0;
    // LineString without points
    if vect.is_empty() {
        return PositionPoint::Outside;
    }
    // Point is on linstring
    if linestring.contains(p) {
        return PositionPoint::OnBoundary;
    }

    let mut xints = 0.;
    let mut crossings = 0;
    for (p1, p2) in vect.iter().zip(vect[1..].iter()) {
        if p.lat() > p1.lat().min(p2.lat()) {
            if p.lat() <= p1.lat().max(p2.lat()) {
                if p.lng() <= p1.lng().max(p2.lng()) {
                    if p1.lat() != p2.lat() {
                        xints = (p.lat() - p1.lat()) * (p2.lng() - p1.lng()) /
                                (p2.lat() - p1.lat()) + p1.lng();
                    }
                    if (p1.lng() == p2.lng()) || (p.lng() <= xints) {
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

impl Contains<Point> for Polygon {
    fn contains(&self, p: &Point) -> bool {
        match get_position(p, &self.0) {
            PositionPoint::OnBoundary => false,
            PositionPoint::Outside => false,
            _ => self.1.iter().all(|ls| get_position(p, ls) == PositionPoint::Outside),
        }
    }
}

impl Contains<Point> for MultiPolygon {
    fn contains(&self, p: &Point) -> bool {
        self.0.iter().any(|poly| poly.contains(p))
    }
}

impl Contains<LineString> for Polygon {
    fn contains(&self, linestring: &LineString) -> bool {
        // All points of LineString must be in the polygon ?
        match linestring.0.iter().all(|point| self.contains(point)) {
            true => !self.intersects(linestring),
            false => false,
        }
    }
}


#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon, MultiPolygon};
    use algorithm::contains::Contains;
    /// Tests: Point in LineString
    #[test]
    fn empty_linestring_test() {
        let v = Vec::new();
        let linestring = LineString(v);
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
        let v1 = Vec::new();
        let v2 = Vec::new();
        let linestring = LineString(v1);
        let poly = Polygon(linestring, v2);
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_one_point_test() {
        let p = Point::new(2., 1.);
        let v = Vec::new();
        let linestring = LineString(vec![p]);
        let poly = Polygon(linestring, v);
        assert!(!poly.contains(&Point::new(3., 1.)));
    }
    #[test]
    fn polygon_with_one_point_is_vertex_test() {
        let p = Point::new(2., 1.);
        let v = Vec::new();
        let linestring = LineString(vec![p]);
        let poly = Polygon(linestring, v);
        assert!(!poly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_point_on_boundary_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(!poly.contains(&p(1., 0.)));
        assert!(!poly.contains(&p(2., 1.)));
        assert!(!poly.contains(&p(1., 2.)));
        assert!(!poly.contains(&p(0., 1.)));
    }
    #[test]
    fn point_in_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(poly.contains(&p(1., 1.)));
    }
    #[test]
    fn point_out_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(!poly.contains(&p(2.1, 1.)));
        assert!(!poly.contains(&p(1., 2.1)));
        assert!(!poly.contains(&p(2.1, 2.1)));
    }
    #[test]
    fn point_polygon_with_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mut v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let inner_linestring = LineString(vec![p(0.5, 0.5),
                                               p(1.5, 0.5),
                                               p(1.5, 1.5),
                                               p(0.0, 1.5),
                                               p(0.0, 0.0)]);
        v.push(inner_linestring);
        let poly = Polygon(linestring, v);
        assert!(!poly.contains(&p(1., 1.)));
        assert!(!poly.contains(&p(1.5, 1.5)));
        assert!(!poly.contains(&p(1.5, 1.)));
    }
    /// Tests: Point in MultiPolygon
    #[test]
    fn empty_multipolygon_test() {
        let v = Vec::new();
        let multipoly = MultiPolygon(v);
        assert!(!multipoly.contains(&Point::new(2., 1.)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mut v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(1., 0.), p(1., 1.), p(0., 1.), p(0., 0.)]);
        let poly = Polygon(linestring, Vec::new());
        v.push(poly);
        let linestring = LineString(vec![p(2., 0.), p(3., 0.), p(3., 1.), p(2., 1.), p(2., 0.)]);
        let poly = Polygon(linestring, Vec::new());
        v.push(poly);
        let multipoly = MultiPolygon(v);
        assert!(multipoly.contains(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains(&Point::new(1.5, 0.5)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_and_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mut v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let inner_linestring = LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 1.)]);
        let poly = Polygon(linestring, vec![inner_linestring]);
        v.push(poly);
        let linestring = LineString(vec![p(9., 0.), p(14., 0.), p(14., 4.), p(9., 4.), p(9., 0.)]);
        let poly = Polygon(linestring, Vec::new());
        v.push(poly);
        let multipoly = MultiPolygon(v);
        assert!(multipoly.contains(&Point::new(3., 5.)));
        assert!(multipoly.contains(&Point::new(12., 2.)));
        assert!(!multipoly.contains(&Point::new(3., 2.)));
        assert!(!multipoly.contains(&Point::new(7., 2.)));
    }
    /// Tests: LineString in Polygon
    #[test]
    fn linstring_in_polygon_with_linestring_is_boundary_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon(linestring.clone(), v);
        assert!(!poly.contains(&linestring.clone()));
        assert!(!poly.contains(&LineString(vec![p(0., 0.), p(2., 0.)])));
        assert!(!poly.contains(&LineString(vec![p(2., 0.), p(2., 2.)])));
        assert!(!poly.contains(&LineString(vec![p(0., 2.), p(0., 0.)])));
    }
    #[test]
    fn linstring_outside_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(!poly.contains(&LineString(vec![p(1., 1.), p(3., 0.)])));
        assert!(!poly.contains(&LineString(vec![p(3., 0.), p(5., 2.)])));
    }
    #[test]
    fn linstring_in_inner_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mut v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let inner_linestring = LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 4.),
                                               p(1., 1.)]);
        v.push(inner_linestring);
        let poly = Polygon(linestring, v);
        assert!(!poly.contains(&LineString(vec![p(2., 2.), p(3., 3.)])));
        assert!(!poly.contains(&LineString(vec![p(2., 2.), p(2., 5.)])));
        assert!(!poly.contains(&LineString(vec![p(3., 0.5), p(3., 5.)])));
    }
}
