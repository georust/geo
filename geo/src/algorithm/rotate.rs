use crate::algorithm::centroid::Centroid;
use crate::algorithm::map_coords::MapCoords;
use crate::{
    CoordFloat, GeoFloat, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point,
    Polygon,
};

#[inline]
fn rotate_inner<T>(x: T, y: T, x0: T, y0: T, sin_theta: T, cos_theta: T) -> Point<T>
where
    T: CoordFloat,
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
fn rotate_one<T: CoordFloat>(angle: T, origin: Point<T>, point: Point<T>) -> Point<T> {
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
    T: CoordFloat,
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
    /// use geo::algorithm::rotate::Rotate;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 5.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// let rotated = line_string.rotate(-45.0);
    ///
    /// let expected = line_string![
    ///     (x: -2.0710678118654755, y: 5.0),
    ///     (x: 5.0, y: 5.0),
    ///     (x: 12.071067811865476, y: 5.0),
    /// ];
    ///
    /// assert_eq!(expected, rotated);
    /// ```
    fn rotate(&self, angle: T) -> Self
    where
        T: CoordFloat;
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
    /// use geo::algorithm::rotate::RotatePoint;
    /// use geo::{line_string, point};
    ///
    /// let ls = line_string![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 5.0, y: 5.0),
    ///     (x: 10.0, y: 10.0)
    /// ];
    ///
    /// let rotated = ls.rotate_around_point(
    ///     -45.0,
    ///     point!(x: 10.0, y: 0.0),
    /// );
    ///
    /// assert_eq!(rotated, line_string![
    ///     (x: 2.9289321881345245, y: 7.071067811865475),
    ///     (x: 10.0, y: 7.0710678118654755),
    ///     (x: 17.071067811865476, y: 7.0710678118654755)
    /// ]);
    /// ```
    fn rotate_around_point(&self, angle: T, point: Point<T>) -> Self
    where
        T: CoordFloat;
}

impl<T, G> RotatePoint<T> for G
where
    T: CoordFloat,
    G: MapCoords<T, T, Output = G>,
{
    fn rotate_around_point(&self, angle: T, point: Point<T>) -> Self {
        let (sin_theta, cos_theta) = angle.to_radians().sin_cos();
        let (x0, y0) = point.x_y();
        self.map_coords(|&(x, y)| rotate_inner(x, y, x0, y0, sin_theta, cos_theta).x_y())
    }
}

impl<T> Rotate<T> for Point<T>
where
    T: CoordFloat,
{
    /// Rotate the Point about itself by the given number of degrees
    /// This operation leaves the point coordinates unchanged
    fn rotate(&self, _angle: T) -> Self {
        *self
    }
}

impl<T> Rotate<T> for Line<T>
where
    T: GeoFloat,
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
    T: GeoFloat,
{
    /// Rotate the LineString about its centroid by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        rotate_many(angle, self.centroid().unwrap(), self.points_iter()).collect()
    }
}

impl<T> Rotate<T> for Polygon<T>
where
    T: GeoFloat,
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
    T: GeoFloat,
{
    /// Rotate the contained Polygons about their centroids by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        match self.centroid() {
            Some(centroid) => {
                MultiPolygon(self.iter().map(|poly| poly.rotate_around_point(angle, centroid)).collect())
            },
            None => {
                // Multipolygon was empty or otherwise degenerate and had no computable centroid
                self.clone()
            }
        }
    }
}

impl<T> Rotate<T> for MultiLineString<T>
where
    T: GeoFloat,
{
    /// Rotate the contained LineStrings about their centroids by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        MultiLineString(self.iter().map(|ls| ls.rotate(angle)).collect())
    }
}

