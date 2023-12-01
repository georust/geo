use std::{fmt, fmt::Debug};

use crate::coord;
use crate::{BoundingRect, Coord, GeoFloat, Line, MultiPoint, Polygon, Triangle};

pub const DEFAULT_SUPER_TRIANGLE_EXPANSION: f64 = 20.;

type Result<T> = std::result::Result<T, DelaunayTriangulationError>;

/// Delaunay Triangulation for a given set of points using the
/// [Bowyer](https://doi.org/10.1093%2Fcomjnl%2F24.2.162)-[Watson](https://doi.org/10.1093%2Fcomjnl%2F24.2.167)
/// algorithm
pub trait TriangulateDelaunay<T: GeoFloat> {
    /// # Examples
    ///
    /// ```
    /// use geo::{coord, polygon, Triangle, TriangulateDelaunay};
    ///
    /// let points = polygon![
    ///     (x: 0., y: 0.),
    ///     (x: 1., y: 2.),
    ///     (x: 2., y: 4.),
    ///     (x: 2., y: 0.),
    ///     (x: 3., y: 2.),
    ///     (x: 4., y: 0.),
    /// ];
    ///
    /// let tri_force = points.delaunay_triangulation().unwrap();
    ///
    /// assert_eq!(vec![
    ///     Triangle(
    ///         coord! {x: 1., y: 2.},
    ///         coord! {x: 0., y: 0.},
    ///         coord! {x: 2., y: 0.},
    ///     ),
    ///     Triangle(
    ///         coord! {x: 2., y: 4.},
    ///         coord! {x: 1., y: 2.},
    ///         coord! {x: 3., y: 2.},
    ///     ),
    ///     Triangle(
    ///         coord! {x: 1., y: 2.},
    ///         coord! {x: 2., y: 0.},
    ///         coord! {x: 3., y: 2.},
    ///     ),
    ///     Triangle(
    ///         coord! {x: 3., y: 2.},
    ///         coord! {x: 2., y: 0.},
    ///         coord! {x: 4., y: 0.},
    ///     )],
    ///     tri_force
    /// );
    ///
    /// ```
    ///
    fn delaunay_triangulation(&self) -> Result<Vec<Triangle<T>>>;
}

impl<T: GeoFloat> TriangulateDelaunay<T> for Polygon<T>
where
    f64: From<T>,
{
    fn delaunay_triangulation(&self) -> Result<Vec<Triangle<T>>> {
        let super_triangle = DelaunayTriangle(create_super_triangle(self)?);

        let mut triangles: Vec<DelaunayTriangle<T>> = vec![super_triangle.clone()];
        let mut processed_points: Vec<Coord<T>> = Vec::new();
        for pt in self.exterior().coords() {
            // If we have already added the point, do not process again
            // to avoid producing intersecting triangles
            if !processed_points.contains(pt) {
                add_coordinate(&mut triangles, pt);
                processed_points.push(*pt);
            }
        }
        let triangles = remove_super_triangle(&triangles, &super_triangle);
        Ok(triangles.iter().map(|x| x.0).collect())
    }
}

impl<T: GeoFloat> TriangulateDelaunay<T> for MultiPoint<T>
where
    f64: From<T>,
{
    fn delaunay_triangulation(&self) -> Result<Vec<Triangle<T>>> {
        let poly = Polygon::new(self.into_iter().cloned().collect(), vec![]);
        poly.delaunay_triangulation()
    }
}

fn create_super_triangle<T: GeoFloat>(geometry: &Polygon<T>) -> Result<Triangle<T>>
where
    f64: From<T>,
{
    let expand_factor = T::from(DEFAULT_SUPER_TRIANGLE_EXPANSION)
        .ok_or(DelaunayTriangulationError::FailedToConstructSuperTriangle)?;
    let bounds = geometry
        .bounding_rect()
        .ok_or(DelaunayTriangulationError::FailedToConstructSuperTriangle)?;
    let width = bounds.width() * expand_factor;
    let height = bounds.height() * expand_factor;
    let bounds_min = bounds.min();
    let bounds_max = bounds.max();

    Ok(Triangle(
        coord! {x: bounds_min.x - width, y: bounds_min.y},
        coord! {x: bounds_max.x + width, y: bounds_min.y - height},
        coord! {x: bounds_max.x + width, y: bounds_max.y + height},
    ))
}

