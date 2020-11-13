use crate::algorithm::convex_hull::qhull;
use crate::algorithm::euclidean_distance::EuclideanDistance;
use crate::algorithm::euclidean_length::EuclideanLength;
use crate::algorithm::kernels::HasKernel;
use crate::prelude::Centroid;
use crate::utils::partial_min;
use crate::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
use geo_types::{Coordinate, CoordinateType};
use num_traits::Float;
use rstar::{RTree, RTreeNum};
use std::collections::VecDeque;

/// Returns a polygon which covers a geometry. Unlike convex hulls, which also cover
/// their geometry, a concave hull does so while trying to further minimize its area by
/// constructing edges such that the exterior of the polygon incorporates points that would
/// be interior points in a convex hull.
///
/// This implementation is inspired by https://github.com/mapbox/concaveman
/// and also uses ideas from the following paper:
/// www.iis.sinica.edu.tw/page/jise/2012/201205_10.pdf
///
/// # Examples
/// ```
/// use geo::{line_string, polygon};
/// use geo::algorithm::concave_hull::ConcaveHull;
///
/// // a square shape
/// let poly = polygon![
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 4.0),
///     (x: 0.0, y: 4.0),
/// ];
///
/// // The correct concave hull coordinates
/// let correct_hull = line_string![
///     (x: 4.0, y: 0.0),
///     (x: 4.0, y: 4.0),
///     (x: 0.0, y: 4.0),
///     (x: 0.0, y: 0.0),
///     (x: 4.0, y: 0.0),
/// ];
///
/// let res = poly.concave_hull(2.0);
/// assert_eq!(res.exterior(), &correct_hull);
/// ```
pub trait ConcaveHull {
    type Scalar: CoordinateType;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar>;
}

impl<T> ConcaveHull for Polygon<T>
where
    T: Float + RTreeNum + HasKernel,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar> {
        let mut points: Vec<_> = self.exterior().0.clone();
        Polygon::new(concave_hull(&mut points, concavity), vec![])
    }
}

impl<T> ConcaveHull for MultiPolygon<T>
where
    T: Float + RTreeNum + HasKernel,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar> {
        let mut aggregated: Vec<Coordinate<Self::Scalar>> = self
            .0
            .iter()
            .flat_map(|elem| elem.exterior().0.clone())
            .collect();
        Polygon::new(concave_hull(&mut aggregated, concavity), vec![])
    }
}

impl<T> ConcaveHull for LineString<T>
where
    T: Float + RTreeNum + HasKernel,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: Self::Scalar) -> Polygon<Self::Scalar> {
        Polygon::new(concave_hull(&mut self.0.clone(), concavity), vec![])
    }
}

impl<T> ConcaveHull for MultiLineString<T>
where
    T: Float + RTreeNum + HasKernel,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        let mut aggregated: Vec<Coordinate<T>> =
            self.iter().flat_map(|elem| elem.0.clone()).collect();
        Polygon::new(concave_hull(&mut aggregated, concavity), vec![])
    }
}

impl<T> ConcaveHull for MultiPoint<T>
where
    T: Float + RTreeNum + HasKernel,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        let mut coordinates: Vec<Coordinate<T>> = self.iter().map(|point| point.0).collect();
        Polygon::new(concave_hull(&mut coordinates, concavity), vec![])
    }
}

