use types::{LineString, Polygon};
use algorithm::contains::Contains;

/// Checks if the geometry A intersects the geometry B.

pub trait Intersects<RHS = Self> {
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
    fn intersects(&self, rhs: &RHS) -> bool;
}

impl Intersects<LineString> for LineString {
    // See: https://github.com/brandonxiang/
    // 		geojson-python-utils/blob/33b4c00c6cf27921fb296052d0c0341bd6ca1af2/geojson_utils.py
    fn intersects(&self, linstring: &LineString) -> bool {
        let vect0 = &self.0;
        let vect1 = &linstring.0;
        if vect0.is_empty() || vect1.is_empty() {
            return false;
        }
        for (p01, p02) in vect0.iter().zip(vect0[1..].iter()) {
            for (p11, p12) in vect1.iter().zip(vect1[1..].iter()) {
                let u_b = (p02.lat() - p11.lat()) * (p02.lng() - p01.lng()) -
                          (p12.lng() - p11.lng()) * (p02.lat() - p01.lat());
                if u_b == 0. {
                    // The point is on boundary
                    return true;
                } else {
                    let ua_t = (p12.lng() - p11.lng()) * (p01.lat() - p11.lat()) -
                               (p02.lat() - p11.lat()) * (p01.lng() - p11.lng());
                    let ub_t = (p02.lng() - p01.lng()) * (p01.lat() - p11.lat()) -
                               (p02.lat() - p01.lat()) * (p01.lng() - p11.lng());
                    let u_a = ua_t / u_b;
                    let u_b = ub_t / u_b;
                    if (0. <= u_a) && (u_a <= 1.) && (0. <= u_b) && (u_b <= 1.) {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}

impl Intersects<LineString> for Polygon {
    fn intersects(&self, linstring: &LineString) -> bool {
        if !self.0.intersects(linstring) {
            return false;
        } else {
            if self.1.is_empty() {
                return true;
            } else {
                !self.1.iter().any(|ls| {
                    linstring.0.iter().all(|point| Polygon(ls.clone(), Vec::new()).contains(point))
                })
            }
        }
    }
}
#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon};
    use algorithm::intersects::Intersects;
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
        assert!(!LineString(Vec::new()).intersects(&LineString(Vec::new())));
    }
    #[test]
    fn intersect_linstring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(linestring.intersects(&LineString(vec![p(3., 4.), p(8., 4.)])));
    }
    /// Tests: intersection LineString and Polygon
    #[test]
    fn linstring_in_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(poly.intersects(&LineString(vec![p(2., 2.), p(3., 3.)])));
    }
    #[test]
    fn linstring_on_boundary_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(poly.intersects(&LineString(vec![p(0., 0.), p(5., 0.)])));
        assert!(poly.intersects(&LineString(vec![p(5., 0.), p(5., 6.)])));
        assert!(poly.intersects(&LineString(vec![p(5., 6.), p(0., 6.)])));
        assert!(poly.intersects(&LineString(vec![p(0., 6.), p(0., 0.)])));
    }
    #[test]
    fn intersect_linstring_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(poly.intersects(&LineString(vec![p(2., 2.), p(6., 6.)])));
    }
    #[test]
    fn linstring_outside_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert!(!poly.intersects(&LineString(vec![p(7., 2.), p(9., 4.)])));
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
        assert!(!poly.intersects(&LineString(vec![p(2., 2.), p(3., 3.)])));
        assert!(poly.intersects(&LineString(vec![p(2., 2.), p(4., 4.)])));
    }
    #[test]
    fn linstring_traverse_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mut v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let inner_linestring = LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 4.),
                                               p(1., 1.)]);
        v.push(inner_linestring);
        let poly = Polygon(linestring, v);
        assert!(poly.intersects(&LineString(vec![p(2., 0.5), p(2., 5.)])));
    }
    #[test]
    fn linstring_in_inner_with_2_inner_polygon_test() {
        //                                        (8,9)
        //     (2,8)                                |                                      (14,2)
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
        let mut v = Vec::new();
        let linestring = LineString(vec![p(2., 2.), p(14., 2.), p(14., 8.), p(2., 8.), p(2., 2.)]);
        let inner_linestring = LineString(vec![p(4., 3.), p(7., 3.), p(7., 6.), p(4., 6.),
                                               p(4., 3.)]);
        v.push(inner_linestring);
        let inner_linestring = LineString(vec![p(9., 3.), p(12., 3.), p(12., 6.), p(9., 6.),
                                               p(9., 3.)]);
        v.push(inner_linestring);
        let poly = Polygon(linestring, v);
        assert!(!poly.intersects(&LineString(vec![p(5., 4.), p(6., 5.)])));
        assert!(poly.intersects(&LineString(vec![p(11., 2.5), p(11., 7.)])));
        assert!(poly.intersects(&LineString(vec![p(4., 7.), p(6., 7.)])));
        assert!(poly.intersects(&LineString(vec![p(8., 1.), p(8., 9.)])));
    }
}
