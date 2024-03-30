use crate::CoordsIter;
use geo_types::Point;
use num_traits::Zero;

/// Calculate the mean center of a geometry.
///
/// The mean center of a geometry is a measure of central tendancy
/// of a set of coordinates. It is calculated by taking the average
/// of all x and y values in the set.
///
/// The weighted mean center applies a weight to each coordinate and is
/// used in the calculation of the center.
///
///
///
/// ```rust
/// # use geo::MeanCenter;
/// # use geo::{Point, MultiPoint, Coord};
/// let coords: Vec<Point> = vec![
///     Coord { x: 0.0, y: 0.0 }.into(),
///     Coord { x: 1.0, y: 0.0 }.into(),
///     Coord { x: 0.0, y: 1.0 }.into(),
///     Coord { x: 1.0, y: 1.0 }.into(),
/// ];
///
/// let mpnt = MultiPoint::new(coords);
/// let mean_center = mpnt.mean_center();
/// assert_eq!(mean_center, Point::new(0.5, 0.5));
///
/// let weighted_center = mpnt.weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
/// assert_eq!(weighted_center, Point::new(0.6, 0.8));
/// ```
pub trait MeanCenter {
    /// Return the unweighted mean center of a geometry.
    fn mean_center(&self) -> Point;

    /// Return the weighted mean center of a geometry.
    /// The weights are cycled if there are fewer weights than coordinates.
    fn weighted_mean_center(&self, weights: &[f64]) -> Point;
}

