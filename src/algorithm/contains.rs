use num_traits::Float;

use types::COORD_PRECISION;
use traits::{PointTrait, LineStringTrait, PolygonTrait, MultiPolygonTrait};

pub fn point_contains_point<'a, P1, P2, T>(point1: &'a P1, point2: &'a P2) -> bool
    where T: 'a + Float ,
          P1: 'a + PointTrait<T> + ?Sized,
          P2: 'a + PointTrait<T> + ?Sized,
{
    point1.distance_to_point(point2).to_f32().unwrap() < COORD_PRECISION
}

pub fn line_string_contains_point<'a, L, P, T>(line_string: &'a L, point: &'a P) -> bool
    where T: 'a + Float ,
          L: 'a + LineStringTrait<'a, T> + ?Sized,
          P: 'a + PointTrait<T> + ?Sized,
{
    // FIXME: remove collect
    let vect = line_string.points().collect::<Vec<_>>();

    // LineString without points
    if vect.is_empty() {
        return false;
    }
    // LineString with one point equal p
    if vect.len() == 1 {
        return vect[0].contains_point(point);
    }
    // check if point is a vertex
    for p in &vect {
        if p.eq_coordinates(point) {
            return true;
        }
    }

    for ps in vect.windows(2) {
        if ((ps[0].y() == ps[1].y()) && (ps[0].y() == point.y()) &&
            (point.x() > ps[0].x().min(ps[1].x())) &&
            (point.x() < ps[0].x().max(ps[1].x()))) ||
           ((ps[0].x() == ps[1].x()) && (ps[0].x() == point.x()) &&
            (point.y() > ps[0].y().min(ps[1].y())) &&
            (point.y() < ps[0].y().max(ps[1].y()))) {
            return true;
        }
    }
    false
}

#[derive(PartialEq, Clone, Debug)]
enum PositionPoint {
    OnBoundary,
    Inside,
    Outside,
}