impl<T> Rotate<T> for MultiPoint<T>
where
    T: CoordFloat,
{
    /// Rotate the contained Points about their centroids by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        MultiPoint(self.iter().map(|p| p.rotate(angle)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, point, polygon, Coordinate, Point};
    use approx::assert_relative_eq;

    #[test]
    fn test_rotate_around_point() {
        let p = point!(x: 1.0, y: 5.0);
        let rotated = p.rotate(30.0);
        // results agree with Shapely / GEOS
        assert_eq!(rotated, Point::new(1.0, 5.0));
    }
    #[test]
    fn test_rotate_linestring() {
        let linestring = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 5.0),
            (x: 10.0, y: 10.0)
        ];
        let rotated = linestring.rotate(-45.0);
        // results agree with Shapely / GEOS
        assert_eq!(
            rotated,
            line_string![
                (x: -2.0710678118654755, y: 5.0),
                (x: 5.0, y: 5.0),
                (x: 12.071067811865476, y: 5.0)
            ]
        );
    }
    #[test]
    fn test_rotate_polygon() {
        let poly1 = polygon![
            (x: 5., y: 1.),
            (x: 4., y: 2.),
            (x: 4., y: 3.),
            (x: 5., y: 4.),
            (x: 6., y: 4.),
            (x: 7., y: 3.),
            (x: 7., y: 2.),
            (x: 6., y: 1.),
            (x: 5., y: 1.)
        ];
        let rotated = poly1.rotate(-15.0);
        let correct = polygon![
            (x: 4.628808519201685, y: 1.1805207831176578),
            (x: 3.921701738015137, y: 2.405265654509247),
            (x: 4.180520783117657, y: 3.3711914807983154),
            (x: 5.405265654509247, y: 4.0782982619848624),
            (x: 6.371191480798315, y: 3.819479216882342),
            (x: 7.0782982619848624, y: 2.594734345490753),
            (x: 6.819479216882343, y: 1.6288085192016848),
            (x: 5.594734345490753, y: 0.9217017380151371),
            (x: 4.628808519201685, y: 1.1805207831176578)
        ];
        // results agree with Shapely / GEOS
        assert_eq!(rotated, correct);
    }
    #[test]
    fn test_rotate_polygon_holes() {
        let poly1 = polygon![
            exterior: [
                (x: 5.0, y: 1.0),
                (x: 4.0, y: 2.0),
                (x: 4.0, y: 3.0),
                (x: 5.0, y: 4.0),
                (x: 6.0, y: 4.0),
                (x: 7.0, y: 3.0),
                (x: 7.0, y: 2.0),
                (x: 6.0, y: 1.0),
                (x: 5.0, y: 1.0)
            ],
            interiors: [
                [
                    (x: 5.0, y: 1.3),
                    (x: 5.5, y: 2.0),
                    (x: 6.0, y: 1.3),
                    (x: 5.0, y: 1.3),
                ],
                [
                    (x: 5., y: 2.3),
                    (x: 5.5, y: 3.0),
                    (x: 6., y: 2.3),
                    (x: 5., y: 2.3),
                ],
            ],
        ];
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
        let line1 = Line::from([(1., 1.), (-1., 1.)]);
        assert_relative_eq!(line0.rotate(90.0), line1);
    }
    #[test]
    fn test_rotate_line_around_point() {
        let line0 = Line::new(Point::new(0., 0.), Point::new(0., 2.));
        let line1 = Line::new(Point::new(0., 0.), Point::new(-2., 0.));
        assert_relative_eq!(line0.rotate_around_point(90., Point::new(0., 0.)), line1);
    }

    #[test]
    fn test_rotate_multipolygon_around_centroid() {
        let multipolygon: MultiPolygon<f64> = vec![
            polygon![
                (x: 0., y: 0.),
                (x: 10., y: 0.),
                (x: 10., y: 10.),
                (x: 0., y: 10.),
                (x: 0., y: 0.),
            ],
            polygon![
                (x: 0., y: 0.),
                (x: -10., y: 0.),
                (x: -10., y: -10.),
                (x: 0., y: -10.),
                (x: 0., y: 0.),
            ]
        ].into();
            
        let expected: MultiPolygon<f64> = vec![
            polygon![
                (x: 0., y: 0.),
                (x: 7.0710678118654755, y: 7.0710678118654746),
                (x: 0., y: 14.1421356237309510),
                (x: -7.0710678118654746, y: 7.0710678118654755),
                (x: 0., y: 0.),
            ],
            polygon![
                (x: 0., y: 0.),
                (x: -7.0710678118654755, y: -7.0710678118654746),
                (x: 0., y: -14.1421356237309510),
                (x: 7.0710678118654746, y: -7.0710678118654755),
                (x: 0., y: 0.),
            ]
        ].into();

        // results agree with Shapely / GEOS
        // (relaxing the episilon a bit)
        assert_relative_eq!(multipolygon.rotate(45.), expected, epsilon=1e-12);
    }

    #[test]
    fn test_rotate_multipolygon_errors_gracefully() {
        // an multipolygon whose centroid cannot be found should return itself, rather
        // than panicing (e.g. an empty multipolygon, or a multipolygon composed of empty polygons)
        let empty_multipolygon: MultiPolygon<f64> = Vec::<Polygon<f64>>::new().into();
        let rotated_empty_multipolygon = empty_multipolygon.rotate(90.);
        assert_eq!(empty_multipolygon, rotated_empty_multipolygon);
    }
}
