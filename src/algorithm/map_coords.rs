use num_traits::Float;
use types::{Point, Polygon, LineString, Line, MultiPoint, MultiPolygon, MultiLineString, GeometryCollection, Geometry};

/// Map all the coordinates in an object, returning a new one
pub trait MapCoords<T, NT> {
    type Output;

    /// Apply a function to all the coordinates in a geometric object, returning a new object.
    ///
    /// ```
    /// use geo::Point;
    /// use geo::algorithm::map_coords::MapCoords;
    ///
    /// let p1 = Point::new(10., 20.);
    /// let p2 = p1.map_coords(&|&(x, y)| (x+1000., y*2.));
    ///
    /// assert_eq!(p2, Point::new(1010., 40.));
    /// ```
    ///
    /// You can conver the coordinate type this way as well
    ///
    /// ```
    /// # use geo::Point;
    /// # use geo::algorithm::map_coords::MapCoords;
    ///
    /// let p1: Point<f32> = Point::new(10.0f32, 20.0f32);
    /// let p2: Point<f64> = p1.map_coords(&|&(x, y)| (x as f64, y as f64));
    ///
    /// assert_eq!(p2, Point::new(10.0f64, 20.0f64));
    /// ```
    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
        where T: Float, NT: Float;


}

impl<T: Float, NT: Float> MapCoords<T, NT> for Point<T> {
    type Output = Point<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        let new_point = func(&(self.0.x, self.0.y));
        Point::new(new_point.0, new_point.1)
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for Line<T> {
    type Output = Line<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        Line::new(self.start.map_coords(func), self.end.map_coords(func))
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for LineString<T> {
    type Output = LineString<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        unsafe {
            LineString::new_unchecked(self.points().iter().map(|p| p.map_coords(func)).collect())
        }
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for Polygon<T> {
    type Output = Polygon<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        Polygon::new(self.exterior.map_coords(func), self.interiors.iter().map(|l| l.map_coords(func)).collect())
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for MultiPoint<T> {
    type Output = MultiPoint<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        MultiPoint(self.0.iter().map(|p| p.map_coords(func)).collect())
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for MultiLineString<T> {
    type Output = MultiLineString<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        MultiLineString(self.0.iter().map(|l| l.map_coords(func)).collect())
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for MultiPolygon<T> {
    type Output = MultiPolygon<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        MultiPolygon(self.0.iter().map(|p| p.map_coords(func)).collect())
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for Geometry<T> {
    type Output = Geometry<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        match *self {
            Geometry::Point(ref x) => Geometry::Point(x.map_coords(func)),
            Geometry::LineString(ref x) => Geometry::LineString(x.map_coords(func)),
            Geometry::Polygon(ref x) => Geometry::Polygon(x.map_coords(func)),
            Geometry::MultiPoint(ref x) => Geometry::MultiPoint(x.map_coords(func)),
            Geometry::MultiLineString(ref x) => Geometry::MultiLineString(x.map_coords(func)),
            Geometry::MultiPolygon(ref x) => Geometry::MultiPolygon(x.map_coords(func)),
            Geometry::GeometryCollection(ref x) => Geometry::GeometryCollection(x.map_coords(func)),
        }
    }
}

impl<T: Float, NT: Float> MapCoords<T, NT> for GeometryCollection<T> {
    type Output = GeometryCollection<NT>;

    fn map_coords(&self, func: &Fn(&(T, T)) -> (NT, NT)) -> Self::Output
    {
        GeometryCollection(self.0.iter().map(|g| g.map_coords(func)).collect())
    }
}


mod test {
    #[allow(unused_imports)]
    
    use super::*;

    #[test]
    fn point() {
        let p = Point::new(10., 10.);
        let new_p = p.map_coords(&|&(x, y)| (x+10., y+100.));
        assert_eq!(new_p.x(), 20.);
        assert_eq!(new_p.y(), 110.);
    }

    #[test]
    fn line() {
        let line = Line::new(Point::new(0., 0.), Point::new(1., 2.));
        assert_eq!(line.map_coords(&|&(x, y)| (x*2., y)),
            Line::new(Point::new(0., 0.), Point::new(2., 2.))
        );
    }

    #[test]
    fn linestring() {
        let line1: LineString<f32> = LineString(vec![Point::new(0., 0.), Point::new(1., 2.)]);
        let line2 = line1.map_coords(&|&(x, y)| (x+10., y-100.));
        assert_eq!(line2.0[0], Point::new(10., -100.));
        assert_eq!(line2.0[1], Point::new(11., -98.));
    }

    #[test]
    fn polygon() {
        let exterior = LineString(vec![Point::new(0., 0.), Point::new(1., 1.),
                                       Point::new(1., 0.), Point::new(0., 0.)]);
        let interiors = vec![LineString(vec![Point::new(0.1, 0.1), Point::new(0.9, 0.9),
                                             Point::new(0.9, 0.1), Point::new(0.1, 0.1)])];
        let p = Polygon::new(exterior, interiors);

        let p2 = p.map_coords(&|&(x, y)| (x+10., y-100.));

        let exterior2 = LineString(vec![Point::new(10., -100.), Point::new(11., -99.),
                                       Point::new(11., -100.), Point::new(10., -100.)]);
        let interiors2 = vec![LineString(vec![Point::new(10.1, -99.9), Point::new(10.9, -99.1),
                                             Point::new(10.9, -99.9), Point::new(10.1, -99.9)])];
        let expected_p2 = Polygon::new(exterior2, interiors2);

        assert_eq!(p2, expected_p2);
    }

    #[test]
    fn multipoint() {
        let p1 = Point::new(10., 10.);
        let p2 = Point::new(0., -100.);
        let mp = MultiPoint(vec![p1, p2]);

        assert_eq!(
            mp.map_coords(&|&(x, y)| (x+10., y+100.)),
            MultiPoint(vec![Point::new(20., 110.), Point::new(10., 0.)])
            );
    }

    #[test]
    fn multilinestring() {
        let line1: LineString<f32> = LineString(vec![Point::new(0., 0.), Point::new(1., 2.)]);
        let line2: LineString<f32> = LineString(vec![Point::new(-1., 0.), Point::new(0., 0.), Point::new(1., 2.)]);
        let mline = MultiLineString(vec![line1, line2]);
        let mline2 = mline.map_coords(&|&(x, y)| (x+10., y-100.));
        assert_eq!(mline2,
            MultiLineString(vec![
                LineString(vec![Point::new(10., -100.), Point::new(11., -98.)]),
                LineString(vec![Point::new(9., -100.), Point::new(10., -100.), Point::new(11., -98.)]),
                ])
            );

    }

    #[test]
    fn multipolygon() {
        let poly1 = Polygon::new(LineString(vec![Point::new(0., 0.), Point::new(10., 0.), Point::new(10., 10.), Point::new(0., 10.), Point::new(0., 0.)]), vec![]);
        let poly2 = Polygon::new(
            LineString(vec![Point::new(11., 11.), Point::new(20., 11.), Point::new(20., 20.), Point::new(11., 20.), Point::new(11., 11.)]),
            vec![
                LineString(vec![Point::new(13., 13.), Point::new(13., 17.), Point::new(17., 17.), Point::new(17., 13.), Point::new(13., 13.)])
            ]);

        let mp = MultiPolygon(vec![poly1, poly2]);
        let mp2 = mp.map_coords(&|&(x, y)| (x*2., y+100.));
        assert_eq!(mp2.0.len(), 2);
        assert_eq!(mp2.0[0],
            Polygon::new(LineString(vec![Point::new(0., 100.), Point::new(20., 100.), Point::new(20., 110.), Point::new(0., 110.), Point::new(0., 100.)]), vec![])
        );
        assert_eq!(mp2.0[1],
           Polygon::new(
            LineString(vec![Point::new(22., 111.), Point::new(40., 111.), Point::new(40., 120.), Point::new(22., 120.), Point::new(22., 111.)]),
            vec![
                LineString(vec![Point::new(26., 113.), Point::new(26., 117.), Point::new(34., 117.), Point::new(34., 113.), Point::new(26., 113.)])
            ])
        );
    }

    #[test]
    fn geometrycollection() {
        let p1 = Geometry::Point(Point::new(10., 10.));
        let line1 = Geometry::LineString(LineString(vec![Point::new(0., 0.), Point::new(1., 2.)]));

        let gc = GeometryCollection(vec![p1, line1]);

        assert_eq!(gc.map_coords(&|&(x, y)| (x+10., y+100.)),
            GeometryCollection(vec![
                Geometry::Point(Point::new(20., 110.)),
                Geometry::LineString(LineString(vec![Point::new(10., 100.), Point::new(11., 102.)])),
                ])
            );
    }
    
    #[test]
    fn convert_type() {
        let p1: Point<f64> = Point::new(1., 2.);
        let p2: Point<f32> = p1.map_coords(&|&(x, y)| (x as f32, y as f32));
        assert_eq!(p2.x(), 1f32);
        assert_eq!(p2.y(), 2f32);
    }

}