fn find_line<T: GeoFloat>(line: &Line<T>, lines: &[Line<T>]) -> Option<usize> {
    for (idx, sample_line) in lines.iter().enumerate() {
        if line == sample_line {
            return Some(idx);
        }
    }
    None
}

fn add_coordinate<T: GeoFloat>(triangles: &mut Vec<DelaunayTriangle<T>>, c: &Coord<T>)
where
    f64: From<T>,
{
    let mut bad_triangles: Vec<&DelaunayTriangle<T>> = Vec::new();

    // Check for the triangles where the point is present within the
    // corresponding circumcircle.
    for tri in triangles.iter() {
        if tri.is_in_circumcircle(c) {
            bad_triangles.push(tri);
        }
    }

    let mut bad_triangle_edges: Vec<Line<T>> = Vec::new();
    let mut bad_triangle_edge_count: Vec<u128> = Vec::new();
    for x in &bad_triangles {
        for y in x.0.to_lines() {
            let flipped_y = Line::new(y.end, y.start);
            let idx =
                find_line(&y, &bad_triangle_edges).or(find_line(&flipped_y, &bad_triangle_edges));
            if let Some(idx) = idx {
                // The unwrap is acceptable due to the push lines below.
                let count = bad_triangle_edge_count.get_mut(idx).unwrap();
                *count += 1;
            } else {
                // These two vectors must be updated at the same time
                bad_triangle_edges.push(y);
                bad_triangle_edge_count.push(1);
            }
        }
    }

    // Shared edges are those with a count of > 1
    let polygon: Vec<Line<T>> = bad_triangle_edge_count
        .iter()
        .enumerate()
        .filter(|(_, count)| **count < 2)
        .map(|(idx, _)| bad_triangle_edges[idx])
        .collect();

    // Remove the bad triangles
    let mut new_triangles: Vec<DelaunayTriangle<T>> = triangles
        .iter()
        .filter(|x| !bad_triangles.contains(x))
        .cloned()
        .collect();

    polygon
        .iter()
        .for_each(|x| new_triangles.push(DelaunayTriangle(Triangle(x.start, x.end, *c))));

    // Replace the triangles
    triangles.clear();
    new_triangles.iter().for_each(|x| triangles.push(x.clone()));
}

fn remove_super_triangle<T: GeoFloat>(
    triangles: &[DelaunayTriangle<T>],
    super_triangle: &DelaunayTriangle<T>,
) -> Vec<DelaunayTriangle<T>> {
    let mut new_triangles: Vec<DelaunayTriangle<T>> = Vec::new();
    let super_tri_vertices = super_triangle.0.to_array();

    for tri in triangles.iter() {
        let mut add_to_update = true;
        for pt in tri.0.to_array().iter() {
            if super_tri_vertices.contains(pt) {
                add_to_update = false;
            }
        }
        if add_to_update {
            new_triangles.push(tri.clone())
        }
    }
    new_triangles
}

/// A triangle structure used during Delaunay Triangulation.
#[derive(Debug, Clone, PartialEq)]
pub struct DelaunayTriangle<T: GeoFloat>(Triangle<T>);

impl<T: GeoFloat> From<Triangle<T>> for DelaunayTriangle<T> {
    fn from(value: Triangle<T>) -> Self {
        DelaunayTriangle(value)
    }
}

impl<T: GeoFloat> From<DelaunayTriangle<T>> for Triangle<T> {
    fn from(value: DelaunayTriangle<T>) -> Self {
        value.0
    }
}

fn is_line_shared<T: GeoFloat>(a: &Line<T>, b: &Line<T>) -> bool {
    (a.start == b.start && a.end == b.end) || (a.start == b.end && a.end == b.start)
}

