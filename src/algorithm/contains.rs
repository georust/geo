
pub use types::{Coordinate, Point, LineString, Polygon};

/// Checks if the geometry A is completely inside the B geometry.

pub trait Contains<RHS = Self> {
    /// Checks if the geometry A is completely inside the B geometry.
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
        self == p
    }
}

impl Contains<Point> for LineString {
    fn contains(&self, p: &Point) -> bool {
        let vect = &self.0;
        if vect.is_empty() {
            return false;
        }
        if vect.len() == 1 {
        	return vect[0].contains(p);
        }
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
#[derive(PartialEq)]
enum PositionPoint {
    OnBoundary,
    Inside,
    Outside,
}

fn make_position(p: &Point, linestring: &LineString) -> PositionPoint {
        let vect = &linestring.0;
        if vect.is_empty() {
            return PositionPoint::Outside;
        }
		// On boundary
    	if linestring.contains(p) {
    		return PositionPoint::OnBoundary;
    	}
		// Is vertex 
        if vect.contains(p) {
        	return PositionPoint::Inside;
        }
        if vect.len() == 1 && vect[0].contains(p) {
        	return PositionPoint::Inside;
        }
        let mut xints = 0.;
        let mut crossings = 0;
        for (p1, p2) in vect.iter().zip(vect[1..].iter()) {
            if p.lat() > p1.lat().min(p2.lat()) {
                if p.lat() <= p1.lat().max(p2.lat()) {
                    if p.lng() <= p1.lng().max(p2.lng()) {
                        if p1.lat() != p2.lat() {
                            xints = (p.lat() - p1.lat()) * (p2.lng() - p1.lng()) /
                                    (p2.lat() - p1.lat()) +
                                    p1.lng();
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
    /// See: http://www.ecse.rpi.edu/Homepages/wrf/Research/Short_Notes/pnpoly.html
    /// 	 http://geospatialpython.com/search?updated-min=2011-01-01T00:00:00-06:00&updated-max=2012-01-01T00:00:00-06:00&max-results=19
    fn contains(&self, p: &Point) -> bool {
    	let linestring = &self.0;
		let value = make_position(p, linestring);
		if value == PositionPoint::OnBoundary || value == PositionPoint::Outside {
			false
		} else {
			let vect_linestring = &self.1;
			for ls in vect_linestring.iter() {
				let tmp = make_position(p, ls);
				if tmp == PositionPoint::OnBoundary || tmp == PositionPoint::Inside {
					return false;
				}
			}
			true
		}
    }
}

#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon};
    use algorithm::contains::Contains;
    #[test]
    fn empty_linestring_test() {
        let v = Vec::new();
        let linestring = LineString(v);
        assert_eq!(linestring.contains(&Point::new(2., 1.)), false);
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
    #[test]
    fn empty_polygon_test() {
        let v1 = Vec::new();
        let v2 = Vec::new();
        let linestring = LineString(v1);
        let poly = Polygon(linestring, v2);
        assert_eq!(poly.contains(&Point::new(2., 1.)), false);
    }
    #[test]
    fn polygon_with_one_point_test() {
        let p = Point::new(2., 1.);
        let v = Vec::new();
        let linestring = LineString(vec![p]);
        let poly = Polygon(linestring, v);
        assert_eq!(poly.contains(&Point::new(3., 1.)), false);
    }
    #[test]
    fn polygon_with_one_point_is_vertex_test() {
        let p = Point::new(2., 1.);
        let v = Vec::new();
        let linestring = LineString(vec![p]);
        let poly = Polygon(linestring, v);
        assert_eq!(poly.contains(&Point::new(2., 1.)), false);
    }
    #[test]
    fn polygon_with_point_on_boundary_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert_eq!(poly.contains(&p(1., 0.)), false);
        assert_eq!(poly.contains(&p(2., 1.)), false);
        assert_eq!(poly.contains(&p(1., 2.)), false);
        assert_eq!(poly.contains(&p(0., 1.)), false);
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
        assert_eq!(poly.contains(&p(2.1, 1.)), false);
        assert_eq!(poly.contains(&p(1., 2.1)), false);
        assert_eq!(poly.contains(&p(2.1, 2.1)), false);
    }
    #[test]
    fn point_polygon_with_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mut v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let inner_linestring = LineString(vec![p(0.5, 0.5), p(1.5, 0.5), p(1.5, 1.5), p(0.0, 1.5), p(0.0, 0.0)]);
        v.push(inner_linestring);
        let poly = Polygon(linestring, v);
        assert_eq!(poly.contains(&p(1., 1.)), false);
        assert_eq!(poly.contains(&p(1.5, 1.5)), false);
        assert_eq!(poly.contains(&p(1.5, 1.)), false);
    }
}
