
pub use types::{Coordinate, Point, LineString, Polygon};

/// Calculation of the centroid.

pub trait Centroid {
    /// Calculation the centroid, see: https://en.wikipedia.org/wiki/Centroid
    ///
    /// ```
    /// use geo::{Point, LineString, Coordinate};
    /// use geo::algorithm::centroid::Centroid;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(40.02f64, 116.34));
    /// let linestring = LineString(vec);
    ///
    /// println!("Centroid {:?}", linestring.centroid());
    /// ```
    ///
    fn centroid(&self) -> Option<Point>;
}

impl Centroid for LineString {
    ///
    /// Centroid on a LineString is the mean of the middle of the segment
    /// weighted by the length of the segments.
    ///
    fn centroid(&self) -> Option<Point> {
        let vect = &self.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            Some(Point::new(vect[0].lng(), vect[0].lat()))
        } else {
            let mut sum_x = 0.;
            let mut sum_y = 0.;
            let mut total_length = 0.;
            for (p1, p2) in vect.iter().zip(vect[1..].iter()) {
                let segment_len = p1.distance_to(&p2);
                total_length += segment_len;
                sum_x += segment_len * ((p1.lng() + p2.lng()) / 2.);
                sum_y += segment_len * ((p1.lat() + p2.lat()) / 2.);
            }
            Some(Point::new(sum_x / total_length, sum_y / total_length))
        }
    }
}

impl Centroid for Polygon {
    ///
    /// Centroid on a Polygon.
    /// See: https://en.wikipedia.org/wiki/Centroid
    ///
    fn centroid(&self) -> Option<Point> {
        // TODO: consideration of inner polygons;
        let linestring = &self.0;
        let vect = &linestring.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            Some(Point::new(vect[0].lng(), vect[0].lat()))
        } else {
            let mut area = 0.;
            let mut sum_x = 0.;
            let mut sum_y = 0.;
            for (p1, p2) in vect.iter().zip(vect[1..].iter()) {
                let tmp = p1.lng() * p2.lat() - p2.lng() * p1.lat();
                area += tmp;
                sum_x += (p2.lng() + p1.lng()) * tmp;
                sum_y += (p2.lat() + p1.lat()) * tmp;
            }
            area /= 2.;
            Some(Point::new(sum_x / (6. * area), sum_y / (6. * area)))
        }
    }
}

#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon};
    use algorithm::centroid::Centroid;
    #[test]
    fn empty_linestring_test() {
        let vec = Vec::new();
        let linestring = LineString(vec);
        let centroid = linestring.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let p = Point::new(40.02f64, 116.34);
        let mut vect = Vec::new();
        vect.push(p);
        let linestring = LineString(vect);
        let centroid = linestring.centroid();
        assert_eq!(centroid, Some(p));
    }
    #[test]
    fn linestring_test() {
        let p = |x| Point(Coordinate { x: x, y: 1. });
        let linestring = LineString(vec![p(1.), p(7.), p(8.), p(9.), p(10.), p(11.)]);
        assert_eq!(linestring.centroid(),
                   Some(Point(Coordinate { x: 6., y: 1. })));
    }
    #[test]
    fn empty_polygon_test() {
        let v1 = Vec::new();
        let v2 = Vec::new();
        let linestring = LineString(v1);
        let poly = Polygon(linestring, v2);
        assert!(poly.centroid().is_none());
    }
    #[test]
    fn polygon_one_point_test() {
        let p = Point(Coordinate { x: 2., y: 1. });
        let v = Vec::new();
        let linestring = LineString(vec![p]);
        let poly = Polygon(linestring, v);
        assert_eq!(poly.centroid(), Some(p));
    }
    #[test]
    fn polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon(linestring, v);
        assert_eq!(poly.centroid(), Some(p(1., 1.)));
    }
}
