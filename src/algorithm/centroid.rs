use num_traits::{Float, FromPrimitive};

use types::{Point, LineString, Polygon, MultiPolygon, Bbox};
use algorithm::area::Area;
use algorithm::distance::Distance;

/// Calculation of the centroid.
pub trait Centroid<T: Float> {
    /// Calculation of the centroid, see: https://en.wikipedia.org/wiki/Centroid
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
    where T: Float
{
    ///
    /// The Centroid of a LineString is the mean of the middle of the segment
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
            for ps in vect.windows(2) {
                let segment_len = ps[0].distance(&ps[1]);
                let (x1, y1, x2, y2) = (ps[0].x(), ps[0].y(), ps[1].x(), ps[1].y());
                total_length = total_length + segment_len;
                sum_x = sum_x + segment_len * ((x1 + x2) / (T::one() + T::one()));
                sum_y = sum_y + segment_len * ((y1 + y2) / (T::one() + T::one()));
            }
            Some(Point::new(sum_x / total_length, sum_y / total_length))
        }
    }
}

impl<T> Centroid<T> for Polygon<T>
    where T: Float + FromPrimitive
{
    ///
    /// Centroid on a Polygon.
    /// See: https://en.wikipedia.org/wiki/Centroid
    ///
    fn centroid(&self) -> Option<Point<T>> {
        // TODO: consideration of inner polygons;
        let linestring = &self.exterior;
        let vect = &linestring.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            Some(Point::new(vect[0].x(), vect[0].y()))
        } else {
            let area = self.area();
            let mut sum_x = T::zero();
            let mut sum_y = T::zero();
            for ps in vect.windows(2) {
                let tmp = ps[0].x() * ps[1].y() - ps[1].x() * ps[0].y();
                sum_x = sum_x + ((ps[1].x() + ps[0].x()) * tmp);
                sum_y = sum_y + ((ps[1].y() + ps[0].y()) * tmp);
            }
            let six = T::from_i32(6).unwrap();
            Some(Point::new(sum_x / (six * area), sum_y / (six * area)))
        }
    }
}

impl<T> Centroid<T> for MultiPolygon<T>
    where T: Float + FromPrimitive
{
    // See: https://fotino.me/calculating-centroids/
    fn centroid(&self) -> Option<Point<T>> {
        let mut sum_x = T::zero();
        let mut sum_y = T::zero();
        let mut total_area = T::zero();
        let vect = &self.0;
        if vect.is_empty() {
            return None;
        }
        for poly in &self.0 {
            // the area is signed
            let area = poly.area().abs();
            total_area = total_area + area;
            if let Some(p) = poly.centroid() {
                sum_x = sum_x + area * p.x();
                sum_y = sum_y + area * p.y();
            }
        }
        Some(Point::new(sum_x / total_area, sum_y / total_area))
    }
}

impl<T> Centroid<T> for Bbox<T>
    where T: Float
{
    ///
    /// Calculate the Centroid of a Bbox.
    ///
    fn centroid(&self) -> Option<Point<T>> {
        let two = T::one() + T::one();
        Some(Point::new((self.xmax + self.xmin)/two, (self.ymax + self.ymin)/two))
    }
}

#[cfg(test)]
mod test {
    use types::{COORD_PRECISION, Coordinate, Point, LineString, Polygon, MultiPolygon, Bbox};
    use algorithm::centroid::Centroid;
    use algorithm::distance::Distance;
    /// Tests: Centroid of LineString
    #[test]
    fn empty_linestring_test() {
        let vec = Vec::<Point<f64>>::new();
        let linestring = LineString(vec);
        let centroid = linestring.centroid();
        assert!(centroid.is_none());
    }
    #[test]
    fn linestring_one_point_test() {
        let p = Point::new(40.02f64, 116.34);
        let mut vect = Vec::<Point<f64>>::new();
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
    /// Tests: Centroid of Polygon
    #[test]
    fn empty_polygon_test() {
        let v1 = Vec::new();
        let v2 = Vec::new();
        let linestring = LineString::<f64>(v1);
        let poly = Polygon::new(linestring, v2);
        assert!(poly.centroid().is_none());
    }
    #[test]
    fn polygon_one_point_test() {
        let p = Point(Coordinate { x: 2., y: 1. });
        let v = Vec::new();
        let linestring = LineString(vec![p]);
        let poly = Polygon::new(linestring, v);
        assert_eq!(poly.centroid(), Some(p));
    }
    #[test]
    fn polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let v = Vec::new();
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, v);
        assert_eq!(poly.centroid(), Some(p(1., 1.)));
    }
    /// Tests: Centroid of MultiPolygon
    #[test]
    fn empty_multipolygon_polygon_test() {
        assert!(MultiPolygon::<f64>(Vec::new()).centroid().is_none());
    }
    #[test]
    fn multipolygon_one_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly = Polygon::new(linestring, Vec::new());
        assert_eq!(MultiPolygon(vec![poly]).centroid(), Some(p(1., 1.)));
    }
    #[test]
    fn multipolygon_two_polygons_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(2., 1.), p(5., 1.), p(5., 3.), p(2., 3.), p(2., 1.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring = LineString(vec![p(7., 1.), p(8., 1.), p(8., 2.), p(7., 2.), p(7., 1.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        let dist = MultiPolygon(vec![poly1, poly2]).centroid().unwrap().distance(&p(4.07142857142857, 1.92857142857143));
        assert!(dist < COORD_PRECISION);
    }
    #[test]
    fn multipolygon_two_polygons_of_opposite_clockwise_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring = LineString(vec![p(0., 0.), p(-2., 0.), p(-2., 2.), p(0., 2.), p(0., 0.)]);
        let poly2 = Polygon::new(linestring, Vec::new());
        assert_eq!(MultiPolygon(vec![poly1, poly2]).centroid(), Some(p(0., 1.)));
    }
    #[test]
    fn bbox_test() {
        let bbox = Bbox{ xmax: 4., xmin: 0., ymax: 100., ymin: 50.};
        let point = Point(Coordinate { x: 2., y: 75. });
        assert_eq!(point, bbox.centroid().unwrap());
    }
}