/// Methods required for delaunay triangulation
impl<T: GeoFloat> DelaunayTriangle<T>
where
    f64: From<T>,
{
    #[cfg(feature = "voronoi")]
    /// Check if a `DelaunayTriangle` shares at least one vertex.
    pub fn shares_vertex(&self, other: &DelaunayTriangle<T>) -> bool {
        let other_vertices = other.0.to_array();
        for vertex in self.0.to_array().iter() {
            if other_vertices.contains(vertex) {
                return true;
            }
        }
        false
    }

    #[cfg(feature = "voronoi")]
    /// Check if a `DelaunayTriangle` is a neighbour i.e.
    /// shares an edge.
    /// If the triangles are neighbours the shared edge is returned.
    pub fn shares_edge(&self, other: &DelaunayTriangle<T>) -> Option<Line<T>> {
        let other_lines = other.0.to_lines();
        for line in self.0.to_lines().iter() {
            for other_line in other_lines.iter() {
                if is_line_shared(line, other_line) {
                    return Some(*line);
                }
            }
        }
        None
    }

    /// Check if a `Coord` is in the [Circumcircle](https://en.wikipedia.org/wiki/Circumcircle)
    /// for the Delaunay triangle.
    /// This method uses the determinant of the vertices of the triangle and the
    /// new point as described by [Guibas & Stolfi](https://doi.org/10.1145%2F282918.282923)
    /// and on [Wikipedia](https://en.wikipedia.org/wiki/Delaunay_triangulation#Algorithms).
    pub fn is_in_circumcircle(&self, c: &Coord<T>) -> bool {
        let a_d_x: f64 = (self.0 .0.x - c.x).into();
        let a_d_y: f64 = (self.0 .0.y - c.y).into();
        let b_d_x: f64 = (self.0 .1.x - c.x).into();
        let b_d_y: f64 = (self.0 .1.y - c.y).into();
        let c_d_x: f64 = (self.0 .2.x - c.x).into();
        let c_d_y: f64 = (self.0 .2.y - c.y).into();
        let a_d_x_d_y = a_d_x.powi(2) + a_d_y.powi(2);
        let b_d_x_d_y = b_d_x.powi(2) + b_d_y.powi(2);
        let c_d_x_d_y = c_d_x.powi(2) + c_d_y.powi(2);

        // Compute the determinant of the following matrix
        // [
        //     [a_d_x, a_d_y, a_d_x_d_y],
        //     [b_d_x, b_d_y, b_d_x_d_y],
        //     [c_d_x, c_d_y, c_d_x_d_y],
        // ]
        //
        let determinant = a_d_x * ((b_d_y * c_d_x_d_y) - (b_d_x_d_y * c_d_y))
            - a_d_y * ((b_d_x * c_d_x_d_y) - (b_d_x_d_y * c_d_x))
            + a_d_x_d_y * (b_d_x * c_d_y - b_d_y * c_d_x);

        determinant > 0.0
    }

    #[cfg(feature = "voronoi")]
    /// Get the center of the [Circumcircle](https://en.wikipedia.org/wiki/Circumcircle)
    /// for the Delaunay triangle.
    pub fn get_circumcircle_center(&self) -> Result<Coord<T>> {
        // Pin the triangle to the origin to simplify the calculation
        let b = self.0 .1 - self.0 .0;
        let c = self.0 .2 - self.0 .0;

        let a_x = self.0 .0.x;
        let a_y = self.0 .0.y;
        let b_x = b.x;
        let b_y = b.y;
        let c_x = c.x;
        let c_y = c.y;

        let d = T::from(2.0).ok_or(DelaunayTriangulationError::GeoTypeConversionError)?
            * (b_x * c_y - b_y * c_x);

        if d.is_zero() {
            return Err(DelaunayTriangulationError::FailedToComputeCircumcircle);
        }

        let u_x = (c_y * (b_x.powi(2) + b_y.powi(2)) - b_y * (c_x.powi(2) + c_y.powi(2))) / d;
        let u_y = (b_x * (c_x.powi(2) + c_y.powi(2)) - c_x * (b_x.powi(2) + b_y.powi(2))) / d;

        Ok(coord! {x: a_x + u_x, y: a_y + u_y})
    }
}

