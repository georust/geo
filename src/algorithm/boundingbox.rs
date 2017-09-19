use num_traits::Float;

use types::{Bbox, Point, MultiPoint, Line, LineString, MultiLineString, Polygon, MultiPolygon};

/// Calculation of the bounding box of a geometry.

pub trait BoundingBox<T: Float> {
    type Output;

    /// Return the Bounding Box of a geometry
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::boundingbox::BoundingBox;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 118.34));
    /// let linestring = LineString::new(vec).unwrap();
    /// let bbox = linestring.bbox().unwrap();
    ///
    /// assert_eq!(40.02f64, bbox.xmin);
    /// assert_eq!(42.02f64, bbox.xmax);
    /// assert_eq!(116.34, bbox.ymin);
    /// assert_eq!(118.34, bbox.ymax);
    /// ```
    ///
    fn bbox(&self) -> Self::Output;
}


fn get_min_max<T>(p: T, min: T, max: T) -> (T, T)
    where T: Float
{
    if p > max {(min, p)} else if p < min {(p, max)} else {(min, max)}
}

fn get_bbox<'a, I, T>(collection: I) -> Option<Bbox<T>>
    where T: 'a + Float,
          I: 'a + IntoIterator<Item = &'a Point<T>>
{
    let mut iter  = collection.into_iter();
    if let Some(pnt) = iter.next() {
        let mut xrange = (pnt.x(), pnt.x());
        let mut yrange = (pnt.y(), pnt.y());
        for pnt in iter {
            let (px, py) = (pnt.x(), pnt.y());
            xrange = get_min_max(px, xrange.0, xrange.1);
            yrange = get_min_max(py, yrange.0, yrange.1);
        }
        return Some(Bbox{xmin: xrange.0, xmax: xrange.1,
                         ymin: yrange.0, ymax: yrange.1})
    }
    None
}


impl<T> BoundingBox<T> for MultiPoint<T>
    where T: Float
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a MultiPoint
    ///
    fn bbox(&self) -> Self::Output {
        get_bbox(&self.0)
    }
}

impl<T> BoundingBox<T> for Line<T>
    where T: Float
{
    type Output = Bbox<T>;

    fn bbox(&self) -> Self::Output {
        let a = self.start;
        let b = self.end;
        let (xmin, xmax) = if a.x() <= b.x() {(a.x(), b.x())} else {(b.x(), a.x())};
        let (ymin, ymax) = if a.y() <= b.y() {(a.y(), b.y())} else {(b.y(), a.y())};
        Bbox {xmin: xmin, xmax: xmax,
              ymin: ymin, ymax: ymax}
    }
}

impl<T> BoundingBox<T> for LineString<T>
    where T: Float
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a LineString
    ///
    fn bbox(&self) -> Self::Output {
        // TODO this should return a Bbox
        get_bbox(self.points())
    }
}

impl<T> BoundingBox<T> for MultiLineString<T>
    where T: Float
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a MultiLineString
    ///
    fn bbox(&self) -> Self::Output {
        get_bbox(self.0.iter().flat_map(|line| line.points().iter()))
    }
}

impl<T> BoundingBox<T> for Polygon<T>
    where T: Float
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a Polygon
    ///
    fn bbox(&self) -> Self::Output {
        let line = &self.exterior;
        get_bbox(line.points())
    }
}

impl<T> BoundingBox<T> for MultiPolygon<T>
    where T: Float
{
    type Output = Option<Bbox<T>>;

    ///
    /// Return the BoundingBox for a MultiPolygon
    ///
    fn bbox(&self) -> Self::Output {
        get_bbox(self.0.iter().flat_map(|poly| poly.exterior.points().iter()))
    }
}



#[cfg(test)]
mod test {
    use types::{Bbox, Coordinate, Point, MultiPoint, Line, LineString, MultiLineString, Polygon, MultiPolygon};
    use algorithm::boundingbox::BoundingBox;

    #[test]
    fn empty_linestring_test() {
        let vect = Vec::<Point<f64>>::new();
        let linestring = LineString::new(vect).unwrap();
        let bbox = linestring.bbox();
        assert!(bbox.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let p = Point::new(40.02f64, 116.34);
        let mut vect = Vec::<Point<f64>>::new();
        vect.push(p);
        let linestring = LineString::new(vect).unwrap();
        let bbox = Bbox{xmin: 40.02f64, ymax: 116.34, xmax: 40.02, ymin: 116.34};
        assert_eq!(bbox, linestring.bbox().unwrap());
    }
    #[test]
    fn linestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString::new(vec![p(1., 1.), p(2., -2.), p(-3., -3.), p(-4., 4.)]).unwrap();
        let bbox = Bbox{xmin: -4., ymax: 4., xmax: 2., ymin: -3.};
        assert_eq!(bbox, linestring.bbox().unwrap());
    }
    #[test]
    fn multilinestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let multiline = MultiLineString::new(vec![LineString::new(vec![p(1., 1.), p(-40., 1.)]).unwrap(),
                                             LineString::new(vec![p(1., 1.), p(50., 1.)]).unwrap(),
                                             LineString::new(vec![p(1., 1.), p(1., -60.)]).unwrap(),
                                             LineString::new(vec![p(1., 1.), p(1., 70.)])]).unwrap();
        let bbox = Bbox{xmin: -40., ymax: 70., xmax: 50., ymin: -60.};
        assert_eq!(bbox, multiline.bbox().unwrap());
    }
    #[test]
    fn multipoint_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let multipoint = MultiPoint(vec![p(1., 1.), p(2., -2.), p(-3., -3.), p(-4., 4.)]);
        let bbox = Bbox{xmin: -4., ymax: 4., xmax: 2., ymin: -3.};
        assert_eq!(bbox, multipoint.bbox().unwrap());
    }
    #[test]
    fn polygon_test(){
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString::new(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]).unwrap();
        let line_bbox = linestring.bbox().unwrap();
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(line_bbox, poly.bbox().unwrap());
    }
    #[test]
    fn multipolygon_test(){
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mpoly = MultiPolygon(vec![Polygon::new(LineString::new(vec![p(0., 0.), p(50., 0.), p(0., -70.), p(0., 0.)]), Vec::new()).unwrap(),
                                      Polygon::new(LineString::new(vec![p(0., 0.), p(5., 0.), p(0., 80.), p(0., 0.)]), Vec::new()).unwrap(),
                                      Polygon::new(LineString::new(vec![p(0., 0.), p(-60., 0.), p(0., 6.), p(0., 0.)]), Vec::new()).unwrap(),
                                      ]);
        let bbox = Bbox{xmin: -60., ymax: 80., xmax: 50., ymin: -70.};
        assert_eq!(bbox, mpoly.bbox().unwrap());
    }
    #[test]
    fn line_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let line1 = Line::new(p(0., 1.), p(2., 3.));
        let line2 = Line::new(p(2., 3.), p(0., 1.));
        assert_eq!(line1.bbox(), Bbox {xmin: 0., xmax: 2., ymin: 1., ymax: 3.});
        assert_eq!(line2.bbox(), Bbox {xmin: 0., xmax: 2., ymin: 1., ymax: 3.});
    }
}
