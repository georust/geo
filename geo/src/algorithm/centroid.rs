use num_traits::{Float, FromPrimitive};

use ::{Bbox, Line, LineString, MultiPolygon, Point, Polygon};
use algorithm::area::Area;
use algorithm::euclidean_length::EuclideanLength;

/// Calculation of the centroid.
pub trait Centroid<T: Float> {
    type Output;

    /// See: https://en.wikipedia.org/wiki/Centroid
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::centroid::Centroid;
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(40.02f64, 116.34));
    /// vec.push(Point::new(40.02f64, 118.23));
    /// let linestring = LineString(vec);
    ///
    /// assert_eq!(linestring.centroid().unwrap(), Point::new(40.02, 117.285));
    /// ```
    ///
    fn centroid(&self) -> Self::Output;
}

// Calculation of simple (no interior holes) Polygon area
fn simple_polygon_area<T>(linestring: &LineString<T>) -> T
where
    T: Float,
{
    if linestring.0.is_empty() || linestring.0.len() == 1 {
        return T::zero();
    }
    let mut tmp = T::zero();
    for line in linestring.lines() {
        tmp = tmp + line.determinant();
    }
    tmp / (T::one() + T::one())
}

// Calculation of a Polygon centroid without interior rings
fn simple_polygon_centroid<T>(poly_ext: &LineString<T>) -> Option<Point<T>>
where
    T: Float + FromPrimitive,
{
    let area = simple_polygon_area(poly_ext);
    if area == T::zero() {
        // if the polygon is flat (area = 0), it is considered as a linestring
        return poly_ext.centroid();
    }
    let mut sum_x = T::zero();
    let mut sum_y = T::zero();
    for line in poly_ext.lines() {
        let tmp = line.determinant();
        sum_x = sum_x + ((line.end.x() + line.start.x()) * tmp);
        sum_y = sum_y + ((line.end.y() + line.start.y()) * tmp);
    }
    let six = T::from_i32(6).unwrap();
    Some(Point::new(sum_x / (six * area), sum_y / (six * area)))
}

impl<T> Centroid<T> for Line<T>
where
    T: Float,
{
    type Output = Point<T>;

    fn centroid(&self) -> Self::Output {
        let two = T::one() + T::one();
        let x = self.start.x() + self.dx() / two;
        let y = self.start.y() + self.dy() / two;
        Point::new(x, y)
    }
}

impl<T> Centroid<T> for LineString<T>
where
    T: Float,
{
    type Output = Option<Point<T>>;

    // The Centroid of a LineString is the mean of the middle of the segment
    // weighted by the length of the segments.
    fn centroid(&self) -> Self::Output {
        if self.0.is_empty() {
            return None;
        }
        if self.0.len() == 1 {
            Some(self.0[0])
        } else {
            let mut sum_x = T::zero();
            let mut sum_y = T::zero();
            let mut total_length = T::zero();
            for line in self.lines() {
                let segment_len = line.euclidean_length();
                let line_center = line.centroid();
                total_length = total_length + segment_len;
                sum_x = sum_x + segment_len * line_center.x();
                sum_y = sum_y + segment_len * line_center.y();
            }
            Some(Point::new(sum_x / total_length, sum_y / total_length))
        }
    }
}

impl<T> Centroid<T> for Polygon<T>
where
    T: Float + FromPrimitive,
{
    type Output = Option<Point<T>>;

    // Calculate the centroid of a Polygon.
    // We distinguish between a simple polygon, which has no interior holes,
    // and a complex polygon, which has one or more interior holes.
    // A complex polygon's centroid is the weighted average of its
    // exterior shell centroid and the centroids of the interior ring(s),
    // which are both considered simple polygons for the purposes of
    // this calculation.
    // See here for a formula: http://math.stackexchange.com/a/623849
    // See here for detail on alternative methods: https://fotino.me/calculating-centroids/
    fn centroid(&self) -> Self::Output {
        let linestring = &self.exterior;
        let vect = &linestring.0;
        if vect.is_empty() {
            return None;
        }
        if vect.len() == 1 {
            Some(Point::new(vect[0].x(), vect[0].y()))
        } else {
            let external_centroid = simple_polygon_centroid(&self.exterior)?;
            if !self.interiors.is_empty() {
                let external_area = simple_polygon_area(&self.exterior).abs();
                // accumulate interior Polygons
                let (totals_x, totals_y, internal_area) = self.interiors
                    .iter()
                    .filter_map(|ring| {
                        let area = simple_polygon_area(ring).abs();
                        let centroid = simple_polygon_centroid(ring)?;
                        Some((centroid.x() * area, centroid.y() * area, area))
                    })
                    .fold((T::zero(), T::zero(), T::zero()), |accum, val| {
                        (accum.0 + val.0, accum.1 + val.1, accum.2 + val.2)
                    });
                return Some(Point::new(
                    ((external_centroid.x() * external_area) - totals_x) / (external_area - internal_area),
                    ((external_centroid.y() * external_area) - totals_y) / (external_area - internal_area),
                ));
            }
            Some(external_centroid)
        }
    }
}

