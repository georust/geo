use crate::algorithm::centroid::Centroid;
use crate::algorithm::map_coords::MapCoords;
use crate::prelude::BoundingRect;
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
    /// Rotate a geometry around its [centroid](Centroid) by an angle, in degrees
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
    /// let rotated = line_string.rotate_around_centroid(-45.0);
    ///
    /// let expected = line_string![
    ///     (x: -2.0710678118654755, y: 5.0),
    ///     (x: 5.0, y: 5.0),
    ///     (x: 12.071067811865476, y: 5.0),
    /// ];
    ///
    /// assert_eq!(expected, rotated);
    /// ```
    fn rotate_around_centroid(&self, angle: T) -> Self
    where
        T: CoordFloat;

    /// Rotate a geometry around the center of its [bounding box](BoundingRect) by an angle, in
    /// degrees.
    ///
    /// Positive angles are counter-clockwise, and negative angles are clockwise rotations.
    ///
    /// # Units
    ///
    /// - `angle`: degrees
    fn rotate_around_center(&self, angle: T) -> Self
    where
        T: CoordFloat;

    #[deprecated(
        note = "Equivalent to `rotate_around_centroid` except for `Polygon<T>`, 
                    where it is equivalent to rotating around the polygon's outer ring. 
                    Call that instead, or `rotate_around_center` if you'd like to rotate 
                    around the geometry's bounding box center."
    )]
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

    fn rotate_around_center(&self, _angle: T) -> Self {
        *self
    }

    fn rotate_around_centroid(&self, _angle: T) -> Self {
        *self
    }
}

impl<T> Rotate<T> for Line<T>
where
    T: GeoFloat,
{
    fn rotate(&self, angle: T) -> Self {
        self.rotate_around_centroid(angle)
    }

    fn rotate_around_centroid(&self, angle: T) -> Self {
        let centroid = self.centroid();
        self.rotate_around_point(angle, centroid)
    }

    fn rotate_around_center(&self, angle: T) -> Self {
        let center: Point<T> = self.bounding_rect().center().into();
        self.rotate_around_point(angle, center)
    }
}

impl<T> Rotate<T> for LineString<T>
where
    T: GeoFloat,
{
    fn rotate(&self, angle: T) -> Self {
        self.rotate_around_centroid(angle)
    }

    /// Rotate the LineString about its centroid by the given number of degrees
    fn rotate_around_centroid(&self, angle: T) -> Self {
        match self.centroid() {
            Some(centroid) => self.rotate_around_point(angle, centroid),
            None => {
                // LineString was empty or otherwise degenerate and had no computable centroid
                self.clone()
            }
        }
    }

    fn rotate_around_center(&self, angle: T) -> Self {
        match self.bounding_rect() {
            Some(bounding_rect) => self.rotate_around_point(angle, bounding_rect.center().into()),
            None => self.clone(), // LineString was empty or otherwise degenerate and had no computable bounding rect
        }
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
            self.centroid()
        } else {
            self.exterior().centroid()
        };

        // return a rotated polygon, or a clone if no centroid is computable
        if let Some(centroid) = centroid {
            Polygon::new(
                rotate_many(angle, centroid, self.exterior().points()).collect(),
                self.interiors()
                    .iter()
                    .map(|ring| rotate_many(angle, centroid, ring.points()).collect())
                    .collect(),
            )
        } else {
            // Polygon was empty or otherwise degenerate and had no computable centroid
            self.clone()
        }
    }

    /// Rotate the Polygon about its centroid by the given number of degrees
    fn rotate_around_centroid(&self, angle: T) -> Self {
        match self.centroid() {
            Some(centroid) => self.rotate_around_point(angle, centroid),
            None => self.clone(), // Polygon was empty or otherwise degenerate and had no computable centroid
        }
    }

    /// Rotate the Polygon about the center of its bounding rectangle by the given number of degrees
    fn rotate_around_center(&self, angle: T) -> Self {
        match self.bounding_rect() {
            Some(bounding_rect) => self.rotate_around_point(angle, bounding_rect.center().into()),
            None => self.clone(), // Polygon was empty or otherwise degenerate and had no computable center
        }
    }
}

