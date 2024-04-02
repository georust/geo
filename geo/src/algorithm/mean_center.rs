use crate::{Centroid, CoordsIter};
use geo_types::{
    Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon, Rect, Triangle,
};

/// Calculate the (Weighted) Mean Center of a Geometry
///
/// The mean center of a geometry is a measure of central tendancy
/// of a set of geometry. It is calculated by taking the average
/// of all x and y values in the geometry. For [`Point`], [`Line`], [`LineString`],
/// [`Polygon`], [`Triangle`], and [`Rect`] this is a simple alias to [`Centroid`].
/// However, [`MultiLineString`], [`MultiPolygon`], and [`GeometryCollection`],
/// the mean center is found by taking the centroid of each geometry and
/// calculating the mean center of the centroids.
///
/// The weighted mean center applies a weight to each geometry. The weight is then
/// applied to the value of the coordinate used used in the calculation of the
/// mean center.
///
/// For `Line`, `Triangle`, `Rect`, `LineString`, `Polygon`, and `MultiPoint`, weights
/// are applied  to each coordinate in the geometry. For `MultiLineString`, `MultiPolygon`,
/// and `GeometryCollection`, the weights are applied to the centroid of each component
/// geometry. If a geometry does not have a centroid, it is filtered out.
///
/// The `weights` argument takes a slice which is used as an iterator which is cycled.
/// If there are fewer weights than coordinates, the weights will be recycled until the
/// calculation has completed.
///
// See [How Mean Center Works](https://pro.arcgis.com/en/pro-app/latest/tool-reference/spatial-statistics/mean-center.htm).
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
    type Output;
    /// Return the unweighted mean center of a geometry.
    fn mean_center(&self) -> Self::Output;

    /// Return the weighted mean center of a geometry.
    /// The weights are cycled if there are fewer weights than geometries.
    fn weighted_mean_center(&self, weights: &[f64]) -> Self::Output;
}