impl<T> Centroid<T> for MultiPolygon<T>
where
    T: Float + FromPrimitive,
{
    type Output = Option<Point<T>>;

    fn centroid(&self) -> Self::Output {
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
where
    T: Float,
{
    type Output = Point<T>;

    fn centroid(&self) -> Self::Output {
        let two = T::one() + T::one();
        Point::new((self.xmax + self.xmin) / two, (self.ymax + self.ymin) / two)
    }
}

impl<T> Centroid<T> for Point<T>
where
    T: Float,
{
    type Output = Point<T>;

    fn centroid(&self) -> Self::Output {
        Point::new(self.x(), self.y())
    }
}

#[cfg(test)]
mod test {
    use ::{Bbox, Coordinate, Line, LineString, MultiPolygon, Point, Polygon, COORD_PRECISION};
    use algorithm::centroid::Centroid;
    use algorithm::euclidean_distance::EuclideanDistance;
    // Tests: Centroid of LineString
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
        assert_eq!(
            linestring.centroid(),
            Some(Point(Coordinate { x: 6., y: 1. }))
        );
    }
    // Tests: Centroid of Polygon
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
    #[test]
    fn polygon_hole_test() {
        let ls1 = LineString(vec![
            Point::new(5.0, 1.0),
            Point::new(4.0, 2.0),
            Point::new(4.0, 3.0),
            Point::new(5.0, 4.0),
            Point::new(6.0, 4.0),
            Point::new(7.0, 3.0),
            Point::new(7.0, 2.0),
            Point::new(6.0, 1.0),
            Point::new(5.0, 1.0),
        ]);

        let ls2 = LineString(vec![
            Point::new(5.0, 1.3),
            Point::new(5.5, 2.0),
            Point::new(6.0, 1.3),
            Point::new(5.0, 1.3),
        ]);

        let ls3 = LineString(vec![
            Point::new(5., 2.3),
            Point::new(5.5, 3.0),
            Point::new(6., 2.3),
            Point::new(5., 2.3),
        ]);

        let p1 = Polygon::new(ls1, vec![ls2, ls3]);
        let centroid = p1.centroid().unwrap();
        assert_eq!(centroid, Point::new(5.5, 2.5518518518518514));
    }
    #[test]
    fn flat_polygon_test() {
        let p = |x| Point(Coordinate { x: x, y: 1. });
        let poly = Polygon::new(LineString(vec![p(0.), p(1.), p(0.)]), vec![]);
        assert_eq!(
            poly.centroid(),
            Some(p(0.5))
        );
    }
    #[test]
    fn polygon_flat_interior_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(LineString(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]), 
                    vec![LineString(vec![p(0., 0.), p(0., 1.), p(0., 0.)])]);
        assert_eq!(
            poly.centroid(),
            Some(p(0.5, 0.5))
        );
    }
    #[test]
    fn empty_interior_polygon_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let poly = Polygon::new(LineString(vec![p(0., 0.), p(0., 1.), p(1., 1.), p(1., 0.), p(0., 0.)]), 
                    vec![LineString(vec![])]);
        assert_eq!(
            poly.centroid(),
            Some(p(0.5, 0.5))
        );
    }
    // Tests: Centroid of MultiPolygon
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
        let dist = MultiPolygon(vec![poly1, poly2])
            .centroid()
            .unwrap()
            .euclidean_distance(&p(4.07142857142857, 1.92857142857143));
        assert!(dist < COORD_PRECISION);
    }
    #[test]
    fn multipolygon_two_polygons_of_opposite_clockwise_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let linestring = LineString(vec![p(0., 0.), p(2., 0.), p(2., 2.), p(0., 2.), p(0., 0.)]);
        let poly1 = Polygon::new(linestring, Vec::new());
        let linestring = LineString(vec![
            p(0., 0.),
            p(-2., 0.),
            p(-2., 2.),
            p(0., 2.),
            p(0., 0.),
        ]);
        let poly2 = Polygon::new(linestring, Vec::new());
        assert_eq!(MultiPolygon(vec![poly1, poly2]).centroid(), Some(p(0., 1.)));
    }
    #[test]
    fn bbox_test() {
        let bbox = Bbox {
            xmax: 4.,
            xmin: 0.,
            ymax: 100.,
            ymin: 50.,
        };
        let point = Point(Coordinate { x: 2., y: 75. });
        assert_eq!(point, bbox.centroid());
    }
    #[test]
    fn line_test() {
        let p = |x, y| Point(Coordinate { x: x, y: y });
        let line1 = Line::new(p(0., 1.), p(1., 3.));
        assert_eq!(line1.centroid(), p(0.5, 2.));
    }
}
