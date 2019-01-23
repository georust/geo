use algorithm::centroid::Centroid;
use algorithm::map_coords::MapCoords;
use num_traits::{Float, FromPrimitive};
use std::iter::Sum;
use {Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};

#[inline]
fn rotate_inner<T>(x: T, y: T, x0: T, y0: T, sin_theta: T, cos_theta: T) -> Point<T>
where
    T: Float,
{
    let x = x - x0;
    let y = y - y0;
    Point::new(
        x * cos_theta - y * sin_theta + x0,
        x * sin_theta + y * cos_theta + y0,
    )
}

// Rotate a single point "angle" degrees about an origin. Origin can be an
// arbitrary point. Pass Point::new(0., 0.) for the actual origin.
fn rotate_one<T: Float>(angle: T, origin: Point<T>, point: Point<T>) -> Point<T> {
    let (sin_theta, cos_theta) = angle.to_radians().sin_cos();
    rotate_inner(
        point.x(),
        point.y(),
        origin.x(),
        origin.y(),
        sin_theta,
        cos_theta,
    )
}

// Rotate an iterator of points "angle" degrees about an origin. Origin can be
// an arbitrary point. Pass Point::new(0., 0.) for the actual origin.
fn rotate_many<T>(
    angle: T,
    origin: Point<T>,
    points: impl Iterator<Item = Point<T>>,
) -> impl Iterator<Item = Point<T>>
where
    T: Float,
{
    let (sin_theta, cos_theta) = angle.to_radians().sin_cos();
    let (x0, y0) = origin.x_y();
    points.map(move |point| rotate_inner(point.x(), point.y(), x0, y0, sin_theta, cos_theta))
}

pub trait Rotate<T> {
    /// Rotate a Geometry around its centroid by an angle, in degrees
    ///
    /// Positive angles are counter-clockwise, and negative angles are clockwise rotations.
    ///
    /// # Units
    ///
    /// - `angle`: degrees
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::rotate::{Rotate};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(0.0, 0.0));
    /// vec.push(Point::new(5.0, 5.0));
    /// vec.push(Point::new(10.0, 10.0));
    /// let linestring = LineString::from(vec);
    /// let rotated = linestring.rotate(-45.0);
    /// let mut correct = Vec::new();
    /// correct.push(Point::new(-2.0710678118654755, 5.0));
    /// correct.push(Point::new(5.0, 5.0));
    /// correct.push(Point::new(12.071067811865476, 5.0));
    /// let correct_ls = LineString::from(correct);
    /// assert_eq!(rotated, correct_ls);
    /// ```
    fn rotate(&self, angle: T) -> Self
    where
        T: Float;
}

pub trait RotatePoint<T> {
    /// Rotate a Geometry around an arbitrary point by an angle, given in degrees
    ///
    /// Positive angles are counter-clockwise, and negative angles are clockwise rotations.
    ///
    /// # Units
    ///
    /// - `angle`: degrees
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::rotate::{RotatePoint};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(0.0, 0.0));
    /// vec.push(Point::new(5.0, 5.0));
    /// vec.push(Point::new(10.0, 10.0));
    /// let linestring = LineString::from(vec);
    /// let rotated = linestring.rotate_around_point(-45.0, Point::new(10.0, 0.0));
    /// let mut correct = Vec::new();
    /// correct.push(Point::new(2.9289321881345245, 7.071067811865475));
    /// correct.push(Point::new(10.0, 7.0710678118654755));
    /// correct.push(Point::new(17.071067811865476, 7.0710678118654755));
    /// let correct_ls = LineString::from(correct);
    /// assert_eq!(rotated, correct_ls);
    /// ```
    fn rotate_around_point(&self, angle: T, point: Point<T>) -> Self
    where
        T: Float;
}

impl<T, G> RotatePoint<T> for G
where
    T: Float,
    G: MapCoords<T, T, Output = G>,
{
    fn rotate_around_point(&self, angle: T, point: Point<T>) -> Self {
        let (sin_theta, cos_theta) = angle.to_radians().sin_cos();
        let (x0, y0) = point.x_y();
        self.map_coords(&|&(x, y)| rotate_inner(x, y, x0, y0, sin_theta, cos_theta).x_y())
    }
}

impl<T> Rotate<T> for Point<T>
where
    T: Float,
{
    /// Rotate the Point about itself by the given number of degrees
    /// This operation leaves the point coordinates unchanged
    fn rotate(&self, _angle: T) -> Self {
        *self
    }
}

impl<T> Rotate<T> for Line<T>
where
    T: Float,
{
    fn rotate(&self, angle: T) -> Self {
        let centroid = self.centroid();
        Line::new(
            rotate_one(angle, centroid, self.start_point()),
            rotate_one(angle, centroid, self.end_point()),
        )
    }
}

impl<T> Rotate<T> for LineString<T>
where
    T: Float,
{
    /// Rotate the LineString about its centroid by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        rotate_many(angle, self.centroid().unwrap(), self.points_iter()).collect()
    }
}

impl<T> Rotate<T> for Polygon<T>
where
    T: Float + FromPrimitive + Sum,
{
    /// Rotate the Polygon about its centroid by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        // if a polygon has holes, use the centroid of its outer shell as the rotation origin
        let centroid = if self.interiors().is_empty() {
            self.centroid().unwrap()
        } else {
            self.exterior().centroid().unwrap()
        };
        Polygon::new(
            rotate_many(angle, centroid, self.exterior().points_iter()).collect(),
            self.interiors()
                .iter()
                .map(|ring| rotate_many(angle, centroid, ring.points_iter()).collect())
                .collect(),
        )
    }
}

