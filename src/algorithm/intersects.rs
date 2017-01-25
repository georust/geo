use num_traits::Float;
use traits::{LineStringTrait, PolygonTrait, PointTrait};

pub fn line_string_intersects_line_string<'a, L1, L2, T>(line_string1: &'a L1, line_string2: &'a L2) -> bool
    where T: 'a + Float + ::num_traits::FromPrimitive,
          L1: 'a + LineStringTrait<'a, T> + ?Sized,
          L2: 'a + LineStringTrait<'a, T> + ?Sized,
{
    // See: https://github.com/brandonxiang/geojson-python-utils/blob/33b4c00c6cf27921fb296052d0c0341bd6ca1af2/geojson_utils.py
    // TODO: remove `collect`
    let vect0 = line_string1.points().collect::<Vec<_>>();
    let vect1 = line_string2.points().collect::<Vec<_>>();
    if vect0.is_empty() || vect1.is_empty() {
        return false;
    }
    for a in vect0.windows(2) {
        for b in vect1.windows(2) {
            let u_b = (b[1].y() - b[0].y()) * (a[1].x() - a[0].x()) -
                      (b[1].x() - b[0].x()) * (a[1].y() - a[0].y());
            if u_b == T::zero() {
                continue;
            }
            let ua_t = (b[1].x() - b[0].x()) * (a[0].y() - b[0].y()) -
                       (b[1].y() - b[0].y()) * (a[0].x() - b[0].x());
            let ub_t = (a[1].x() - a[0].x()) * (a[0].y() - b[0].y()) -
                       (a[1].y() - a[0].y()) * (a[0].x() - b[0].x());
            let u_a = ua_t / u_b;
            let u_b = ub_t / u_b;
            if (T::zero() <= u_a) && (u_a <= T::one()) && (T::zero() <= u_b) && (u_b <= T::one()) {
                return true;
            }
        }
    }
    false
}

pub fn polygon_intersects_line_string<'a, P, L, T>(polygon: &'a P, line_string: &'a L) -> bool
    where T: 'a + Float + ::num_traits::FromPrimitive,
          P: 'a + PolygonTrait<'a, T> + ?Sized,
          L: 'a + LineStringTrait<'a, T> + Sized,
{
    let mut rings = polygon.rings();
    let exterior_ring = rings.next().expect("no outer ring");

    // line intersects inner or outer polygon edge
    if exterior_ring.intersects_line_string(line_string) || rings.any(|inner| inner.intersects_line_string(line_string)) {
        return true;
    } else {
        // or if it's contained in the polygon
        return line_string.points().any(|point| polygon.contains_point(point))
    }
}

/*
impl<T> Intersects<Bbox<T>> for Bbox<T>
    where T: Float + ::num_traits::FromPrimitive
{
    fn intersects(&self, bbox: &Bbox<T>) -> bool {
        // line intersects inner or outer polygon edge
        if bbox.contains_point(&self) {
            return false
        } else {
            (self.xmin >= bbox.xmin && self.xmin <= bbox.xmax || self.xmax >= bbox.xmin && self.xmax <= bbox.xmax) &&
            (self.ymin >= bbox.ymin && self.ymin <= bbox.ymax || self.ymax >= bbox.ymin && self.ymax <= bbox.ymax)
        }
    }
}

impl<T> Intersects<Polygon<T>> for Bbox<T>
    where T: Float + ::num_traits::FromPrimitive
{
    fn intersects(&self, polygon: &Polygon<T>) -> bool {
        polygon.intersects(self)
    }
}

impl<T> Intersects<Bbox<T>> for Polygon<T>
    where T: Float + ::num_traits::FromPrimitive
{
    fn intersects(&self, bbox: &Bbox<T>) -> bool {
        let p = Polygon::new(LineString(vec![Point::new(bbox.xmin, bbox.ymin),
                                             Point::new(bbox.xmin, bbox.ymax),
                                             Point::new(bbox.xmax, bbox.ymax),
                                             Point::new(bbox.xmax, bbox.ymin),
                                             Point::new(bbox.xmin, bbox.ymin)]),
                             vec![]);
        self.intersects(&p)
    }
}
*/

