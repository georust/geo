use num_traits::Float;
use types::Point;
use algorithm::centroid::Centroid;
use algorithm::map_coords::MapCoords;

// rotate a slice of points "angle" degrees about an origin
// origin can be an arbitrary point, pass &Point::new(0., 0.)
// for the actual origin
pub trait Rotate<T> {
    /// Rotate a Geometry around its centroid by an angle, in degrees
    ///
    /// Positive angles are counter-clockwise, and negative angles are clockwise rotations.
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
    /// let rotated = linestring.rotate(-45.0);
    /// let mut correct = Vec::new();
    /// correct.push(Point::new(-2.0710678118654755, 5.0));
    /// correct.push(Point::new(5.0, 5.0));
    /// correct.push(Point::new(12.071067811865476, 5.0));
    /// let correct_ls = LineString(correct);
    /// assert_eq!(rotated, correct_ls);
    /// ```
    fn rotate(&self, angle: T) -> Self where T: Float;
}

pub trait RotatePoint<T> {
    /// Rotate a Geometry around an arbitrary point by an angle, given in degrees
    ///
    /// Positive angles are counter-clockwise, and negative angles are clockwise rotations.
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::rotate::{RotatePoint};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(0.0, 0.0));
    /// vec.push(Point::new(5.0, 5.0));
    /// vec.push(Point::new(10.0, 10.0));
    /// let linestring = LineString(vec);
    /// let rotated = linestring.rotate_around_point(-45.0, &Point::new(10.0, 0.0));
    /// let mut correct = Vec::new();
    /// correct.push(Point::new(2.9289321881345245, 7.071067811865475));
    /// correct.push(Point::new(10.0, 7.0710678118654755));
    /// correct.push(Point::new(17.071067811865476, 7.0710678118654755));
    /// let correct_ls = LineString(correct);
    /// assert_eq!(rotated, correct_ls);
    /// ```
    fn rotate_around_point(&self, angle: T, point: &Point<T>) -> Self where T: Float;
}

impl<T, G> RotatePoint<T> for G
    where T: Float,
        G: MapCoords<T, T, Output=G>,
{
    fn rotate_around_point(&self, angle: T, point: &Point<T>) -> Self {
        let cos_theta = angle.to_radians().cos();
        let sin_theta = angle.to_radians().sin();
        let x0 = point.x();
        let y0 = point.y();
        self.map_coords(&|&(x, y)| {
            let x = x - x0;
            let y = y - y0;
            (x * cos_theta - y * sin_theta + x0,
            x * sin_theta + y * cos_theta + y0)
        })
    }
}

impl<T, G> Rotate<T> for G
    where T: Float,
        G: MapCoords<T, T, Output=G> + Centroid<T>,
{
    fn rotate(&self, angle: T) -> Self {
        self.rotate_around_point(angle, &self.centroid().unwrap())
    }
}

#[cfg(test)]
mod test {
    use types::{Point, LineString, Polygon, Line, Coordinate};
    use super::*;
    #[test]
    fn test_rotate_around_point() {
        let p = Point::new(1.0, 5.0);
        let rotated = p.rotate(30.0);
        // results agree with Shapely / GEOS
        assert_eq!(rotated, Point::new(1.0, 5.0));
    }
    #[test]
    fn test_rotate_linestring() {
        let mut vec = Vec::new();
        vec.push(Point::new(0.0, 0.0));
        vec.push(Point::new(5.0, 5.0));
        vec.push(Point::new(10.0, 10.0));
        let linestring = LineString(vec);
        let rotated = linestring.rotate(-45.0);
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
        let points = points_raw
            .iter()
            .map(|e| Point::new(e.0, e.1))
            .collect::<Vec<_>>();
        let poly1 = Polygon::new(LineString(points), vec![]);
        let rotated = poly1.rotate(-15.0);
        let correct_outside = vec![(4.628808519201685, 1.1805207831176578),
                                   (3.921701738015137, 2.405265654509247),
                                   (4.180520783117657, 3.3711914807983154),
                                   (5.405265654509247, 4.0782982619848624),
                                   (6.371191480798315, 3.819479216882342),
                                   (7.0782982619848624, 2.594734345490753),
                                   (6.819479216882343, 1.6288085192016848),
                                   (5.594734345490753, 0.9217017380151371),
                                   (4.628808519201685, 1.1805207831176578)];
        let correct = Polygon::new(LineString(correct_outside
                                                  .iter()
                                                  .map(|e| Point::new(e.0, e.1))
                                                  .collect::<Vec<_>>()),
                                   vec![]);
        // results agree with Shapely / GEOS
        assert_eq!(rotated, correct);
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

        let ls3 = LineString(vec![Point::new(5., 2.3),
                                  Point::new(5.5, 3.0),
                                  Point::new(6., 2.3),
                                  Point::new(5., 2.3)]);

        let poly1 = Polygon::new(ls1, vec![ls2, ls3]);
        let rotated = poly1.rotate(-15.0);
        let correct_outside = [
            Point(Coordinate { x: 4.615388272418591, y: 1.1822875921248914 }),
            Point(Coordinate { x: 3.9082814912320436, y: 2.4070324635164804 }),
            Point(Coordinate { x: 4.167100536334564, y: 3.372958289805549 }),
            Point(Coordinate { x: 5.391845407726153, y: 4.080065070992096 }),
            Point(Coordinate { x: 6.357771234015222, y: 3.8212460258895753 }),
            Point(Coordinate { x: 7.064878015201769, y: 2.5965011544979864 }),
            Point(Coordinate { x: 6.806058970099249, y: 1.6305753282089182 }),
            Point(Coordinate { x: 5.58131409870766, y: 0.9234685470223707 }),
            Point(Coordinate { x: 4.615388272418591, y: 1.1822875921248914 })
        ];
        let correct_inside = [
            Point(Coordinate { x: 4.693033985949348, y: 1.472065340011612 }),
            Point(Coordinate { x: 5.357170230665646, y: 2.0188038958626993 }),
            Point(Coordinate { x: 5.6589598122384155, y: 1.2132462949090912 }),
            Point(Coordinate { x: 4.693033985949348, y: 1.472065340011612 })
        ];
        assert_eq!(rotated.exterior.0, correct_outside);
        assert_eq!(rotated.interiors[0].0, correct_inside);
    }
    #[test]
    fn test_rotate_around_point_arbitrary() {
        let p = Point::new(5.0, 10.0);
        let rotated = p.rotate_around_point(-45., &Point::new(10., 34.));
        assert_eq!(rotated, Point::new(-10.506096654409877, 20.564971157455595));
    }
    #[test]
    fn test_rotate_line() {
        let line0 = Line::new(Point::new(0., 0.), Point::new(0., 2.));
        let line1 = Line::new(Point::new(1., 0.9999999999999999), Point::new(-1., 1.));
        assert_eq!(line0.rotate(90.), line1);
    }
    #[test]
    fn test_rotate_line_around_point() {
        let line0 = Line::new(Point::new(0., 0.), Point::new(0., 2.));
        let line1 = Line::new(Point::new(0., 0.), Point::new(-2., 0.00000000000000012246467991473532));
        assert_eq!(line0.rotate_around_point(90., &Point::new(0., 0.)), line1);
    }
}
