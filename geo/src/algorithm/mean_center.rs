use crate::CoordsIter;
use geo_types::Point;
use num_traits::Zero;

pub trait MeanCenter {
    fn mean_center(&self) -> Point;
}

impl<T> MeanCenter for T
where
    T: CoordsIter<Scalar = f64>,
{
    fn mean_center(&self) -> Point {
        let mut x_sum = T::Scalar::zero();
        let mut y_sum = T::Scalar::zero();
        let mut n = 0_usize;

        for coord in self.coords_iter() {
            let (xi, yi) = coord.x_y();
            x_sum = xi + x_sum;
            y_sum = yi + y_sum;
            n += 1_usize;
        }

        let denominator = n as T::Scalar;

        let x = x_sum / denominator;
        let y = y_sum / denominator;

        Point::new(x, y)
    }
}

#[cfg(test)]
mod test {

    use super::*;
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

        let mean_center = MultiPoint::new(coords).mean_center();
        assert_eq!(mean_center, Point::new(0.5, 0.5));
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
    }

    #[test]
    fn test_mean_center_triangle() {
        let v1 = Coord { x: 0.0, y: 0.0 };
        let v2 = Coord { x: 1.0, y: 0.0 };
        let v3 = Coord { x: 0.0, y: 1.0 };

        let triangle = Triangle::new(v1, v2, v3);
        let mean_center = triangle.mean_center();
        assert_eq!(mean_center, Point::new(1.0 / 3.0, 1.0 / 3.0));
    }

    #[test]
    fn test_mean_center_rect() {
        let rect = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 });
        let mean_center = rect.mean_center();
        assert_eq!(mean_center, Point::new(0.5, 0.5));
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
        let mean_center = Geometry::LineString(lns).mean_center();
        assert_eq!(mean_center, Point::new(0.25, 0.25));
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

        let mean_center = GeometryCollection(vec![
            Geometry::Polygon(poly1),
            Geometry::Polygon(poly2),
            Geometry::MultiPolygon(mpoly),
        ])
        .mean_center();
        assert_eq!(mean_center, Point::new(1.25, 1.25));
    }
}
