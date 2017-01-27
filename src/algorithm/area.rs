use num_traits::Float;
use {MultiPolygonTrait, PolygonTrait, LineStringTrait, PointTrait};

fn get_linestring_area<'a, T, G>(linestring: &'a G) -> T
    where T: 'a + Float,
          G: 'a + LineStringTrait<'a, T>
{
    let mut points = linestring.points();
    let mut p1 = match points.next() {
        Some(p) => p,
        None => return T::zero(),
    };
    let mut tmp = T::zero();
    for p2 in points {
        tmp = tmp + (p1.x() * p2.y() - p2.x() * p1.y());
        p1 = p2;
    }
    tmp / (T::one() + T::one())
}

pub fn polygon<'a, G, T>(polygon: &'a G) -> T
    where T: 'a + Float,
          G: 'a + PolygonTrait<'a, T> + ?Sized
{
    let mut rings = polygon.rings();
    let outer_ring = rings.next().expect("no outer ring in polygon");
    let outer_ring_area = get_linestring_area(outer_ring);
    rings.fold(outer_ring_area,
               |acc, inner_ring| acc - get_linestring_area(inner_ring))
}

pub fn multi_polygon<'a, G, T>(multi_polygon: &'a G) -> T
    where T: 'a + Float,
          G: 'a + MultiPolygonTrait<'a, T> + ?Sized
{
    multi_polygon.polygons().map(polygon).fold(T::zero(), |acc, n| acc + n)
}

#[cfg(test)]
mod test {
    use num_traits::Float;
    use types::{Coordinate, Point, LineString, Polygon, MultiPolygon};
    use test_helpers::within_epsilon;
    use {PolygonTrait, MultiPolygonTrait};

    #[test]
    fn area_empty_polygon_test() {
        let poly = Polygon::<f64>::new(LineString(Vec::new()), Vec::new());
        assert!(within_epsilon(poly.area(), 0., Float::epsilon()));
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = Polygon::new(LineString(vec![Point::new(1., 0.)]), Vec::new());
        assert!(within_epsilon(poly.area(), 0., Float::epsilon()));
    }
    #[test]
    fn area_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert!(within_epsilon(poly.area(), 30., Float::epsilon()));
    }

    #[test]
    fn area_polygon_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let outer = LineString(vec![p(0., 0.), p(10., 0.), p(10., 10.), p(0., 10.), p(0., 0.)]);
        let inner0 = LineString(vec![p(1., 1.), p(2., 1.), p(2., 2.), p(1., 2.), p(1., 1.)]);
        let inner1 = LineString(vec![p(5., 5.), p(6., 5.), p(6., 6.), p(5., 6.), p(5., 5.)]);
        let poly = Polygon::new(outer, vec![inner0, inner1]);
        assert!(within_epsilon(poly.area(), 98., Float::epsilon()));
    }

    #[test]
    fn area_multipolygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly0 = Polygon::new(LineString(vec![p(0., 0.),
                                                 p(10., 0.),
                                                 p(10., 10.),
                                                 p(0., 10.),
                                                 p(0., 0.)]),
                                 Vec::new());
        let poly1 = Polygon::new(LineString(vec![p(1., 1.), p(2., 1.), p(2., 2.), p(1., 2.),
                                                 p(1., 1.)]),
                                 Vec::new());
        let poly2 = Polygon::new(LineString(vec![p(5., 5.), p(6., 5.), p(6., 6.), p(5., 6.),
                                                 p(5., 5.)]),
                                 Vec::new());
        let mpoly = MultiPolygon(vec![poly0, poly1, poly2]);
        assert_eq!(mpoly.area(), 102.);
        assert!(within_epsilon(mpoly.area(), 102., Float::epsilon()));
    }
}