impl<T> Rotate<T> for MultiPolygon<T>
where
    T: Float + FromPrimitive + Sum,
{
    /// Rotate the contained Polygons about their centroids by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        MultiPolygon(self.0.iter().map(|poly| poly.rotate(angle)).collect())
    }
}

impl<T> Rotate<T> for MultiLineString<T>
where
    T: Float + FromPrimitive,
{
    /// Rotate the contained LineStrings about their centroids by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        MultiLineString(self.0.iter().map(|ls| ls.rotate(angle)).collect())
    }
}

impl<T> Rotate<T> for MultiPoint<T>
where
    T: Float + FromPrimitive,
{
    /// Rotate the contained Points about their centroids by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        MultiPoint(self.0.iter().map(|p| p.rotate(angle)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use {Coordinate, LineString, Point, Polygon};
    #[test]
    fn test_rotate_around_point() {
        let p = Point::new(1.0, 5.0);
        let rotated = p.rotate(30.0);
        // results agree with Shapely / GEOS
        assert_eq!(rotated, Point::new(1.0, 5.0));
    }
    #[test]
    fn test_rotate_linestring() {
        let vec = vec![(0.0, 0.0), (5.0, 5.0), (10.0, 10.0)];
        let linestring = LineString::from(vec);
        let rotated = linestring.rotate(-45.0);
        let mut correct = Vec::new();
        correct.push(Point::new(-2.0710678118654755, 5.0));
        correct.push(Point::new(5.0, 5.0));
        correct.push(Point::new(12.071067811865476, 5.0));
        let correct_ls = LineString::from(correct);
        // results agree with Shapely / GEOS
        assert_eq!(rotated, correct_ls);
    }
    #[test]
    fn test_rotate_polygon() {
        let coords = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.),
        ];
        let poly1 = Polygon::new(LineString::from(coords), vec![]);
        let rotated = poly1.rotate(-15.0);
        let correct_outside = vec![
            (4.628808519201685, 1.1805207831176578),
            (3.921701738015137, 2.405265654509247),
            (4.180520783117657, 3.3711914807983154),
            (5.405265654509247, 4.0782982619848624),
            (6.371191480798315, 3.819479216882342),
            (7.0782982619848624, 2.594734345490753),
            (6.819479216882343, 1.6288085192016848),
            (5.594734345490753, 0.9217017380151371),
            (4.628808519201685, 1.1805207831176578),
        ];
        let correct = Polygon::new(
            LineString(
                correct_outside
                    .iter()
                    .map(|e| Coordinate::from((e.0, e.1)))
                    .collect::<Vec<_>>(),
            ),
            vec![],
        );
        // results agree with Shapely / GEOS
        assert_eq!(rotated, correct);
    }
    #[test]
    fn test_rotate_polygon_holes() {
        let ls1 = LineString::from(vec![
            (5.0, 1.0),
            (4.0, 2.0),
            (4.0, 3.0),
            (5.0, 4.0),
            (6.0, 4.0),
            (7.0, 3.0),
            (7.0, 2.0),
            (6.0, 1.0),
            (5.0, 1.0),
        ]);

        let ls2 = LineString::from(vec![(5.0, 1.3), (5.5, 2.0), (6.0, 1.3), (5.0, 1.3)]);

        let ls3 = LineString::from(vec![(5., 2.3), (5.5, 3.0), (6., 2.3), (5., 2.3)]);

        let poly1 = Polygon::new(ls1, vec![ls2, ls3]);
        let rotated = poly1.rotate(-15.0);
        let correct_outside = vec![
            Coordinate::from((4.628808519201685, 1.180520783117658)),
            Coordinate::from((3.921701738015137, 2.4052656545092472)),
            Coordinate::from((4.180520783117657, 3.3711914807983154)),
            Coordinate::from((5.405265654509247, 4.078298261984863)),
            Coordinate::from((6.371191480798315, 3.8194792168823426)),
            Coordinate::from((7.0782982619848624, 2.594734345490753)),
            Coordinate::from((6.819479216882343, 1.628808519201685)),
            Coordinate::from((5.594734345490753, 0.9217017380151373)),
            Coordinate::from((4.628808519201685, 1.180520783117658)),
        ];
        let correct_inside = vec![
            Coordinate::from((4.706454232732441, 1.4702985310043786)),
            Coordinate::from((5.37059047744874, 2.017037086855466)),
            Coordinate::from((5.672380059021509, 1.2114794859018578)),
            Coordinate::from((4.706454232732441, 1.4702985310043786)),
        ];
        assert_eq!(rotated.exterior().0, correct_outside);
        assert_eq!(rotated.interiors()[0].0, correct_inside);
    }
    #[test]
    fn test_rotate_around_point_arbitrary() {
        let p = Point::new(5.0, 10.0);
        let rotated = p.rotate_around_point(-45., Point::new(10., 34.));
        assert_eq!(rotated, Point::new(-10.506096654409877, 20.564971157455595));
    }
    #[test]
    fn test_rotate_line() {
        let line0 = Line::from([(0., 0.), (0., 2.)]);
        let line1 = Line::from([(1., 0.9999999999999999), (-1., 1.)]);
        assert_eq!(line0.rotate(90.), line1);
    }
    #[test]
    fn test_rotate_line_around_point() {
        let line0 = Line::new(Point::new(0., 0.), Point::new(0., 2.));
        let line1 = Line::new(
            Point::new(0., 0.),
            Point::new(-2., 0.00000000000000012246467991473532),
        );
        assert_eq!(line0.rotate_around_point(90., Point::new(0., 0.)), line1);
    }
}