fn get_position<'a, P, L, T>(point: &'a P, line_string: &'a L) -> PositionPoint
    where T: 'a + Float ,
          P: 'a + PointTrait<T> + ?Sized,
          L: 'a + LineStringTrait<'a, T> + ?Sized,
{
    // See: http://www.ecse.rpi.edu/Homepages/wrf/Research/Short_Notes/pnpoly.html
    //      http://geospatialpython.com/search
    //         ?updated-min=2011-01-01T00:00:00-06:00&updated-max=2012-01-01T00:00:00-06:00&max-results=19
    // Return the position of the point relative to a linestring

    // TODO: remove `collect` call here
    let vect = line_string.points().collect::<Vec<_>>();

    // LineString without points
    if vect.is_empty() {
        return PositionPoint::Outside;
    }
    // Point is on linestring
    if line_string.contains_point(point) {
        return PositionPoint::OnBoundary;
    }

    let mut xints = T::zero();
    let mut crossings = 0;
    for ps in vect.windows(2) {
        if point.y() > ps[0].y().min(ps[1].y()) {
            if point.y() <= ps[0].y().max(ps[1].y()) {
                if point.x() <= ps[0].x().max(ps[1].x()) {
                    if ps[0].y() != ps[1].y() {
                        xints = (point.y() - ps[0].y()) * (ps[1].x() - ps[0].x()) /
                                (ps[1].y() - ps[0].y()) + ps[0].x();
                    }
                    if (ps[0].x() == ps[1].x()) || (point.x() <= xints) {
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

pub fn polygon_contains_point<'a, P1, P2, T>(polygon: &'a P1, point: &'a P2) -> bool
    where T: 'a + Float ,
          P1: 'a + PolygonTrait<'a, T> + ?Sized,
          P2: 'a + PointTrait<T> + ?Sized,
{
    let mut rings = polygon.rings();
    let exterior_ring = rings.next().expect("expected outer ring");
    match get_position(point, exterior_ring) {
        PositionPoint::OnBoundary => false,
        PositionPoint::Outside => false,
        _ => rings.all(|ls| get_position(point, ls) == PositionPoint::Outside),
    }
}

pub fn multi_polygon_contains_point<'a, M, P, T>(multi_polygon: &'a M, point: &'a P) -> bool
    where T: 'a + Float ,
          M: 'a + MultiPolygonTrait<'a, T> + ?Sized,
          P: 'a + PointTrait<T> + ?Sized,
{
    multi_polygon.polygons().any(|poly| poly.contains_point(point))
}

pub fn polygon_contains_line_string<'a, P, L, T>(polygon: &'a P, line_string: &'a L) -> bool
    where T: 'a + Float ,
          P: 'a + PolygonTrait<'a, T> + ?Sized,
          L: 'a + LineStringTrait<'a, T> + Sized,
{
    // All points of LineString must be in the polygon ?
    if line_string.points().all(|point| polygon.contains_point(point)) {
        !polygon.intersects_line_string(line_string)
    } else {
        false
    }
}

/*
impl<T> Contains<Bbox<T>> for Bbox<T>
    where T: Float 
{
    fn contains(&self, bbox: &Bbox<T>) -> bool {
        // All points of LineString must be in the polygon ?
        self.xmin <= bbox.xmin && self.xmax >= bbox.xmax && self.ymin <= bbox.ymin && self.ymax >= bbox.ymax
    }
}
*/


#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon, MultiPolygon};
    use traits::{LineStringTrait, PolygonTrait, MultiPolygonTrait};
    /// Tests: Point in LineString
    #[test]
    fn empty_linestring_test() {
        let linestring = LineString(Vec::new());
        assert!(!linestring.contains_point(&Point::new(2., 1.)));
    }
    #[test]
    fn linestring_point_is_vertex_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.)]);
        assert!(linestring.contains_point(&p(2., 2.)));
    }
    #[test]
    fn linestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.)]);
        assert!(linestring.contains_point(&p(1., 0.)));
    }
    /// Tests: Point in Polygon
    #[test]
    fn empty_polygon_test() {
        let linestring = LineString(Vec::new());
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains_point(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_one_point_test() {
        let linestring = LineString(vec![Point::new(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains_point(&Point::new(3., 1.)));
    }
    #[test]
    fn polygon_with_one_point_is_vertex_test() {
        let linestring = LineString(vec![Point::new(2., 1.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains_point(&Point::new(2., 1.)));
    }
    #[test]
    fn polygon_with_point_on_boundary_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains_point(&p(1., 0.)));
        assert!(!poly.contains_point(&p(2., 1.)));
        assert!(!poly.contains_point(&p(1., 2.)));
        assert!(!poly.contains_point(&p(0., 1.)));
    }
    #[test]
    fn point_in_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(poly.contains_point(&p(1., 1.)));
    }
    #[test]
    fn point_out_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains_point(&p(2.1, 1.)));
        assert!(!poly.contains_point(&p(1., 2.1)));
        assert!(!poly.contains_point(&p(2.1, 2.1)));
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
        assert!(poly.contains_point(&p(0.25, 0.25)));
        assert!(!poly.contains_point(&p(1., 1.)));
        assert!(!poly.contains_point(&p(1.5, 1.5)));
        assert!(!poly.contains_point(&p(1.5, 1.)));
    }
    /// Tests: Point in MultiPolygon
    #[test]
    fn empty_multipolygon_test() {
        let multipoly = MultiPolygon(Vec::new());
        assert!(!multipoly.contains_point(&Point::new(2., 1.)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly1 = Polygon::new(LineString(vec![p(0., 0.), p(1., 0.), p(1., 1.), p(0., 1.), p(0., 0.)]),
                                 Vec::new());
        let poly2 = Polygon::new(LineString(vec![p(2., 0.), p(3., 0.), p(3., 1.), p(2., 1.), p(2., 0.)]),
                                 Vec::new());
        let multipoly = MultiPolygon(vec![poly1, poly2]);
        assert!(multipoly.contains_point(&Point::new(0.5, 0.5)));
        assert!(multipoly.contains_point(&Point::new(2.5, 0.5)));
        assert!(!multipoly.contains_point(&Point::new(1.5, 0.5)));
    }
    #[test]
    fn empty_multipolygon_two_polygons_and_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly1 = Polygon::new(LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
                                 vec![LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 1.)])]);
        let poly2 = Polygon::new(LineString(vec![p(9., 0.), p(14., 0.), p(14., 4.), p(9., 4.), p(9., 0.)]),
                                 Vec::new());

        let multipoly = MultiPolygon(vec![poly1, poly2]);
        assert!(multipoly.contains_point(&Point::new(3., 5.)));
        assert!(multipoly.contains_point(&Point::new(12., 2.)));
        assert!(!multipoly.contains_point(&Point::new(3., 2.)));
        assert!(!multipoly.contains_point(&Point::new(7., 2.)));
    }
    /// Tests: LineString in Polygon
    #[test]
    fn linestring_in_polygon_with_linestring_is_boundary_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring.clone(), Vec::new());
        assert!(!poly.contains_line_string(&linestring.clone()));
        assert!(!poly.contains_line_string(&LineString(vec![p(0., 0.), p(2., 0.)])));
        assert!(!poly.contains_line_string(&LineString(vec![p(2., 0.), p(2., 2.)])));
        assert!(!poly.contains_line_string(&LineString(vec![p(0., 2.), p(0., 0.)])));
    }
    #[test]
    fn linestring_outside_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(!poly.contains_line_string(&LineString(vec![p(1., 1.), p(3., 0.)])));
        assert!(!poly.contains_line_string(&LineString(vec![p(3., 0.), p(5., 2.)])));
    }
    #[test]
    fn linestring_in_inner_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });

        let poly = Polygon::new(LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]),
                                vec![LineString(vec![p(1., 1.), p(4., 1.), p(4., 4.), p(1., 4.), p(1., 1.)])]);
        assert!(!poly.contains_line_string(&LineString(vec![p(2., 2.), p(3., 3.)])));
        assert!(!poly.contains_line_string(&LineString(vec![p(2., 2.), p(2., 5.)])));
        assert!(!poly.contains_line_string(&LineString(vec![p(3., 0.5), p(3., 5.)])));
    }
    /*
    #[test]
    fn bbox_in_inner_bbox_test() {
        let bbox_xl = Bbox { xmin: -100., xmax: 100., ymin: -200., ymax: 200.};
        let bbox_sm = Bbox { xmin: -10., xmax: 10., ymin: -20., ymax: 20.};
        assert_eq!(true, bbox_xl.contains(&bbox_sm));
        assert_eq!(false, bbox_sm.contains(&bbox_xl));
    }
    */
}