macro_rules! impl_mean_center_point {
    ($type:ty) => {
        impl MeanCenter for $type {
            type Output = Point;
            fn mean_center(&self) -> Self::Output {
                self.centroid()
            }

            fn weighted_mean_center(&self, weights: &[f64]) -> Self::Output {
                let mut x_sum = 0.0;
                let mut y_sum = 0.0;
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
    };
}

impl_mean_center_point!(Line);
impl_mean_center_point!(Point);
impl_mean_center_point!(Rect);
impl_mean_center_point!(Triangle);

macro_rules! impl_mean_center_option {
    ($type:ty) => {
        impl MeanCenter for $type {
            type Output = Option<Point>;
            fn mean_center(&self) -> Self::Output {
                self.centroid()
            }

            fn weighted_mean_center(&self, weights: &[f64]) -> Self::Output {
                let mut x_sum = 0.0;
                let mut y_sum = 0.0;
                let mut weight_sum = 0.0;
                let n = self.coords_count();
                if n == 0 {
                    return None;
                }

                for (coord, weight) in self.coords_iter().zip(weights.iter().cycle()) {
                    let (xi, yi) = coord.x_y();
                    x_sum += xi * weight;
                    y_sum += yi * weight;
                    weight_sum += weight;
                }

                let x = x_sum / weight_sum;
                let y = y_sum / weight_sum;

                Some(Point::new(x, y))
            }
        }
    };
}

impl_mean_center_option!(LineString);
impl_mean_center_option!(MultiPoint);

impl MeanCenter for Polygon {
    type Output = Option<Point>;
    fn mean_center(&self) -> Self::Output {
        self.centroid()
    }

    fn weighted_mean_center(&self, weights: &[f64]) -> Self::Output {
        let mut x_sum = 0.0;
        let mut y_sum = 0.0;
        let mut weight_sum = 0.0;
        let n = self.coords_count();

        if n == 0 {
            return None;
        }

        for ((i, coord), weight) in self.coords_iter().enumerate().zip(weights.iter().cycle()) {
            if i < n - 1 {
                let (xi, yi) = coord.x_y();
                x_sum += xi * weight;
                y_sum += yi * weight;
                weight_sum += weight;
            }
        }

        let x = x_sum / weight_sum;
        let y = y_sum / weight_sum;

        Some(Point::new(x, y))
    }
}

macro_rules! impl_mean_center_multi {
    ($type:ty) => {
        impl MeanCenter for $type {
            type Output = Option<Point>;
            fn mean_center(&self) -> Self::Output {
                let centroids = self
                    .iter()
                    .map(|p| p.centroid())
                    .filter(|c| c.is_some())
                    .map(|c| c.unwrap())
                    .collect::<Vec<_>>();

                // Create multipoint from the centroids and calculate the mean center
                MultiPoint::new(centroids).centroid()
            }

            fn weighted_mean_center(&self, weights: &[f64]) -> Self::Output {
                let centroids = self
                    .iter()
                    .map(|p| p.centroid())
                    .filter(|c| c.is_some())
                    .map(|c| c.unwrap())
                    .collect::<Vec<_>>();

                let n = centroids.len();

                if n == 0 {
                    return None;
                }

                // Create multipoint from the centroids and calculate the mean center
                MultiPoint::new(centroids).weighted_mean_center(weights)
            }
        }
    };
}

// Use the macro for MultiPolygon and MultiLineString
impl_mean_center_multi!(MultiPolygon);
impl_mean_center_multi!(MultiLineString);

impl MeanCenter for Geometry {
    type Output = Option<Point>;
    fn mean_center(&self) -> Self::Output {
        match self {
            Geometry::Point(p) => p.mean_center().into(),
            Geometry::Line(l) => l.mean_center().into(),
            Geometry::LineString(ls) => ls.mean_center(),
            Geometry::Polygon(p) => p.mean_center(),
            Geometry::MultiPoint(mp) => mp.mean_center(),
            Geometry::MultiLineString(mls) => mls.mean_center(),
            Geometry::MultiPolygon(mp) => mp.mean_center(),
            Geometry::GeometryCollection(gc) => gc.mean_center(),
            Geometry::Rect(r) => r.mean_center().into(),
            Geometry::Triangle(t) => t.mean_center().into(),
        }
    }

    fn weighted_mean_center(&self, weights: &[f64]) -> Self::Output {
        match self {
            Geometry::Point(p) => p.weighted_mean_center(weights).into(),
            Geometry::Line(l) => l.weighted_mean_center(weights).into(),
            Geometry::LineString(ls) => ls.weighted_mean_center(weights),
            Geometry::Polygon(p) => p.weighted_mean_center(weights),
            Geometry::MultiPoint(mp) => mp.weighted_mean_center(weights),
            Geometry::MultiLineString(mls) => mls.weighted_mean_center(weights),
            Geometry::MultiPolygon(mp) => mp.weighted_mean_center(weights),
            Geometry::GeometryCollection(gc) => gc.weighted_mean_center(weights),
            Geometry::Rect(r) => r.weighted_mean_center(weights).into(),
            Geometry::Triangle(t) => t.weighted_mean_center(weights).into(),
        }
    }
}

impl MeanCenter for GeometryCollection {
    type Output = Option<Point>;
    fn mean_center(&self) -> Self::Output {
        // take the centroid of each geometry
        // the mean center is calculated from the centroids
        // any geometry that does not have a centroid is filtered out
        // this should be rare
        let centroids = self
            .iter()
            .map(|g| g.centroid())
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .collect::<Vec<_>>();

        let n = centroids.len();

        if n == 0 {
            return None;
        }

        // Create multipoint from the centroids and calculate the mean center
        MultiPoint::new(centroids).centroid()
    }

    fn weighted_mean_center(&self, weights: &[f64]) -> Self::Output {
        let centroids = self
            .iter()
            .map(|g| g.centroid())
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .collect::<Vec<_>>();

        let n = centroids.len();

        if n == 0 {
            return None;
        }

        // Create multipoint from the centroids and calculate the mean center
        MultiPoint::new(centroids).weighted_mean_center(weights)
    }
}

// #[cfg(test)]
// mod test {

//     use super::*;
//     use crate::Centroid;
//     use geo_types::{
//         Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
//         Point, Polygon, Rect, Triangle,
//     };

//     #[test]
//     fn test_mean_center_point() {
//         let x = Point::new(1.0, 1.0);
//         assert_eq!(x.mean_center(), x);
//     }

//     #[test]
//     fn test_mean_center_multipoint() {
//         let coords: Vec<Point> = vec![
//             Coord { x: 0.0, y: 0.0 }.into(),
//             Coord { x: 1.0, y: 0.0 }.into(),
//             Coord { x: 0.0, y: 1.0 }.into(),
//             Coord { x: 1.0, y: 1.0 }.into(),
//         ];

//         let mpnt = MultiPoint::new(coords);
//         let mean_center = mpnt.mean_center();
//         assert_eq!(mean_center, Point::new(0.5, 0.5));
//         assert_eq!(mean_center, mpnt.centroid().unwrap());
//     }

//     #[test]
//     fn test_mean_center_linestring() {
//         let coords = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 1.0, y: 1.0 },
//         ];

//         let lns = LineString::new(coords);
//         let mean_center = lns.mean_center();
//         assert_eq!(mean_center, Point::new(0.5, 0.5));
//         assert_eq!(mean_center, lns.centroid().unwrap());
//     }

//     #[test]
//     fn test_mean_center_multilinestring() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 1.0, y: 1.0 },
//         ];

//         let coords2 = vec![
//             Coord {
//                 x: 0.0 + 1.0,
//                 y: 0.0,
//             },
//             Coord {
//                 x: 1.0 + 1.0,
//                 y: 0.0,
//             },
//             Coord {
//                 x: 0.0 + 1.0,
//                 y: 1.0,
//             },
//             Coord {
//                 x: 1.0 + 1.0,
//                 y: 1.0,
//             },
//         ];

//         let coords3 = vec![
//             Coord {
//                 x: 0.0 + 2.0,
//                 y: 0.0 + 3.0,
//             },
//             Coord {
//                 x: 1.0 + 2.0,
//                 y: 0.0 + 3.0,
//             },
//             Coord {
//                 x: 0.0 + 2.0,
//                 y: 1.0 + 3.0,
//             },
//             Coord {
//                 x: 1.0 + 2.0,
//                 y: 1.0 + 3.0,
//             },
//         ];

//         let lns1 = LineString::new(coords1);
//         let lns2 = LineString::new(coords2);
//         let lns3 = LineString::new(coords3);

//         let mlns = MultiLineString(vec![lns1, lns2, lns3]);

//         let mean_center = mlns.mean_center();
//         assert_eq!(mean_center, Point::new(1.5, 1.5));
//         assert_eq!(mean_center, mlns.centroid().unwrap());
//     }

//     #[test]
//     fn test_mean_center_polygon() {
//         let coords = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 3.0, y: 0.0 },
//             Coord { x: 0.0, y: 3.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let poly = Polygon::new(LineString::new(coords), vec![]);
//         let mean_center = poly.mean_center();

//         assert_eq!(mean_center, Point::new(1.0, 1.0));
//     }

//     #[test]
//     fn test_mean_center_multipolygon() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let coords2 = vec![
//             Coord { x: 2.0, y: 2.0 },
//             Coord { x: 3.0, y: 2.0 },
//             Coord { x: 2.0, y: 3.0 },
//             Coord { x: 2.0, y: 2.0 },
//         ];

//         let poly1 = Polygon::new(LineString::new(coords1), vec![]);
//         let poly2 = Polygon::new(LineString::new(coords2), vec![]);

//         let mpoly = MultiPolygon(vec![poly1, poly2]);
//         let mean_center = mpoly.mean_center();

//         let one_one_third = 1.0 + 1.0 / 3.0;

//         assert_eq!(mean_center, Point::new(one_one_third, one_one_third));
//         assert_ne!(mean_center, mpoly.centroid().unwrap());
//     }

//     #[test]
//     fn test_mean_center_triangle() {
//         let v1 = Coord { x: 0.0, y: 0.0 };
//         let v2 = Coord { x: 1.0, y: 0.0 };
//         let v3 = Coord { x: 0.0, y: 1.0 };

//         let triangle = Triangle::new(v1, v2, v3);
//         let mean_center = triangle.mean_center();
//         assert_eq!(mean_center, Point::new(1.0 / 3.0, 1.0 / 3.0));
//         assert_eq!(mean_center, triangle.centroid());
//     }

//     #[test]
//     fn test_mean_center_rect() {
//         let rect = Rect::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 1.0, y: 1.0 });
//         let mean_center = rect.mean_center();
//         assert_eq!(mean_center, Point::new(0.5, 0.5));
//         assert_eq!(mean_center, rect.centroid());
//     }

//     #[test]
//     fn test_mean_center_geometry() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let lns = LineString::new(coords1);
//         let mean_center = Geometry::LineString(lns.clone()).mean_center();
//         assert_eq!(mean_center, Point::new(0.25, 0.25));
//         assert_ne!(mean_center, lns.centroid().unwrap());
//     }

//     #[test]
//     fn test_mean_center_geomcollection() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let coords2 = vec![
//             Coord { x: 2.0, y: 2.0 },
//             Coord { x: 3.0, y: 2.0 },
//             Coord { x: 2.0, y: 3.0 },
//             Coord { x: 2.0, y: 2.0 },
//         ];

//         let poly1 = Polygon::new(LineString::new(coords1), vec![]);
//         let poly2 = Polygon::new(LineString::new(coords2), vec![]);

//         let mpoly = MultiPolygon(vec![poly1.clone(), poly2.clone()]);

//         let geom_collection = GeometryCollection(vec![
//             Geometry::Polygon(poly1),
//             Geometry::Polygon(poly2),
//             Geometry::MultiPolygon(mpoly),
//         ]);

//         let mean_center = geom_collection.mean_center();

//         let one_one_third = 1.0 + 1.0 / 3.0;
//         assert_eq!(mean_center, Point::new(one_one_third, one_one_third));
//         assert_ne!(mean_center, geom_collection.centroid().unwrap());
//     }

//     #[test]
//     fn test_weighted_mean_center_point() {
//         let x = Point::new(1.0, 1.0);
//         // any scalar returns the mean center
//         assert_eq!(x.weighted_mean_center(&[std::f64::consts::PI]), x);
//     }

//     #[test]
//     fn test_weighted_mean_center_multipoint() {
//         let coords: Vec<Point> = vec![
//             Coord { x: 0.0, y: 0.0 }.into(),
//             Coord { x: 1.0, y: 0.0 }.into(),
//             Coord { x: 0.0, y: 1.0 }.into(),
//             Coord { x: 1.0, y: 1.0 }.into(),
//         ];

//         let mpnt = MultiPoint::new(coords);
//         let weighted_center = mpnt.weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
//         assert_eq!(weighted_center, Point::new(0.6, 0.8));
//     }

//     #[test]
//     fn test_weighted_mean_center_linestring() {
//         let coords = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 1.0, y: 1.0 },
//         ];

//         let lns = LineString::new(coords);
//         let weighted_center = lns.weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
//         assert_eq!(weighted_center, Point::new(0.6, 0.8));
//     }

//     #[test]
//     fn test_weighted_mean_center_multilinestring() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 1.0, y: 1.0 },
//         ];

//         let coords2 = vec![
//             Coord {
//                 x: 0.0 + 1.0,
//                 y: 0.0,
//             },
//             Coord {
//                 x: 1.0 + 1.0,
//                 y: 0.0,
//             },
//             Coord {
//                 x: 0.0 + 1.0,
//                 y: 1.0,
//             },
//             Coord {
//                 x: 1.0 + 1.0,
//                 y: 1.0,
//             },
//         ];

//         let coords3 = vec![
//             Coord {
//                 x: 0.0 + 2.0,
//                 y: 0.0 + 3.0,
//             },
//             Coord {
//                 x: 1.0 + 2.0,
//                 y: 0.0 + 3.0,
//             },
//             Coord {
//                 x: 0.0 + 2.0,
//                 y: 1.0 + 3.0,
//             },
//             Coord {
//                 x: 1.0 + 2.0,
//                 y: 1.0 + 3.0,
//             },
//         ];

//         let lns1 = LineString::new(coords1);
//         let lns2 = LineString::new(coords2);
//         let lns3 = LineString::new(coords3);

//         let mlns = MultiLineString(vec![lns1, lns2, lns3]);

//         // weights are cycled. Each linestring is getting the same weight since they
//         // are composed of 4 coords each
//         let weighted_center = mlns.weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
//         assert_eq!(weighted_center, Point::new(1.6, 1.8));
//     }

//     #[test]
//     fn test_weighted_mean_center_polygon() {
//         let coords = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let lns = LineString::new(coords);
//         let weighted_center = lns.weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
//         assert_eq!(weighted_center, Point::new(0.2, 0.4));
//     }

//     #[test]
//     fn test_weighted_mean_center_multipolygon() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let coords2 = vec![
//             Coord { x: 2.0, y: 2.0 },
//             Coord { x: 3.0, y: 2.0 },
//             Coord { x: 2.0, y: 3.0 },
//             Coord { x: 2.0, y: 2.0 },
//         ];

//         let poly1 = Polygon::new(LineString::new(coords1), vec![]);
//         let poly2 = Polygon::new(LineString::new(coords2), vec![]);

//         let mpoly = MultiPolygon(vec![poly1, poly2]);

//         // weights are cycled. Each polygon is getting the same weight since they
//         // are composed of 4 coords each
//         let weighted_center = mpoly.weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
//         assert_eq!(weighted_center, Point::new(1.6, 1.8));
//     }

//     #[test]
//     fn test_weighted_mean_center_geometry() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let lns = LineString::new(coords1);
//         let weighted_center =
//             Geometry::LineString(lns.clone()).weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
//         assert_eq!(weighted_center, Point::new(0.6, 0.8));
//     }

//     #[test]
//     fn test_weighted_mean_cneter_geomcollection() {
//         let coords1 = vec![
//             Coord { x: 0.0, y: 0.0 },
//             Coord { x: 1.0, y: 0.0 },
//             Coord { x: 0.0, y: 1.0 },
//             Coord { x: 0.0, y: 0.0 },
//         ];

//         let coords2 = vec![
//             Coord { x: 2.0, y: 2.0 },
//             Coord { x: 3.0, y: 2.0 },
//             Coord { x: 2.0, y: 3.0 },
//             Coord { x: 2.0, y: 2.0 },
//         ];

//         let poly1 = Polygon::new(LineString::new(coords1), vec![]);
//         let poly2 = Polygon::new(LineString::new(coords2), vec![]);

//         let mpoly = MultiPolygon(vec![poly1.clone(), poly2.clone()]);

//         let geom_collection = GeometryCollection(vec![
//             Geometry::Polygon(poly1),
//             Geometry::Polygon(poly2),
//             Geometry::MultiPolygon(mpoly),
//         ]);

//         // weights are cycled. Each polygon is getting the same weight since they
//         // are composed of 4 coords each
//         let weighted_center = geom_collection.weighted_mean_center(&[0.0, 1.0, 2.0, 2.0]);
//         assert_eq!(weighted_center, Point::new(1.6, 1.8));
//     }

//     #[test]
//     fn test_weighted_mean_center_triangle() {
//         let v1 = Coord { x: 0.0, y: 0.0 };
//         let v2 = Coord { x: 1.0, y: 0.0 };
//         let v3 = Coord { x: 0.0, y: 1.0 };

//         let triangle = Triangle::new(v1, v2, v3);
//         let weighted_center = triangle.weighted_mean_center(&[1.0, 2.0, 3.0]);
//         assert_eq!(weighted_center, Point::new(1.0 / 2.0, 1.0 / 2.0));
//     }

//     #[test]
//     fn test_weighted_mean_center_rect() {}

//     #[test]
//     fn test_weighted_mean_center_line() {}
// }