fn find_point_closest_to_line<T>(
    interior_coords_tree: &RTree<Coordinate<T>>,
    line: Line<T>,
    max_dist: T,
    edge_length: T,
    concavity: T,
    line_tree: &RTree<Line<T>>,
) -> Option<Coordinate<T>>
where
    T: Float + RTreeNum,
{
    let h = max_dist + max_dist;
    let w = line.euclidean_length() + h;
    let two = T::add(T::one(), T::one());
    let search_dist = T::div(T::sqrt(T::powi(w, 2) + T::powi(h, 2)), two);
    let centroid = line.centroid();
    let centroid_coord = Coordinate {
        x: centroid.x(),
        y: centroid.y(),
    };
    let mut candidates = interior_coords_tree
        .locate_within_distance(centroid_coord, search_dist)
        .peekable();
    let peek = candidates.peek();
    match peek {
        None => None,
        Some(&point) => {
            let closest_point =
                candidates.fold(Point::new(point.x, point.y), |acc_point, candidate| {
                    let candidate_point = Point::new(candidate.x, candidate.y);
                    if line.euclidean_distance(&acc_point)
                        > line.euclidean_distance(&candidate_point)
                    {
                        candidate_point
                    } else {
                        acc_point
                    }
                });
            let mut edges_nearby_point = line_tree
                .locate_within_distance(closest_point, search_dist)
                .peekable();
            let peeked_edge = edges_nearby_point.peek();
            let closest_edge_option = match peeked_edge {
                None => None,
                Some(&edge) => Some(edges_nearby_point.fold(*edge, |acc, candidate| {
                    if closest_point.euclidean_distance(&acc)
                        > closest_point.euclidean_distance(candidate)
                    {
                        *candidate
                    } else {
                        acc
                    }
                })),
            };
            let decision_distance = partial_min(
                closest_point.euclidean_distance(&line.start_point()),
                closest_point.euclidean_distance(&line.end_point()),
            );
            if let Some(closest_edge) = closest_edge_option {
                let far_enough = edge_length / decision_distance > concavity;
                let are_edges_equal = closest_edge == line;
                if far_enough && are_edges_equal {
                    Some(Coordinate {
                        x: closest_point.x(),
                        y: closest_point.y(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

// This takes significant inspiration from:
// https://github.com/mapbox/concaveman/blob/54838e1/index.js#L11
fn concave_hull<T>(coords: &mut [Coordinate<T>], concavity: T) -> LineString<T>
where
    T: Float + RTreeNum + HasKernel,
{
    let hull = qhull::quick_hull(coords);

    if coords.len() < 4 {
        return hull;
    }

    //Get points in overall dataset that aren't on the exterior linestring of the hull
    let hull_tree: RTree<Coordinate<T>> = RTree::bulk_load(hull.clone().0);

    let interior_coords: Vec<Coordinate<T>> = coords
        .iter()
        .filter(|coord| !hull_tree.contains(coord))
        .map(|coord| *coord)
        .collect();
    let mut interior_points_tree: RTree<Coordinate<T>> = RTree::bulk_load(interior_coords);
    let mut line_tree: RTree<Line<T>> = RTree::new();

    let mut concave_list: Vec<Point<T>> = vec![];
    let lines = hull.lines();
    let mut line_queue: VecDeque<Line<T>> = VecDeque::new();

    for line in lines {
        line_queue.push_back(line.clone());
        line_tree.insert(line);
    }
    while let Some(line) = line_queue.pop_front() {
        let edge_length = line.euclidean_length();
        let dist = edge_length / concavity;
        let possible_closest_point = find_point_closest_to_line(
            &interior_points_tree,
            line,
            dist,
            edge_length,
            concavity,
            &line_tree,
        );

        if let Some(closest_point) = possible_closest_point {
            interior_points_tree.remove(&closest_point);
            line_tree.remove(&line);
            let point = Point::new(closest_point.x, closest_point.y);
            let start_line = Line::new(line.start_point(), point);
            let end_line = Line::new(point, line.end_point());
            line_tree.insert(start_line);
            line_tree.insert(end_line);
            line_queue.push_front(end_line);
            line_queue.push_front(start_line);
        } else {
            // Make sure we don't add duplicates
            if concave_list.is_empty() || !concave_list.ends_with(&[line.start_point()]) {
                concave_list.push(line.start_point());
            }
            concave_list.push(line.end_point());
        }
    }

    return concave_list.into();
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Point;
    use crate::{line_string, polygon};
    use geo_types::Coordinate;

    #[test]
    fn triangle_test() {
        let mut triangle = vec![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 4.0, y: 0.0 },
            Coordinate { x: 2.0, y: 2.0 },
        ];

        let correct = line_string![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 2.0, y: 2.0),
            (x: 0.0, y: 0.0),
        ];

        let concavity = 2.0;
        let res = concave_hull(&mut triangle, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn square_test() {
        let mut square = vec![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 4.0, y: 0.0 },
            Coordinate { x: 4.0, y: 4.0 },
            Coordinate { x: 0.0, y: 4.0 },
        ];

        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 0.0, y: 4.0),
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];

        let concavity = 2.0;
        let res = concave_hull(&mut square, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn one_flex_test() {
        let mut v = vec![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 2.0, y: 1.0 },
            Coordinate { x: 4.0, y: 0.0 },
            Coordinate { x: 4.0, y: 4.0 },
            Coordinate { x: 0.0, y: 4.0 },
        ];
        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 0.0, y: 4.0),
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 4.0, y: 0.0),
        ];
        let concavity = 1.0;
        let res = concave_hull(&mut v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn four_flex_test() {
        let mut v = vec![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 2.0, y: 1.0 },
            Coordinate { x: 4.0, y: 0.0 },
            Coordinate { x: 3.0, y: 2.0 },
            Coordinate { x: 4.0, y: 4.0 },
            Coordinate { x: 2.0, y: 3.0 },
            Coordinate { x: 0.0, y: 4.0 },
            Coordinate { x: 1.0, y: 2.0 },
        ];
        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 3.0, y: 2.0),
            (x: 4.0, y: 4.0),
            (x: 2.0, y: 3.0),
            (x: 0.0, y: 4.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 4.0, y: 0.0),
        ];
        let concavity = 1.7;
        let res = concave_hull(&mut v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn consecutive_flex_test() {
        let mut v = vec![
            Coordinate { x: 0.0, y: 0.0 },
            Coordinate { x: 4.0, y: 0.0 },
            Coordinate { x: 4.0, y: 4.0 },
            Coordinate { x: 3.0, y: 1.0 },
            Coordinate { x: 3.0, y: 2.0 },
        ];
        let correct = line_string![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 2.0),
            (x: 3.0, y: 1.0),
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];
        let concavity = 2.0;
        let res = concave_hull(&mut v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn concave_hull_norway_test() {
        let loaded_norway = include!("test_fixtures/norway_main.rs");
        let norway: MultiPoint<f64> = loaded_norway
            .iter()
            .map(|tuple| Point::new(f64::from(tuple[0]), f64::from(tuple[1])))
            .collect();
        let loaded_norway_concave_hull = include!("test_fixtures/norway_concave_hull.rs");
        let norway_concave_hull_points: Vec<Point<f64>> = loaded_norway_concave_hull
            .iter()
            .map(|tuple| Point::new(f64::from(tuple[0]), f64::from(tuple[1])))
            .collect();
        let norway_concave_hull: LineString<f64> = norway_concave_hull_points.into_iter().collect();
        let res = norway.concave_hull(2.0);
        assert_eq!(res.exterior(), &norway_concave_hull);
    }

    #[test]
    fn concave_hull_linestring_test() {
        let linestring = line_string![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 1.0),
            (x: 3.0, y: 2.0)
        ];
        let concave = linestring.concave_hull(2.0);
        let correct = vec![
            Coordinate::from((4.0, 0.0)),
            Coordinate::from((4.0, 4.0)),
            Coordinate::from((3.0, 2.0)),
            Coordinate::from((3.0, 1.0)),
            Coordinate::from((0.0, 0.0)),
            Coordinate::from((4.0, 0.0)),
        ];
        assert_eq!(concave.exterior().0, correct);
    }

    #[test]
    fn concave_hull_multilinestring_test() {
        let v1 = line_string![
             (x: 0.0, y: 0.0),
             (x: 4.0, y: 0.0)
        ];
        let v2 = line_string![
             (x: 4.0, y: 4.0),
             (x: 3.0, y: 1.0),
             (x: 3.0, y: 2.0)
        ];
        let mls = MultiLineString(vec![v1, v2]);
        let correct = vec![
            Coordinate::from((4.0, 0.0)),
            Coordinate::from((4.0, 4.0)),
            Coordinate::from((3.0, 2.0)),
            Coordinate::from((3.0, 1.0)),
            Coordinate::from((0.0, 0.0)),
            Coordinate::from((4.0, 0.0)),
        ];
        let res = mls.concave_hull(2.0);
        assert_eq!(res.exterior().0, correct);
    }

    #[test]
    fn concave_hull_multipolygon_test() {
        let v1 = polygon![
             (x: 0.0, y: 0.0),
             (x: 4.0, y: 0.0)
        ];
        let v2 = polygon![
             (x: 4.0, y: 4.0),
             (x: 3.0, y: 1.0),
             (x: 3.0, y: 2.0)
        ];
        let multipolygon = MultiPolygon(vec![v1, v2]);
        let res = multipolygon.concave_hull(2.0);
        let correct = vec![
            Coordinate::from((4.0, 0.0)),
            Coordinate::from((4.0, 4.0)),
            Coordinate::from((3.0, 2.0)),
            Coordinate::from((3.0, 1.0)),
            Coordinate::from((0.0, 0.0)),
            Coordinate::from((4.0, 0.0)),
        ];
        assert_eq!(res.exterior().0, correct);
    }
}