impl<T> MeanCenter for T
where
    T: CoordsIter<Scalar = f64>,
{
    fn mean_center(&self) -> Point {
        let mut x_sum = T::Scalar::zero();
        let mut y_sum = T::Scalar::zero();
        let n = self.coords_count();

        for coord in self.coords_iter() {
            let (xi, yi) = coord.x_y();
            x_sum += xi;
            y_sum += yi;
        }

        let denominator = n as T::Scalar;

        let x = x_sum / denominator;
        let y = y_sum / denominator;

        Point::new(x, y)
    }

    fn weighted_mean_center(&self, weights: &[f64]) -> Point {
        let mut x_sum = T::Scalar::zero();
        let mut y_sum = T::Scalar::zero();
        let mut weight_sum = 0.0;

        for (coord, weight) in self.coords_iter().zip(weights.iter().cycle()) {
            let (xi, yi) = coord.x_y();
            x_sum += xi * weight;
            y_sum += yi * weight;
            weight_sum += weight;
        }

        let x = x_sum / weight_sum;
        let y = y_sum / weight_sum;

        Point::new(x, y)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::Centroid;
    use geo_types::{
        Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
        Point, Polygon, Rect, Triangle,
    };

    #[test]
    fn test_mean_center_point() {
        let x = Point::new(1.0, 1.0);
        assert_eq!(x.mean_center(), x);
    }

    #[test]
    fn test_mean_center_multipoint() {
        let coords: Vec<Point> = vec![
            Coord { x: 0.0, y: 0.0 }.into(),
            Coord { x: 1.0, y: 0.0 }.into(),
            Coord { x: 0.0, y: 1.0 }.into(),
            Coord { x: 1.0, y: 1.0 }.into(),
        ];

        let mpnt = MultiPoint::new(coords);
        let mean_center = mpnt.mean_center();
        assert_eq!(mean_center, Point::new(0.5, 0.5));
        assert_eq!(mean_center, mpnt.centroid().unwrap());
    }

    #[test]
    fn test_mean_center_linestring() {
        let coords = vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
            Coord { x: 1.0, y: 1.0 },
        ];

        let lns = LineString::new(coords);
        let mean_center = lns.mean_center();
        assert_eq!(mean_center, Point::new(0.5, 0.5));
        assert_eq!(mean_center, lns.centroid().unwrap());
    }

    #[test]
    fn test_mean_center_multilinestring() {
        let coords1 = vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
            Coord { x: 1.0, y: 1.0 },
        ];

        let coords2 = vec![
            Coord {
                x: 0.0 + 1.0,
                y: 0.0,
            },
            Coord {
                x: 1.0 + 1.0,
                y: 0.0,
            },
            Coord {
                x: 0.0 + 1.0,
                y: 1.0,
            },
            Coord {
                x: 1.0 + 1.0,
                y: 1.0,
            },
        ];

        let coords3 = vec![
            Coord {
                x: 0.0 + 2.0,
                y: 0.0 + 3.0,
            },
            Coord {
                x: 1.0 + 2.0,
                y: 0.0 + 3.0,
            },
            Coord {
                x: 0.0 + 2.0,
                y: 1.0 + 3.0,
            },
            Coord {
                x: 1.0 + 2.0,
                y: 1.0 + 3.0,
            },
        ];

        let lns1 = LineString::new(coords1);
        let lns2 = LineString::new(coords2);
        let lns3 = LineString::new(coords3);

        let mlns = MultiLineString(vec![lns1, lns2, lns3]);

        let mean_center = mlns.mean_center();
        assert_eq!(mean_center, Point::new(1.5, 1.5));
        assert_eq!(mean_center, mlns.centroid().unwrap());
    }

    #[test]
    fn test_mean_center_polygon() {
        let coords = vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
            Coord { x: 0.0, y: 0.0 },
        ];

        let lns = LineString::new(coords);
        let mean_center = lns.mean_center();
        assert_eq!(mean_center, Point::new(0.25, 0.25));
        assert_ne!(mean_center, lns.centroid().unwrap());
    }

    #[test]
    fn test_mean_center_multipolygon() {
        let coords1 = vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
            Coord { x: 0.0, y: 0.0 },
        ];

        let coords2 = vec![
            Coord { x: 2.0, y: 2.0 },
            Coord { x: 3.0, y: 2.0 },
            Coord { x: 2.0, y: 3.0 },
            Coord { x: 2.0, y: 2.0 },
        ];

        let poly1 = Polygon::new(LineString::new(coords1), vec![]);
        let poly2 = Polygon::new(LineString::new(coords2), vec![]);

        let mpoly = MultiPolygon(vec![poly1, poly2]);

        let mean_center = mpoly.mean_center();
        assert_eq!(mean_center, Point::new(1.25, 1.25));
        assert_ne!(mean_center, mpoly.centroid().unwrap());
    }

    #[test]
    fn test_mean_center_triangle() {
        let v1 = Coord { x: 0.0, y: 0.0 };
        let v2 = Coord { x: 1.0, y: 0.0 };
        let v3 = Coord { x: 0.0, y: 1.0 };

        let triangle = Triangle::new(v1, v2, v3);
        let mean_center = triangle.mean_center();
        assert_eq!(mean_center, Point::new(1.0 / 3.0, 1.0 / 3.0));
        assert_eq!(mean_center, triangle.centroid());
    }

    #[test]
    fn test_mean_center_rect() {
        let rect = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 });
        let mean_center = rect.mean_center();
        assert_eq!(mean_center, Point::new(0.5, 0.5));
        assert_eq!(mean_center, rect.centroid());
    }

    #[test]
    fn test_mean_center_geometry() {
        let coords1 = vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
            Coord { x: 0.0, y: 0.0 },
        ];

        let lns = LineString::new(coords1);
        let mean_center = Geometry::LineString(lns.clone()).mean_center();
        assert_eq!(mean_center, Point::new(0.25, 0.25));
        assert_ne!(mean_center, lns.centroid().unwrap());
    }

    #[test]
    fn test_mean_center_geomcollection() {
        let coords1 = vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
            Coord { x: 0.0, y: 0.0 },
        ];

        let coords2 = vec![
            Coord { x: 2.0, y: 2.0 },
            Coord { x: 3.0, y: 2.0 },
            Coord { x: 2.0, y: 3.0 },
            Coord { x: 2.0, y: 2.0 },
        ];

        let poly1 = Polygon::new(LineString::new(coords1), vec![]);
        let poly2 = Polygon::new(LineString::new(coords2), vec![]);

        let mpoly = MultiPolygon(vec![poly1.clone(), poly2.clone()]);

        let geom_collection = GeometryCollection(vec![
            Geometry::Polygon(poly1),
            Geometry::Polygon(poly2),
            Geometry::MultiPolygon(mpoly),
        ]);

        let mean_center = geom_collection.mean_center();

        assert_eq!(mean_center, Point::new(1.25, 1.25));
        assert_ne!(mean_center, geom_collection.centroid().unwrap());
    }
}