pub fn polygon_intersects_polygon<'a, P1, P2, T>(polygon1: &'a P1, polygon2: &'a P2) -> bool
    where T: 'a + Float + ::num_traits::FromPrimitive,
          P1: 'a + PolygonTrait<'a, T> + ?Sized,
          P2: 'a + PolygonTrait<'a, T> + ?Sized,
{
    let mut polygon1_rings = polygon1.rings();
    let polygon1_exterior_ring = polygon1_rings.next().expect("no outer ring");

    let mut polygon2_rings = polygon2.rings();
    let polygon2_exterior_ring = polygon2_rings.next().expect("no outer ring");

    // self intersects (or contains) any line in polygon
    polygon1.intersects_line_string(polygon2_exterior_ring) ||
        polygon2_rings.any(|inner_line_string| polygon1.intersects_line_string(inner_line_string)) ||
        // self is contained inside polygon
        polygon2.intersects_line_string(polygon1_exterior_ring)
}

#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon};
    use traits::{PolygonTrait, LineStringTrait};
    /// Tests: intersection LineString and LineString
    #[test]
    fn empty_linestring1_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(!LineString(Vec::new()).intersects_line_string(&linestring));
    }
    #[test]
    fn empty_linestring2_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(!linestring.intersects_line_string(&LineString(Vec::new())));
    }
    #[test]
    fn empty_all_linestring_test() {
        assert!(!LineString::<f64>(Vec::new()).intersects_line_string(&LineString(Vec::new())));
    }
    #[test]
    fn intersect_linestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(linestring.intersects_line_string(&LineString(vec![p(3., 4.), p(8., 4.)])));
    }
    #[test]
    fn parallel_linestrings_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(3., 2.), p(7., 6.)]);
        assert!(!linestring.intersects_line_string(&LineString(vec![p(3., 1.), p(7., 5.)])));
    }
    /// Tests: intersection LineString and Polygon
    #[test]
    fn linestring_in_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.intersects_line_string(&LineString(vec![p(2., 2.), p(3., 3.)])));
    }
    #[test]
    fn linestring_on_boundary_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
                                Vec::new());
        assert!(poly.intersects_line_string(&LineString(vec![p(0., 0.), p(5., 0.)])));
        assert!(poly.intersects_line_string(&LineString(vec![p(5., 0.), p(5., 6.)])));
        assert!(poly.intersects_line_string(&LineString(vec![p(5., 6.), p(0., 6.)])));
        assert!(poly.intersects_line_string(&LineString(vec![p(0., 6.), p(0., 0.)])));
    }
    #[test]
    fn intersect_linestring_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
                                Vec::new());
        assert!(poly.intersects_line_string(&LineString(vec![p(2., 2.), p(6., 6.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
                                Vec::new());
        assert!(!poly.intersects_line_string(&LineString(vec![p(7., 2.), p(9., 4.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let e = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let v = vec![LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 4.), p(1., 1.)])];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects_line_string(&LineString(vec![p(2., 2.), p(3., 3.)])));
        assert!(poly.intersects_line_string(&LineString(vec![p(2., 2.), p(4., 4.)])));
    }
    #[test]
    fn linestring_traverse_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let e = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let v = vec![LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 4.), p(1., 1.)])];
        let poly = Polygon::new(e, v);
        assert!(poly.intersects_line_string(&LineString(vec![p(2., 0.5), p(2., 5.)])));
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

        let e = LineString(vec![p(2., 2.), p(14., 2.), p(14., 8.), p(2., 8.), p(2., 2.)]);
        let v = vec![LineString(vec![p(4., 3.), p(7., 3.), p(7., 6.), p(4., 6.), p(4., 3.)]),
                     LineString(vec![p(9., 3.), p(12., 3.), p(12., 6.), p(9., 6.), p(9., 3.)])];
        let poly = Polygon::new(e, v);
        assert!(!poly.intersects_line_string(&LineString(vec![p(5., 4.), p(6., 5.)])));
        assert!(poly.intersects_line_string(&LineString(vec![p(11., 2.5), p(11., 7.)])));
        assert!(poly.intersects_line_string(&LineString(vec![p(4., 7.), p(6., 7.)])));
        assert!(poly.intersects_line_string(&LineString(vec![p(8., 1.), p(8., 9.)])));
    }
    #[test]
    fn polygons_do_not_intersect() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(LineString(vec![p(1., 3.), p(3., 3.), p(3., 5.), p(1., 5.), p(1., 3.)]),
                                    Vec::new());
        let p2 = Polygon::new(LineString(vec![p(10., 30.), p(30., 30.), p(30., 50.), p(10., 50.), p(10., 30.)]),
                                    Vec::new());

        assert!(!p1.intersects_polygon(&p2));
        assert!(!p2.intersects_polygon(&p1));
    }
    #[test]
    fn polygons_overlap() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(LineString(vec![p(1., 3.), p(3., 3.), p(3., 5.), p(1., 5.), p(1., 3.)]),
                                    Vec::new());
        let p2 = Polygon::new(LineString(vec![p(2., 3.), p(4., 3.), p(4., 7.), p(2., 7.), p(2., 3.)]),
                                    Vec::new());

        assert!(p1.intersects_polygon(&p2));
        assert!(p2.intersects_polygon(&p1));
    }
    #[test]
    fn polygon_contained() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(LineString(vec![p(1., 3.), p(4., 3.), p(4., 6.), p(1., 6.), p(1., 3.)]),
                                    Vec::new());
        let p2 = Polygon::new(LineString(vec![p(2., 4.), p(3., 4.), p(3., 5.), p(2., 5.), p(2., 4.)]),
                                    Vec::new());

        assert!(p1.intersects_polygon(&p2));
        assert!(p2.intersects_polygon(&p1));
    }
    #[test]
    fn polygons_conincident() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let p1 = Polygon::new(LineString(vec![p(1., 3.), p(4., 3.), p(4., 6.), p(1., 6.), p(1., 3.)]),
                                    Vec::new());
        let p2 = Polygon::new(LineString(vec![p(1., 3.), p(4., 3.), p(4., 6.), p(1., 6.), p(1., 3.)]),
                                    Vec::new());

        assert!(p1.intersects_polygon(&p2));
        assert!(p2.intersects_polygon(&p1));
    }
    /*
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
        let poly  = Polygon::new(LineString(vec![p(0., 0.), p(12., 0.), p(12., 8.), p(0., 8.), p(0., 0.)]),
                                 vec![LineString(vec![p(7., 4.), p(11., 4.), p(11., 7.), p(7., 7.), p(7., 4.)])]);
        let b1 = Bbox { xmin: 11.0, xmax: 13.0, ymin: 1.0, ymax: 2.0 };
        let b2 = Bbox { xmin: 2.0, xmax: 8.0, ymin: 2.0, ymax: 5.0 };
        let b3 = Bbox { xmin: 8.0, xmax: 10.0, ymin: 5.0, ymax: 6.0 };
        let b4 = Bbox { xmin: 1.0, xmax: 3.0, ymin: 1.0, ymax: 3.0 };
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
        let bbox_xl = Bbox { xmin: -100., xmax: 100., ymin: -200., ymax: 200.};
        let bbox_sm = Bbox { xmin: -10., xmax: 10., ymin: -20., ymax: 20.};
        let bbox_s2 = Bbox { xmin: 0., xmax: 20., ymin: 0., ymax: 30.};
        assert_eq!(false, bbox_xl.intersects(&bbox_sm));
        assert_eq!(false, bbox_sm.intersects(&bbox_xl));
        assert_eq!(true, bbox_sm.intersects(&bbox_s2));
        assert_eq!(true, bbox_s2.intersects(&bbox_sm));
    }
    */
}
