use num_traits::Float;
use types::{Line, LineString, Polygon, MultiPolygon, Bbox};

/// Calculation of the area.

pub trait Area<T> where T: Float
{
    /// Area of polygon.
    /// See: https://en.wikipedia.org/wiki/Polygon
    ///
    /// ```
    /// use geo::{Coordinate, Point, LineString, Polygon};
    /// use geo::algorithm::area::Area;
    /// let p = |x, y| Point(Coordinate { x: x, y: y });
    /// let v = Vec::new();
    /// let linestring = LineString::new(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]).unwrap();
    /// let poly = Polygon::new(linestring, v);
    /// assert_eq!(poly.area(), 30.);
    /// ```
    fn area(&self) -> T;
}

fn get_linestring_area<T>(linestring: &LineString<T>) -> T where T: Float {
    let mut tmp = T::zero();
    for line in linestring.lines() {
        tmp = tmp + (line.start.x() * line.end.y() - line.end.x() * line.start.y());
    }
    tmp / (T::one() + T::one())
}

impl<T> Area<T> for Line<T>
    where T: Float
{
    fn area(&self) -> T {
        T::zero()
    }
}

impl<T> Area<T> for Polygon<T>
    where T: Float
{
    fn area(&self) -> T {
        self.interiors.iter().fold(get_linestring_area(&self.exterior),
                                   |total, next| total - get_linestring_area(next))
    }
}

impl<T> Area<T> for MultiPolygon<T>
    where T: Float
{
    fn area(&self) -> T {
        self.0.iter().fold(T::zero(), |total, next| total + next.area())
    }
}

impl<T> Area<T> for Bbox<T>
    where T: Float
{
    fn area(&self) -> T {
        (self.xmax - self.xmin) * (self.ymax - self.ymin)
    }
}

#[cfg(test)]
mod test {
    use types::{Coordinate, Point, Line, LineString, Polygon, MultiPolygon, Bbox};
    use algorithm::area::Area;

    // Area of the polygon
    #[test]
    fn area_empty_polygon_test() {
        let poly = Polygon::<f64>::new(LineString::new(Vec::new()), Vec::new()).unwrap();
        assert_relative_eq!(poly.area(), 0.);
    }

    #[test]
    fn area_one_point_polygon_test() {
        let poly = Polygon::new(LineString::new(vec![Point::new(1., 0.)]), Vec::new()).unwrap();
        assert_relative_eq!(poly.area(), 0.);
    }
    #[test]
    fn area_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString::new(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]).unwrap();
        let poly = Polygon::new(linestring, Vec::new());
        assert_relative_eq!(poly.area(), 30.);
    }
    #[test]
    fn bbox_test() {
        let bbox = Bbox {xmin: 10., xmax: 20., ymin: 30., ymax: 40.};
        assert_relative_eq!(bbox.area(), 100.);
    }
    #[test]
    fn area_polygon_inner_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let outer = LineString::new(vec![p(0., 0.), p(10., 0.), p(10., 10.), p(0., 10.), p(0., 0.)]).unwrap();
        let inner0 = LineString::new(vec![p(1., 1.), p(2., 1.), p(2., 2.), p(1., 2.), p(1., 1.)]).unwrap();
        let inner1 = LineString::new(vec![p(5., 5.), p(6., 5.), p(6., 6.), p(5., 6.), p(5., 5.)]).unwrap();
        let poly = Polygon::new(outer, vec![inner0, inner1]);
        assert_relative_eq!(poly.area(), 98.);
    }
    #[test]
    fn area_multipolygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly0 = Polygon::new(LineString::new(vec![p(0., 0.), p(10., 0.), p(10., 10.), p(0., 10.).unwrap(),
                                                 p(0., 0.)]),
                                 Vec::new());
        let poly1 = Polygon::new(LineString::new(vec![p(1., 1.), p(2., 1.), p(2., 2.), p(1., 2.).unwrap(),
                                                 p(1., 1.)]),
                                 Vec::new());
        let poly2 = Polygon::new(LineString::new(vec![p(5., 5.), p(6., 5.), p(6., 6.), p(5., 6.).unwrap(),
                                                 p(5., 5.)]),
                                 Vec::new());
        let mpoly = MultiPolygon(vec![poly0, poly1, poly2]);
        assert_eq!(mpoly.area(), 102.);
        assert_relative_eq!(mpoly.area(), 102.);
    }
    #[test]
    fn area_line_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let line1 = Line::new(p(0.0, 0.0), p(1.0, 1.0));
        assert_eq!(line1.area(), 0.);
    }
}
