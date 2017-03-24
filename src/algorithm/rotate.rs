use num_traits::Float;
use types::{Point, Polygon, LineString};

// rotate a slice of points "angle" degrees about an origin
// origin can be an arbitrary point, pass &Point::new(0., 0.)
// for the actual origin
fn rotation_matrix<T>(angle: T, origin: &Point<T>, points: &[Point<T>]) -> Vec<Point<T>>
    where T: Float
{
    let cos_theta = angle.to_radians().cos();
    let sin_theta = angle.to_radians().sin();
    let x0 = origin.x();
    let y0 = origin.y();
    points.iter()
        .map(|point| {
                 let x = point.x() - x0;
                 let y = point.y() - y0;
                 Point::new(x * cos_theta - y * sin_theta + x0,
                            x * sin_theta + y * cos_theta + y0)
             })
        .collect::<Vec<_>>()
}

pub trait Rotate<T> {
    /// Rotate a Geometry around a given origin by an angle, in degrees
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::rotate::{Rotate};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(0.0, 0.0));
    /// vec.push(Point::new(5.0, 5.0));
    /// vec.push(Point::new(10.0, 10.0));
    /// let linestring = LineString(vec);
    /// let rotated = linestring.rotate(-45.0, &Point::new(5.0, 5.0));
    /// let mut correct = Vec::new();
    /// correct.push(Point::new(-2.0710678118654755, 5.0));
    /// correct.push(Point::new(5.0, 5.0));
    /// correct.push(Point::new(12.071067811865476, 5.0));
    /// let correct_ls = LineString(correct);
    /// assert_eq!(rotated, correct_ls);
    /// ```
    fn rotate(&self, angle: T, origin: &Point<T>) -> Self where T: Float;
}

impl<T> Rotate<T> for Point<T>
    where T: Float
{
    /// Rotate the Point about the origin by the given number of degrees
    fn rotate(&self, angle: T, origin: &Point<T>) -> Point<T> {
        rotation_matrix(angle, origin, &[*self])[0]
    }
}

impl<T> Rotate<T> for LineString<T>
    where T: Float
{
    /// Rotate the LineString about the origin by the given number of degrees
    fn rotate(&self, angle: T, origin: &Point<T>) -> LineString<T> {
        LineString(rotation_matrix(angle, origin, &self.0))
    }
}

impl<T> Rotate<T> for Polygon<T>
    where T: Float
{
    /// Rotate the Polygon about the origin by the given number of degrees
    fn rotate(&self, angle: T, origin: &Point<T>) -> Polygon<T> {
        Polygon::new(LineString(rotation_matrix(angle, origin, &self.exterior.0)),
                     self.interiors
                         .iter()
                         .map(|ring| ring.rotate(angle, origin))
                         .collect())
    }
}

#[cfg(test)]
mod test {
    use types::{Point, LineString, Polygon};
    use super::*;
    #[test]
    fn test_rotate_point() {
        let p = Point::new(1.0, 5.0);
        let rotated = p.rotate(30.0, &Point::new(0.0, 0.0));
        // results agree with Shapely / GEOS
        assert_eq!(rotated, Point::new(-1.6339745962155607, 4.830127018922194));
    }
    #[test]
    fn test_rotate_linestring() {
        let mut vec = Vec::new();
        vec.push(Point::new(0.0, 0.0));
        vec.push(Point::new(5.0, 5.0));
        vec.push(Point::new(10.0, 10.0));
        let linestring = LineString(vec);
        let rotated = linestring.rotate(-45.0, &Point::new(5.0, 5.0));
        let mut correct = Vec::new();
        correct.push(Point::new(-2.0710678118654755, 5.0));
        correct.push(Point::new(5.0, 5.0));
        correct.push(Point::new(12.071067811865476, 5.0));
        let correct_ls = LineString(correct);
        // results agree with Shapely / GEOS
        assert_eq!(rotated, correct_ls);
    }
    #[test]
    fn test_rotate_polygon() {
        let points_raw = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.),
                              (7., 2.), (6., 1.), (5., 1.)];
        let points = points_raw.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
        let interior = vec![(5., 1.3), (5.5, 2.0), (6.0, 1.3), (5., 1.3)];
        let interior_points = interior.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![LineString(interior_points)]);
        let rotated = poly1.rotate(-15.0, &Point::new(0.0, 0.0));
        let correct_outside = vec![(5.088448176547862, -0.3281693992235354),
                                   (4.381341395361314, 0.8965754721680537),
                                   (4.640160440463836, 1.862501298457122),
                                   (5.864905311855424, 2.5696080796436696),
                                   (6.830831138144493, 2.310789034541149),
                                   (7.53793791933104, 1.0860441631495599),
                                   (7.279118874228519, 0.12011833686049145),
                                   (6.054374002836931, -0.5869884443260561),
                                   (5.088448176547862, -0.3281693992235354)];
        let correct_inside = vec![(5.166093890078619, -0.03839165133681477),
                                  (5.830230134794917, 0.5083469045142726),
                                  (6.132019716367687, -0.2972106964393355),
                                  (5.166093890078619, -0.03839165133681477)];
        let correct = Polygon::new(
            LineString(correct_outside.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>()),
            vec![LineString(correct_inside.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<_>>())]
        );
        // results agree with Shapely / GEOS
        assert_eq!(rotated, correct);
    }
}