impl<T> Rotate<T> for MultiPolygon<T>
where
    T: GeoFloat,
{
    fn rotate(&self, angle: T) -> Self {
        self.rotate_around_centroid(angle)
    }

    /// Rotate the MultiPolygon about the center of its bounding rectangle by the given number of degrees
    fn rotate_around_center(&self, angle: T) -> Self {
        match self.bounding_rect() {
            Some(bounding_rect) => self.rotate_around_point(angle, bounding_rect.center().into()),
            None => self.clone(), // MultiPolygon was empty or otherwise degenerate and had no computable center
        }
    }

    /// Rotate the contained Polygons about their centroids by the given number of degrees
    fn rotate_around_centroid(&self, angle: T) -> Self {
        match self.centroid() {
            Some(centroid) => self.rotate_around_point(angle, centroid),
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
    /// Rotate the contained LineStrings about their centroid by the given number of degrees
    fn rotate(&self, angle: T) -> Self {
        self.rotate_around_centroid(angle)
    }

    /// Rotate the contained LineStrings about center of the bounding rectangle by the given number of degrees
    fn rotate_around_center(&self, angle: T) -> Self {
        match self.bounding_rect() {
            Some(bounding_rect) => self.rotate_around_point(angle, bounding_rect.center().into()),
            None => self.clone(),
        }
    }

    /// Rotate the contained LineStrings about the centroid by the given number of degrees
    fn rotate_around_centroid(&self, angle: T) -> Self {
        match self.centroid() {
            Some(centroid) => self.rotate_around_point(angle, centroid),
            None => self.clone(), // MultiLineString was empty or otherwise degenerate and had not computable bounding rect
        }
    }
}

impl<T> Rotate<T> for MultiPoint<T>
where
    T: GeoFloat,
{
    fn rotate(&self, angle: T) -> Self {
        self.rotate_around_centroid(angle)
    }

    /// Rotate the contained Points about the center of their bounding rectangle by the given number of degrees
    fn rotate_around_center(&self, angle: T) -> Self {
        match self.bounding_rect() {
            Some(bounding_rect) => self.rotate_around_point(angle, bounding_rect.center().into()),
            None => self.clone(), // MultiPoint was empty or otherwise degenerate and had no computable bounding rectangle
        }
    }

    /// Rotate the contained Points about the centroid by the given number of degrees
    fn rotate_around_centroid(&self, angle: T) -> Self {
        match self.centroid() {
            Some(centroid) => self.rotate_around_point(angle, centroid),
            None => self.clone(), // MultiPoint was empty or otherwise degenerate and had no computable centroid
        }
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
        let rotated = p.rotate_around_centroid(30.0);
        // results agree with Shapely / GEOS
        assert_eq!(rotated, Point::new(1.0, 5.0));
    }

    #[test]
    fn test_rotate_points() {
        let point = point!(x: 1.0, y: 5.0);
        let rotated_center = point.rotate_around_center(30.);
        let rotated_centroid = point.rotate_around_centroid(30.);

        // results agree with Shapely / GEOS
        // a rotated point should always equal itself
        assert_eq!(point, rotated_center);
        assert_eq!(point, rotated_centroid);
    }

    #[test]
    fn test_rotate_multipoints() {
        let multi_points = MultiPoint(vec![
            point!(x: 0., y: 0.),
            point!(x: 1., y: 1.),
            point!(x: 2., y: 1.),
        ]);

        // Results match shapely for `centroid`
        let expected_for_centroid = MultiPoint(vec![
            point!(x: 0.7642977396044841, y: -0.5118446353109125),
            point!(x: 0.7642977396044842, y:  0.9023689270621824),
            point!(x: 1.471404520791032, y:  1.60947570824873),
        ]);
        assert_relative_eq!(
            multi_points.rotate_around_centroid(45.),
            expected_for_centroid
        );

        // Results match shapely for `center`
        let expected_for_center = MultiPoint(vec![
            point!(x: 0.6464466094067262, y: -0.5606601717798212),
            point!(x: 0.6464466094067263, y: 0.8535533905932737),
            point!(x: 1.353553390593274, y: 1.560660171779821),
        ]);
        assert_relative_eq!(multi_points.rotate_around_center(45.), expected_for_center);
    }

    #[test]
    fn test_rotate_linestring() {
        let linestring = line_string![
            (x: 0.0, y: 0.0),
            (x: 5.0, y: 5.0),
            (x: 5.0, y: 10.0)
        ];

        // results agree with Shapely / GEOS for `centroid`
        let rotated_around_centroid = linestring.rotate_around_centroid(-45.0);
        assert_relative_eq!(
            rotated_around_centroid,
            line_string![
                (x: -2.196699141100894, y: 3.838834764831844),
                (x: 4.874368670764582, y: 3.838834764831844),
                (x: 8.40990257669732, y: 7.374368670764582)
            ]
        );

        // results agree with Shapely / GEOS for `center`
        let rotated_around_center = linestring.rotate_around_center(-45.0);
        assert_relative_eq!(
            rotated_around_center,
            line_string![
                (x: -2.803300858899106, y: 3.232233047033631),
                (x: 4.267766952966369, y: 3.232233047033632),
                (x: 7.803300858899107, y: 6.767766952966369)
            ],
            epsilon = 1e-12
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
        let rotated = poly1.rotate_around_centroid(-15.0);
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
        #[allow(deprecated)]
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

        // now rotate around center
        let center_expected = polygon![
            exterior: [
                (x: 4.628808519201685, y: 1.180520783117658),
                (x: 3.921701738015137, y: 2.405265654509247),
                (x: 4.180520783117659, y: 3.371191480798315),
                (x: 5.405265654509247, y: 4.078298261984862),
                (x: 6.371191480798316, y: 3.819479216882342),
                (x: 7.078298261984862, y: 2.594734345490753),
                (x: 6.819479216882343, y: 1.628808519201685),
                (x: 5.594734345490753, y: 0.9217017380151372),
                (x: 4.628808519201685, y: 1.180520783117658),
            ],
            interiors: [
                [
                    (x: 4.706454232732442, y: 1.470298531004379),
                    (x: 5.37059047744874, y: 2.017037086855466),
                    (x: 5.67238005902151, y: 1.211479485901858),
                    (x: 4.706454232732442, y: 1.470298531004379),
                ],
                [
                    (x: 4.965273277834962, y: 2.436224357293447),
                    (x: 5.62940952255126, y: 2.982962913144534),
                    (x: 5.931199104124032, y: 2.177405312190926),
                    (x: 4.965273277834962, y: 2.436224357293447),
                ],
            ],
        ];

        let rotated_around_center = poly1.rotate_around_center(-15.);

        assert_relative_eq!(rotated_around_center, center_expected, epsilon = 1e-12);

        // now rotate around centroid
        let centroid_expected = polygon![
            exterior: [
                (x: 4.615388272418591, y: 1.182287592124891),
                (x: 3.908281491232044, y: 2.40703246351648),
                (x: 4.167100536334565, y: 3.372958289805549),
                (x: 5.391845407726153, y: 4.080065070992097),
                (x: 6.357771234015222, y: 3.821246025889576),
                (x: 7.064878015201769, y: 2.596501154497987),
                (x: 6.806058970099248, y: 1.630575328208918),
                (x: 5.58131409870766, y: 0.9234685470223708),
                (x: 4.615388272418591, y: 1.182287592124891),
            ],
            interiors: [
                [
                    (x: 4.693033985949348, y: 1.472065340011612),
                    (x: 5.357170230665646, y: 2.0188038958627),
                    (x: 5.658959812238415, y: 1.213246294909091),
                    (x: 4.693033985949348, y: 1.472065340011612),
                ],
                [
                    (x: 4.951853031051868, y: 2.43799116630068),
                    (x: 5.615989275768166, y: 2.984729722151768),
                    (x: 5.917778857340937, y: 2.179172121198159),
                    (x: 4.951853031051868, y: 2.43799116630068),
                ],
            ],
        ];
        let rotated_around_centroid = poly1.rotate_around_centroid(-15.);
        assert_relative_eq!(rotated_around_centroid, centroid_expected, epsilon = 1e-12);
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
        assert_relative_eq!(line0.rotate_around_centroid(90.0), line1);
        assert_relative_eq!(line0.rotate_around_center(90.0), line1);
    }

    #[test]
    fn test_rotate_multi_line_string() {
        let ls1 = line_string![
            (x: 0., y: 0.),
            (x: 1., y: 1.),
            (x: 4., y: 1.),
        ];
        let ls2 = line_string![
            (x: 10., y: 10.),
            (x: 20., y: 20.),
            (x: 40., y: 20.)
        ];
        let multi_line_string: MultiLineString<f64> = MultiLineString(vec![ls1, ls2]);

        // Results match with Shapely for `centroid`
        let expected_around_centroid = MultiLineString(vec![
            line_string![
                (x: -5.062519283392216, y: 19.72288595632566),
                (x: -3.648305721019121, y: 19.72288595632566),
                (x: -1.526985377459479, y: 17.60156561276602)
            ],
            line_string![
                (x: 9.079616340338735, y: 19.72288595632566),
                (x: 23.22175196406969, y: 19.72288595632566),
                (x: 37.36388758780063, y: 5.580750332594715)
            ],
        ]);
        assert_relative_eq!(
            multi_line_string.rotate_around_centroid(-45.),
            expected_around_centroid,
            epsilon = 1e-12
        );

        // Results match with Shapely for `center`
        let expected_around_center: MultiLineString<f64> = MultiLineString(vec![
            line_string![
                (x: -1.213203435596426, y: 17.07106781186548),
                (x: 0.2010101267766693, y: 17.07106781186548),
                (x: 2.322330470336312, y: 14.94974746830583),
            ],
            line_string![
                (x: 12.92893218813452, y: 17.07106781186548),
                (x: 27.07106781186548, y: 17.07106781186548),
                (x: 41.21320343559643, y: 2.928932188134528),

            ],
        ]);
        assert_relative_eq!(
            multi_line_string.rotate_around_center(-45.),
            expected_around_center,
            epsilon = 1e-12
        );
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
            ],
        ]
        .into();

        let expected_centroid: MultiPolygon<f64> = vec![
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
            ],
        ]
        .into();

        // results agree with Shapely / GEOS
        // (relaxing the episilon a bit)
        assert_relative_eq!(
            multipolygon.rotate_around_centroid(45.),
            expected_centroid,
            epsilon = 1e-12
        );
    }

    #[test]
    fn test_rotate_multipolygons() {
        let multipolygon: MultiPolygon<f64> = vec![
            polygon![
               (x: 1., y: 1. ),
               (x: 2., y: 1. ),
               (x: 2., y: 10.),
               (x: 1., y: 10.),
               (x: 1., y: 1. ),
            ],
            polygon![
                (x: 10., y:  1.),
                (x: 12., y:  1.),
                (x: 12., y:  12.),
                (x: 10., y:  12.),
                (x: 10., y:  1.),
            ],
        ]
        .into();

        let expected_center: MultiPolygon<f64> = vec![
            polygon![
                (x: -0.2360967926537398, y: 2.610912703473988),
                (x: 0.7298290336353284, y: 2.352093658371467),
                (x: 3.059200439558015, y: 11.04542609497308),
                (x: 2.093274613268947, y: 11.3042451400756),
                (x: -0.2360967926537398, y: 2.610912703473988),
            ],
            polygon![
                (x: 8.457235643947875, y: 0.2815412975513012),
                (x: 10.38908729652601, y: -0.2360967926537403),
                (x: 13.23609679265374, y: 10.38908729652601),
                (x: 11.3042451400756, y: 10.90672538673105),
                (x: 8.457235643947875, y: 0.2815412975513012),
            ],
        ]
        .into();

        let expected_centroid: MultiPolygon<f64> = vec![
            polygon![
                (x: -0.1016007672888048, y: 3.05186627999456),
                (x: 0.8643250590002634, y: 2.793047234892039),
                (x: 3.19369646492295, y: 11.48637967149365),
                (x: 2.227770638633882, y: 11.74519871659617),
                (x: -0.1016007672888048, y: 3.05186627999456),
            ],
            polygon![
                (x: 8.591731669312811, y: 0.7224948740718733),
                (x: 10.52358332189095, y: 0.2048567838668318),
                (x: 13.37059281801868, y: 10.83004087304658),
                (x: 11.43874116544054, y: 11.34767896325162),
                (x: 8.591731669312811, y: 0.7224948740718733),
            ],
        ]
        .into();

        // results agree with Shapely / GEOS
        assert_relative_eq!(
            multipolygon.rotate_around_center(-15.),
            expected_center,
            epsilon = 1e-12
        );
        assert_relative_eq!(
            multipolygon.rotate_around_centroid(-15.),
            expected_centroid,
            epsilon = 1e-12
        );
    }

    #[test]
    fn test_rotate_empty_geometries_error_gracefully() {
        // line string
        let empty_linestring: LineString<f64> = line_string![];
        let rotated_empty_linestring = empty_linestring.rotate_around_centroid(90.);
        assert_eq!(empty_linestring, rotated_empty_linestring);

        // multi line string
        let empty_multilinestring: MultiLineString<f64> = MultiLineString::<f64>(vec![]);
        let rotated_empty_multilinestring = empty_multilinestring.rotate_around_centroid(90.);
        assert_eq!(empty_multilinestring, rotated_empty_multilinestring);

        // polygon
        let empty_polygon: Polygon<f64> = polygon![];
        let rotated_empty_polygon = empty_polygon.rotate_around_centroid(90.);
        assert_eq!(empty_polygon, rotated_empty_polygon);

        // multi polygon
        let empty_multipolygon: MultiPolygon<f64> = Vec::<Polygon<f64>>::new().into();
        let rotated_empty_multipolygon = empty_multipolygon.rotate_around_centroid(90.);
        assert_eq!(empty_multipolygon, rotated_empty_multipolygon);
    }
}
