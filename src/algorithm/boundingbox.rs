use num::{Float};

use types::{Bbox, Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon};

/// Calculation of the bounding box of a geometry.

pub trait BoundingBox<T: Float> {
    /// Return a Bounding Box of a geometry
    ///
    /// ```
    /// use geo::{Point, LineString, Coordinate};
    /// use geo::algorithm::boundingbox::BoundingBox;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 116.34));
    /// vec.push(Point::new(42.02f64, 118.34));
    /// let linestring = LineString(vec);
    /// let bbox = linestring.bbox().unwrap();
    ///
    /// println!("Bbox top left coordinates: {}, {}", bbox.xmin, bbox.ymax);
    /// println!("Bbox bottom right coordinates: {}, {}", bbox.xmax, bbox.ymin);
    /// ```
    ///
    fn bbox(&self) -> Option<Bbox<T>>;
}

fn get_bbox<T>(vect: &Vec<Point<T>>) -> Option<Bbox<T>>
    where T: Float
{
    if vect.is_empty() {
        return None;
    }
    if vect.len() == 1 {
        return Some(Bbox{xmin: vect[0].x(), ymax: vect[0].y(),
                         xmax: vect[0].x(), ymin: vect[0].y()})
    } else {
        let (mut xmax, mut xmin) = (T::neg_infinity(), T::infinity());
        let (mut ymax, mut ymin) = (T::neg_infinity(), T::infinity());
        for pnt in vect.iter() {
            let (px, py) = (pnt.x(), pnt.y());
            if px > xmax {
                xmax = px;
            } else if px < xmin {
                xmin = px;
            }
            if py > ymax {
                ymax = py;
            } else if py < ymin {
                ymin = py;
            }
        }
        Some(Bbox{xmin: xmin, ymax: ymax,
                  xmax: xmax, ymin: ymin})
    }
}

impl<T> BoundingBox<T> for MultiPoint<T>
    where T: Float
{
    ///
    /// Return the BoundingBox for a MultiPoint
    ///
    fn bbox(&self) -> Option<Bbox<T>> {
        get_bbox(&self.0)
    }
}

impl<T> BoundingBox<T> for LineString<T>
    where T: Float
{
    ///
    /// Return the BoundingBox for a LineString
    ///
    fn bbox(&self) -> Option<Bbox<T>> {
        get_bbox(&self.0)
    }
}

impl<T> BoundingBox<T> for MultiLineString<T>
    where T: Float
{
    ///
    /// Return the BoundingBox for a MultiLineString
    ///
    fn bbox(&self) -> Option<Bbox<T>> {
        let vect = &self.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            return vect[0].bbox()
        } else {
            let mut bbox = vect[0].bbox().unwrap();
            for geo in vect[1..].iter() {
                let gopt = geo.bbox();
                if gopt.is_some() {
                    bbox += gopt.unwrap();
                }
            }
            Some(bbox)
        }
    }
}

impl<T> BoundingBox<T> for Polygon<T>
    where T: Float
{
    ///
    /// Return the BoundingBox for a Polygon
    ///
    fn bbox(&self) -> Option<Bbox<T>> {
        let line = &self.0;
        get_bbox(&line.0)
    }
}

impl<T> BoundingBox<T> for MultiPolygon<T>
    where T: Float
{
    ///
    /// Return the BoundingBox for a MultiLineString
    ///
    fn bbox(&self) -> Option<Bbox<T>> {
        let vect = &self.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            return vect[0].bbox()
        } else {
            let mut bbox = vect[0].bbox().unwrap();
            for geo in vect[1..].iter() {
                let gopt = geo.bbox();
                if gopt.is_some() {
                    bbox += gopt.unwrap();
                }
            }
            Some(bbox)
        }
    }
}



#[cfg(test)]
mod test {
    use types::{Bbox, Coordinate, Point, MultiPoint, LineString, MultiLineString, Polygon, MultiPolygon};
    use algorithm::boundingbox::BoundingBox;

    #[test]
    fn empty_linestring_test() {
        let vect = Vec::<Point<f64>>::new();
        let linestring = LineString(vect);
        let bbox = linestring.bbox();
        assert!(bbox.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let p = Point::new(40.02f64, 116.34);
        let mut vect = Vec::<Point<f64>>::new();
        vect.push(p);
        let linestring = LineString(vect);
        let bbox = Bbox{xmin: 40.02f64, ymax: 116.34, xmax: 40.02, ymin: 116.34};
        assert_eq!(bbox, linestring.bbox().unwrap());
    }
    #[test]
    fn linestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(1., 1.), p(2., -2.), p(-3., -3.), p(-4., 4.)]);
        let bbox = Bbox{xmin: -4., ymax: 4., xmax: 2., ymin: -3.};
        assert_eq!(bbox, linestring.bbox().unwrap());
    }
    #[test]
    fn multilinestring_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let multiline = MultiLineString(vec![LineString(vec![p(1., 1.), p(-40., 1.)]),
                                             LineString(vec![p(1., 1.), p(50., 1.)]),
                                             LineString(vec![p(1., 1.), p(1., -60.)]),
                                             LineString(vec![p(1., 1.), p(1., 70.)])]);
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
        let linestring = LineString(vec![p(0., 0.), p(5., 0.), p(5., 6.), p(0., 6.), p(0., 0.)]);
        let line_bbox = linestring.bbox().unwrap();
        let poly = Polygon(linestring, Vec::new());
        assert_eq!(line_bbox, poly.bbox().unwrap());
    }
    #[test]
    fn multipolygon_test(){
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let mpoly = MultiPolygon(vec![Polygon(LineString(vec![p(0., 0.), p(50., 0.), p(0., -70.), p(0., 0.)]), Vec::new()),
                                      Polygon(LineString(vec![p(0., 0.), p(5., 0.), p(0., 80.), p(0., 0.)]), Vec::new()),
                                      Polygon(LineString(vec![p(0., 0.), p(-60., 0.), p(0., 6.), p(0., 0.)]), Vec::new()),
                                      ]);
        let bbox = Bbox{xmin: -60., ymax: 80., xmax: 50., ymin: -70.};
        assert_eq!(bbox, mpoly.bbox().unwrap());
    }
}