/// Delaunay Triangulation Errors
#[derive(Debug, PartialEq, Eq)]
pub enum DelaunayTriangulationError {
    /// Failed to compute the circumcircle for a given triangle.
    /// This can occur if the points are collinear.
    FailedToComputeCircumcircle,
    /// Failed to check if a point is in a circumcircle.
    FailedToCheckPointInCircumcircle,
    /// Failed to construct the super triangle.
    /// This error occurs when the `Polygon` describing the points to
    /// triangulate does not return a bounding rectangle.
    FailedToConstructSuperTriangle,
    FailedToConvertSuperTriangleFactor,
    GeoTypeConversionError,
}

impl fmt::Display for DelaunayTriangulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DelaunayTriangulationError::FailedToComputeCircumcircle => {
                write!(f, "Cannot compute circumcircle.")
            }
            DelaunayTriangulationError::FailedToCheckPointInCircumcircle => {
                write!(f, "Cannot determine if the point is in the circumcircle.")
            }
            DelaunayTriangulationError::FailedToConstructSuperTriangle => {
                write!(f, "Failed to construct the super triangle.")
            }
            DelaunayTriangulationError::GeoTypeConversionError => {
                write!(f, "Failed to convert from Geo type T")
            }
            DelaunayTriangulationError::FailedToConvertSuperTriangleFactor => {
                write!(f, "Failed to convert super triangle expansion factor")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Contains;
    use geo_types::{LineString, MultiPoint, Point};

    #[test]
    fn test_triangle_shares_vertex() {
        let triangle = DelaunayTriangle(Triangle::new(
            coord! {x: 0., y: 0.},
            coord! {x: 10., y: 20.},
            coord! {x: -12., y: -2.},
        ));
        let other = DelaunayTriangle(Triangle::new(
            coord! {x: 0., y: 0.},
            coord! {x: 30., y: 40.},
            coord! {x: 40., y: 30.},
        ));

        assert!(triangle.shares_vertex(&other));

        let other = DelaunayTriangle(Triangle::new(
            coord! {x: 30., y: 40.},
            coord! {x: 40., y: 30.},
            coord! {x: 50., y: 20.},
        ));

        assert!(!triangle.shares_vertex(&other));
    }

    #[test]
    fn test_triangle_is_neighbour() {
        let triangle = DelaunayTriangle(Triangle::new(
            coord! {x: 0., y: 0.},
            coord! {x: 10., y: 20.},
            coord! {x: -12., y: -2.},
        ));
        let other = DelaunayTriangle(Triangle::new(
            coord! {x: 0., y: 0.},
            coord! {x: 10., y: 20.},
            coord! {x: 30., y: 40.},
        ));

        assert_eq!(
            triangle.shares_edge(&other).unwrap(),
            Line::new(coord! {x: 0., y:0.}, coord! { x: 10., y: 20.})
        );

        let other = DelaunayTriangle(Triangle::new(
            coord! {x: 0., y: 0.},
            coord! {x: 30., y: 40.},
            coord! {x: 40., y: 50.},
        ));

        assert!(triangle.shares_edge(&other).is_none());
    }

    #[test]
    fn test_point_in_circumcircle() {
        let triangle = DelaunayTriangle(Triangle::new(
            coord! {x: 10., y: 10.},
            coord! {x: 30., y: 10.},
            coord! {x: 20., y: 20.},
        ));

        assert!(triangle.is_in_circumcircle(&coord! {x: 20., y: 10.}));
        assert!(!triangle.is_in_circumcircle(&coord! {x: 10., y: 30.}));
    }

    #[test]
    fn test_get_circumcircle() {
        let triangle = DelaunayTriangle(Triangle::new(
            coord! {x: 10., y: 10.},
            coord! {x: 20., y: 20.},
            coord! {x: 30., y: 10.},
        ));

        let circle_center = triangle.get_circumcircle_center().unwrap();
        approx::assert_relative_eq!(circle_center, coord! {x: 20., y: 10.});
    }

    #[test]
    fn test_get_circumcircle_collinear_points() {
        let triangle = DelaunayTriangle(Triangle::new(
            coord! {x: 10., y: 10.},
            coord! {x: 20., y: 20.},
            coord! {x: 30., y: 30.},
        ));

        // The circumcircle for collinear points cannot be
        // determined as the radius would be infinite
        triangle
            .get_circumcircle_center()
            .expect_err("Cannot compute circumcircle");
    }

    #[test]
    fn test_get_super_triangle() {
        let points: Polygon = Polygon::new(
            LineString::from(vec![(0., 0.), (1., 0.), (1., 1.), (0., 1.)]),
            vec![],
        );

        let super_tri = create_super_triangle(&points).unwrap();
        assert!(super_tri.contains(&points));
    }

    #[test]
    fn test_add_point() {
        // Create a super triangle
        let mut triangles = vec![DelaunayTriangle(Triangle::new(
            coord! {x: -20., y: 0.},
            coord! {x: 21., y: -20.},
            coord! {x: 21., y: 21.},
        ))];

        let expected_result = vec![
            DelaunayTriangle(Triangle::new(
                coord! {x: -20., y: 0.},
                coord! {x: 21., y: -20.},
                coord! {x: 0., y: 0.},
            )),
            DelaunayTriangle(Triangle::new(
                coord! {x: 21., y: -20.},
                coord! {x: 21., y: 21.},
                coord! {x: 0., y: 0.},
            )),
            DelaunayTriangle(Triangle::new(
                coord! {x: 21., y: 21.},
                coord! {x: -20., y: 0.},
                coord! {x: 0., y: 0.},
            )),
        ];

        add_coordinate(&mut triangles, &coord! {x: 0., y: 0.});

        assert_eq!(expected_result, triangles);
    }

    // Execute the geos tests

    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/DelaunayTest.cpp#L113
    #[test]
    fn test_triangle() {
        let points = MultiPoint::new(vec![
            Point::new(10., 10.),
            Point::new(10., 20.),
            Point::new(20., 20.),
        ]);

        let expected_triangle = vec![Triangle::new(
            coord! {x: 10.0, y: 20.},
            coord! {x: 10.0, y: 10.},
            coord! {x: 20.0, y: 20.},
        )];

        assert_eq!(points.delaunay_triangulation().unwrap(), expected_triangle);
    }

    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/DelaunayTest.cpp#L127
    #[test]
    fn test_random() {
        let points = MultiPoint::new(vec![
            Point::new(50., 40.),
            Point::new(140., 70.),
            Point::new(80., 100.),
            Point::new(130., 140.),
            Point::new(30., 150.),
            Point::new(70., 180.),
            Point::new(190., 110.),
            Point::new(120., 20.),
        ]);

        let expected_triangles = vec![
            Triangle::new(
                coord! {x: 50.0, y: 40.},
                coord! {x: 80.0, y: 100.},
                coord! {x: 30.0, y: 150.},
            ),
            Triangle::new(
                coord! {x: 30.0, y: 150.},
                coord! {x: 80.0, y: 100.},
                coord! {x: 70.0, y: 180.},
            ),
            Triangle::new(
                coord! {x: 80.0, y: 100.},
                coord! {x: 130.0, y: 140.},
                coord! {x: 70.0, y: 180.},
            ),
            Triangle::new(
                coord! {x: 70.0, y: 180.},
                coord! {x: 130.0, y: 140.},
                coord! {x: 190.0, y: 110.},
            ),
            Triangle::new(
                coord! {x: 130.0, y: 140.},
                coord! {x: 140.0, y: 70.},
                coord! {x: 190.0, y: 110.},
            ),
            Triangle::new(
                coord! {x: 190.0, y: 110.},
                coord! {x: 140.0, y: 70.},
                coord! {x: 120.0, y: 20.},
            ),
            Triangle::new(
                coord! {x: 140.0, y: 70.},
                coord! {x: 80.0, y: 100.},
                coord! {x: 120.0, y: 20.},
            ),
            Triangle::new(
                coord! {x: 80.0, y: 100.},
                coord! {x: 50.0, y: 40.},
                coord! {x: 120.0, y: 20.},
            ),
            Triangle::new(
                coord! {x: 80.0, y: 100.},
                coord! {x: 140.0, y: 70.},
                coord! {x: 130.0, y: 140.},
            ),
        ];

        let delaunay_triangles = points.delaunay_triangulation().unwrap();

        assert_eq!(delaunay_triangles.len(), expected_triangles.len());
        for tri in delaunay_triangles.iter() {
            assert!(expected_triangles.contains(tri));
        }
    }

    // Test grid
    // https://github.com/libgeos/geos/blob/d51982c6da5b7adb63ca0933ae7b53828cc8d72e/tests/unit/triangulate/DelaunayTest.cpp#L143
    #[test]
    fn test_grid() {
        let points = MultiPoint::new(vec![
            Point::new(10., 10.),
            Point::new(10., 20.),
            Point::new(20., 20.),
            Point::new(20., 10.),
            Point::new(10., 0.),
            Point::new(0., 0.),
            Point::new(0., 10.),
            Point::new(0., 20.),
        ]);

        let expected_triangles = vec![
            Triangle::new(
                coord! {x: 0.0, y: 0.},
                coord! {x: 10.0, y: 10.},
                coord! {x: 0.0, y: 10.},
            ),
            Triangle::new(
                coord! {x: 10.0, y: 0.},
                coord! {x: 10.0, y: 10.},
                coord! {x: 0.0, y: 0.},
            ),
            Triangle::new(
                coord! {x: 0.0, y: 10.},
                coord! {x: 10.0, y: 20.},
                coord! {x: 0.0, y: 20.},
            ),
            Triangle::new(
                coord! {x: 10.0, y: 10.},
                coord! {x: 10.0, y: 20.},
                coord! {x: 0.0, y: 10.},
            ),
            Triangle::new(
                coord! {x: 10.0, y: 20.},
                coord! {x: 10.0, y: 10.},
                coord! {x: 20.0, y: 20.},
            ),
            Triangle::new(
                coord! {x: 20.0, y: 20.},
                coord! {x: 10.0, y: 10.},
                coord! {x: 20.0, y: 10.},
            ),
            Triangle::new(
                coord! {x: 20.0, y: 10.},
                coord! {x: 10.0, y: 10.},
                coord! {x: 10.0, y: 0.},
            ),
        ];

        let delaunay_triangles = points.delaunay_triangulation().unwrap();

        assert_eq!(delaunay_triangles.len(), expected_triangles.len());
        for tri in delaunay_triangles.iter() {
            assert!(expected_triangles.contains(tri));
        }
    }

    #[test]
    fn test_polyon_that_visits_same_point_twice() {
        let points = MultiPoint::new(vec![
            Point::new(23., 3.1),
            Point::new(22., 3.1),
            Point::new(22., 2.),
            Point::new(22., 4.2),
            Point::new(23., 2.),
        ]);

        let delaunay_triangles = points.delaunay_triangulation().unwrap();

        let expected_triangles = vec![
            Triangle::new(
                coord! {x: 22., y: 3.1},
                coord! {x: 23., y: 3.1},
                coord! {x: 22., y: 4.2},
            ),
            Triangle::new(
                coord! {x: 23., y: 3.1},
                coord! {x: 22., y: 3.1},
                coord! {x: 23., y: 2.},
            ),
            Triangle::new(
                coord! {x: 22., y: 3.1},
                coord! {x: 22., y: 2.0},
                coord! {x: 23., y: 2.0},
            ),
        ];

        assert_eq!(delaunay_triangles.len(), expected_triangles.len());
        for tri in delaunay_triangles.iter() {
            assert!(expected_triangles.contains(tri));
        }
    }

    #[test]
    fn test_neg_0() {
        let zero: f64 = num_traits::Zero::zero();
        assert!(zero.is_finite());
    }
}
