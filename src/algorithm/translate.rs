use num_traits::{Float, FromPrimitive};
use types::{Point, Polygon, LineString, Line, MultiPoint, MultiPolygon, MultiLineString};

// translate a slice of points by the given offsets
fn translation_matrix<T>(x_offset: T, y_offset: T, points: &[Point<T>]) -> Vec<Point<T>>
    where T: Float
{
    points
        .iter()
        .map(|point| {
                 let new_x = point.x() + x_offset;
                 let new_y = point.y() + y_offset;
                 Point::new(new_x, new_y)
             })
        .collect::<Vec<_>>()
}

pub trait Translate<T> {
    /// Translate a Geometry along its axes by the given offsets
    ///
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::translate::{Translate};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(0.0, 0.0));
    /// vec.push(Point::new(5.0, 5.0));
    /// vec.push(Point::new(10.0, 10.0));
    /// let linestring = LineString(vec);
    /// let translated = linestring.translate(1.5, 3.5);
    /// let mut correct = Vec::new();
    /// correct.push(Point::new(1.5, 3.5));
    /// correct.push(Point::new(6.5, 8.5));
    /// correct.push(Point::new(11.5, 13.5));
    /// let correct_ls = LineString(correct);
    /// assert_eq!(translated, correct_ls);
    /// ```
    fn translate(&self, xoff: T, yoff: T) -> Self where T: Float;
}

impl<T> Translate<T> for Point<T>
    where T: Float
{
    /// Translate the Point by the given offsets
    fn translate(&self, xoff: T, yoff: T) -> Self {
        Point::new(self.x() + xoff, self.y() + yoff)
    }
}

impl<T> Translate<T> for LineString<T>
    where T: Float
{
    /// Translate the LineString by the given offsets
    fn translate(&self, xoff: T, yoff: T) -> Self {
        LineString(translation_matrix(xoff, yoff, &self.0))
    }
}

impl<T> Translate<T> for Polygon<T>
    where T: Float + FromPrimitive
{
    /// Translate the Polygon by the given offsets
    fn translate(&self, xoff: T, yoff: T) -> Self {
        Polygon::new(LineString(translation_matrix(xoff, yoff, &self.exterior.0)),
                     self.interiors
                         .iter()
                         .map(|ring| LineString(translation_matrix(xoff, yoff, &ring.0)))
                         .collect())
    }
}

impl<T> Translate<T> for Line<T>
    where T: Float
{
    fn translate(&self, xoff: T, yoff: T) -> Self {
        let translated = translation_matrix(xoff, yoff, &vec![self.start, self.end]);
        Line::new(translated[0], translated[1])
    }
}

impl<T> Translate<T> for MultiPolygon<T>
    where T: Float + FromPrimitive
{
    fn translate(&self, xoff: T, yoff: T) -> Self {
        MultiPolygon(self.0.iter().map(|poly| poly.translate(xoff, yoff)).collect())
    }
}

impl<T> Translate<T> for MultiLineString<T>
    where T: Float + FromPrimitive
{
    fn translate(&self, xoff: T, yoff: T) -> Self {
        MultiLineString(self.0.iter().map(|ls| ls.translate(xoff, yoff)).collect())
    }
}

impl<T> Translate<T> for MultiPoint<T>
    where T: Float + FromPrimitive
{
    fn translate(&self, xoff: T, yoff: T) -> Self {
        MultiPoint(self.0.iter().map(|p| p.translate(xoff, yoff)).collect())
    }
}

#[cfg(test)]
mod test {
    use types::{Point, LineString, Polygon};
    use super::*;
    #[test]
    fn test_translate_point() {
        let p = Point::new(1.0, 5.0);
        let translated = p.translate(30.0, 20.0);
        assert_eq!(translated, Point::new(31.0, 25.0));
    }
    #[test]
    fn test_translate_linestring() {
        let mut vec = Vec::new();
        vec.push(Point::new(0.0, 0.0));
        vec.push(Point::new(5.0, 1.0));
        vec.push(Point::new(10.0, 0.0));
        let linestring = LineString(vec);
        let translated = linestring.translate(17.0, 18.0);
        let mut correct = Vec::new();
        correct.push(Point::new(17.0, 18.0));
        correct.push(Point::new(22.0, 19.0));
        correct.push(Point::new(27., 18.));
        let correct_ls = LineString(correct);
        assert_eq!(translated, correct_ls);
    }
    #[test]
    fn test_translate_polygon() {
        let points_raw = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.),
                              (7., 2.), (6., 1.), (5., 1.)];
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let translated = poly1.translate(17.0, 18.0);
        let correct_outside = vec![(22.0, 19.0),
                                 (21.0, 20.0),
                                 (21.0, 21.0),
                                 (22.0, 22.0),
                                 (23.0, 22.0),
                                 (24.0, 21.0),
                                 (24.0, 20.0),
                                 (23.0, 19.0),
                                 (22.0, 19.0)];
        let correct = Polygon::new(LineString(correct_outside
                                                  .iter()
                                                  .map(|e| Point::new(e.0, e.1))
                                                  .collect::<Vec<_>>()),
                                   vec![]);
        // results agree with Shapely / GEOS
        assert_eq!(translated, correct);
    }
    #[test]
    fn test_rotate_polygon_holes() {
        let ls1 = LineString(vec![Point::new(5.0, 1.0),
                                  Point::new(4.0, 2.0),
                                  Point::new(4.0, 3.0),
                                  Point::new(5.0, 4.0),
                                  Point::new(6.0, 4.0),
                                  Point::new(7.0, 3.0),
                                  Point::new(7.0, 2.0),
                                  Point::new(6.0, 1.0),
                                  Point::new(5.0, 1.0)]);

        let ls2 = LineString(vec![Point::new(5.0, 1.3),
                                  Point::new(5.5, 2.0),
                                  Point::new(6.0, 1.3),
                                  Point::new(5.0, 1.3)]);

        let poly1 = Polygon::new(ls1, vec![ls2]);
        let rotated = poly1.translate(17.0, 18.0);
        let correct_outside = vec![(22.0, 19.0),
                                    (21.0, 20.0),
                                    (21.0, 21.0),
                                    (22.0, 22.0),
                                    (23.0, 22.0),
                                    (24.0, 21.0),
                                    (24.0, 20.0),
                                    (23.0, 19.0),
                                    (22.0, 19.0)]
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
        let correct_inside = vec![(22.0, 19.3), (22.5, 20.0), (23.0, 19.3), (22.0, 19.3)]
                .iter()
                .map(|e| Point::new(e.0, e.1))
                .collect::<Vec<_>>();
        assert_eq!(rotated.exterior.0, correct_outside);
        assert_eq!(rotated.interiors[0].0, correct_inside);
    }
}
