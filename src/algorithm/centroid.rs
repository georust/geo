use num::{Float, ToPrimitive};

use types::{Point, LineString, Polygon};

/// Calculation of the centroid.

pub trait Centroid<T: Float> {
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
    fn centroid(&self) -> Option<Point<T>>;
}

impl<T> Centroid<T> for LineString<T>
    where T: Float + ToPrimitive
{
    ///
    /// Centroid on a LineString is the mean of the middle of the segment
    /// weighted by the length of the segments.
    ///
    fn centroid(&self) -> Option<Point<T>> {
        let vect = &self.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            Some(Point::new(vect[0].x(),
                            vect[0].y()))
        } else {
            let mut sum_x = T::zero();
            let mut sum_y = T::zero();
            let mut total_length = T::zero();
            for (p1, p2) in vect.iter().zip(vect[1..].iter()) {
                let segment_len = p1.distance_to(&p2);
                let (x1, y1, x2, y2) = (p1.x(), p1.y(), p2.x(), p2.y());
                total_length = total_length + segment_len;
                sum_x = sum_x + segment_len * ((x1 + x2) / (T::one() + T::one()));
                sum_y = sum_y + segment_len * ((y1 + y2) / (T::one() + T::one()));
            }
            Some(Point::new(sum_x / total_length, sum_y / total_length))
        }
    }
}

impl<T> Centroid<T> for Polygon<T>
    where T: Float + ToPrimitive
{
    ///
    /// Centroid on a Polygon.
    /// See: https://en.wikipedia.org/wiki/Centroid
    ///
    fn centroid(&self) -> Option<Point<T>> {
        // TODO: consideration of inner polygons;
        let linestring = &self.0;
        let vect = &linestring.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            Some(Point::new(vect[0].x(), vect[0].y()))
        } else {
            let mut area = T::zero();
            let mut sum_x = T::zero();
            let mut sum_y = T::zero();
            for (p1, p2) in vect.iter().zip(vect[1..].iter()) {
                let (x1, y1, x2, y2) = (p1.x(), p1.y(), p2.x(), p2.y());
                let tmp = x1 * y2 - x2 * y1;
                area = area + tmp;
                sum_x = sum_x + (x1 + x2) * tmp;
                sum_y = sum_y + (y2 + y1) * tmp;
            }
            area = area / (T::one() + T::one());
            let six = T::one() + T::one() + T::one() + T::one() + T::one() + T::one();
            Some(Point::new(sum_x / (six * area), (sum_y / (six * area))))
        }
    }
}

#[cfg(test)]
mod test {
    use types::{Coordinate, Point, LineString, Polygon};
    use algorithm::centroid::Centroid;
    #[test]
    fn empty_linestring_test() {
        let vec : Vec<Point<f64>> = Vec::new();
        let linestring : LineString<f64> = LineString(vec);
        let centroid = linestring.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let p = Point::new(40.02f64, 116.34);
        let mut vect : Vec<Point<f64>> = Vec::new();
        vect.push(p);
        let linestring : LineString<f64> = LineString(vect);
        let centroid = linestring.centroid();
        assert_eq!(centroid, Some(p));
    }
    #[test]
    fn linestring_test() {
        let p = |x| Point(Coordinate { x: x, y: 1. });
        let linestring : LineString<f64> = LineString(vec![p(1.), p(7.), p(8.), p(9.), p(10.), p(11.)]);
        assert_eq!(linestring.centroid(),
                   Some(Point(Coordinate { x: 6., y: 1. })));
    }
    #[test]
    fn empty_polygon_test() {
        let v1 = Vec::new();
        let v2 = Vec::new();
        let linestring : LineString<f64> = LineString(v1);
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
