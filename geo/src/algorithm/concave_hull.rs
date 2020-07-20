use crate::algorithm::convexhull::ConvexHull;
use crate::algorithm::euclidean_distance::EuclideanDistance;
use crate::algorithm::euclidean_length::EuclideanLength;
use crate::prelude::Centroid;
use crate::utils::partial_min;
use crate::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
use num_traits::Float;
use rstar::{RTree, RTreeNum};
use std::collections::VecDeque;
use std::ops::Add;

pub trait ConcaveHull<T> {
    /// Returns a polygon which covers a geometry. Unlike convex hulls, which also cover
    /// their geometry, a concave hull does so while trying to further minimize its area by
    /// constructing edges such that the exterior of the polygon incorporates points that would
    /// be interior points in a convex hull.
    ///
    /// This implementation is inspired by https://github.com/mapbox/concaveman
    /// and also uses ideas from the following paper:
    /// www.iis.sinica.edu.tw/page/jise/2012/201205_10.pdf
    ///
    /// # Example
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
    fn concave_hull(&self, concavity: T) -> Polygon<T>
    where
        T: Float + RTreeNum;
}

impl<T> ConcaveHull<T> for Polygon<T>
where
    T: Float + RTreeNum,
{
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        let points: Vec<_> = self.exterior().points_iter().collect();
        Polygon::new(LineString::from(concave_hull(&points, concavity)), vec![])
    }
}

impl<T> ConcaveHull<T> for MultiPolygon<T>
where
    T: Float + RTreeNum,
{
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        let aggregated: Vec<Point<T>> = self
            .0
            .iter()
            .flat_map(|elem| elem.exterior().0.iter().map(|c| Point(*c)))
            .collect();
        Polygon::new(
            LineString::from(concave_hull(&aggregated, concavity)),
            vec![],
        )
    }
}

impl<T> ConcaveHull<T> for LineString<T>
where
    T: Float + RTreeNum,
{
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        Polygon::new(
            LineString::from(concave_hull(&self.clone().into_points(), concavity)),
            vec![],
        )
    }
}

impl<T> ConcaveHull<T> for MultiLineString<T>
where
    T: Float + RTreeNum,
{
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        let aggregated: Vec<Point<T>> = self
            .0
            .iter()
            .flat_map(|elem| elem.clone().into_points())
            .collect();
        Polygon::new(
            LineString::from(concave_hull(&aggregated, concavity)),
            vec![],
        )
    }
}

impl<T> ConcaveHull<T> for MultiPoint<T>
where
    T: Float + RTreeNum,
{
    fn concave_hull(&self, concavity: T) -> Polygon<T> {
        Polygon::new(
            LineString::from(concave_hull(&self.0.clone(), concavity)),
            vec![],
        )
    }
}

fn find_point_closest_to_line<T>(
    interior_points_tree: &RTree<Point<T>>,
    line: Line<T>,
    max_dist: T,
) -> Option<Point<T>>
where
    T: Float + RTreeNum,
{
    let h = max_dist + max_dist;
    let w = line.euclidean_length() + h;
    let two = T::add(T::one(), T::one());
    let search_dist = T::div(T::sqrt(T::powi(w, 2) + T::powi(h, 2)), two);
    let mut candidates = interior_points_tree
        .locate_within_distance(line.centroid(), search_dist)
        .peekable();
    let peek = candidates.peek();
    match peek {
        None => None,
        Some(&point) => Some(candidates.fold(*point, |acc, candidate| {
            if line.euclidean_distance(&acc) > line.euclidean_distance(candidate) {
                *candidate
            } else {
                acc
            }
        })),
    }
}

// This takes significant inspiration from:
// https://github.com/mapbox/concaveman/blob/54838e1/index.js#L11
fn concave_hull<T>(points: &Vec<Point<T>>, concavity: T) -> Vec<Point<T>>
where
    T: Float + RTreeNum,
{
    let multipoint: MultiPoint<T> = points.clone().into();
    let hull = multipoint.convex_hull();

    //Get points in overall dataset that aren't on the exterior linestring of the hull
    let hull_exterior = hull.exterior();
    let hull_tree: RTree<Point<T>> = RTree::bulk_load(hull_exterior.clone().into_points());

    let interior_points: Vec<Point<T>> = points
        .iter()
        .filter_map(|point| {
            if !hull_tree.contains(point) {
                Some(*point)
            } else {
                None
            }
        })
        .collect();
    let mut interior_points_tree: RTree<Point<T>> = RTree::bulk_load(interior_points.clone());

    let mut concave_list: Vec<Point<T>> = vec![];
    let mut added_interior_points: Vec<Point<T>> = vec![];
    let lines = hull_exterior.lines();
    let mut line_queue: VecDeque<Line<T>> = VecDeque::new();
    for line in lines {
        line_queue.push_back(line);
    }
    while let Some(line) = line_queue.pop_front() {
        let edge_length = line.euclidean_length();
        let dist = edge_length / concavity;
        let possible_closest_point = find_point_closest_to_line(&interior_points_tree, line, dist);

        if concave_list.is_empty() || !concave_list.ends_with(&[line.start_point()]) {
            concave_list.push(line.start_point());
        }
        if let Some(closest_point) = possible_closest_point {
            add_closest_point_to_line_queue_if_far_enough(
                &concavity,
                &mut interior_points_tree,
                &mut added_interior_points,
                &mut line_queue,
                &line,
                &edge_length,
                &closest_point,
            )
        } else {
            concave_list.push(line.end_point());
        }
    }

    return concave_list;
}

fn add_closest_point_to_line_queue_if_far_enough<T>(
    concavity: &T,
    interior_points_tree: &mut RTree<Point<T>>,
    added_interior_points: &mut Vec<Point<T>>,
    line_queue: &mut VecDeque<Line<T>>,
    line: &Line<T>,
    edge_length: &T,
    closest_point: &Point<T>,
) where
    T: Float + RTreeNum,
{
    let decision_distance = partial_min(
        closest_point.euclidean_distance(&line.start_point()),
        closest_point.euclidean_distance(&line.end_point()),
    );
    if (*edge_length) / decision_distance > (*concavity)
        && !added_interior_points.contains(closest_point)
    {
        interior_points_tree.remove(closest_point);
        added_interior_points.push(*closest_point);
        let new_line = Line::new(*closest_point, line.end_point());
        line_queue.push_front(new_line);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Point;
    use crate::{line_string, polygon};
    use geo_types::Coordinate;

    #[test]
    fn square_test() {
        let square = vec![
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
            Point::new(4.0, 4.0),
            Point::new(0.0, 4.0),
        ];

        let correct = vec![
            Point::new(4.0, 0.0),
            Point::new(4.0, 4.0),
            Point::new(0.0, 4.0),
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
        ];

        let concavity = 2.0;
        let res = concave_hull(&square.clone(), concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn one_flex_test() {
        let v = vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 1.0),
            Point::new(4.0, 0.0),
            Point::new(4.0, 4.0),
            Point::new(0.0, 4.0),
        ];
        let correct = vec![
            Point::new(4.0, 0.0),
            Point::new(2.0, 1.0),
            Point::new(4.0, 4.0),
            Point::new(0.0, 4.0),
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
        ];
        let concavity = 1.0;
        let res = concave_hull(&v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn four_flex_test() {
        let v = vec![
            Point::new(0.0, 0.0),
            Point::new(2.0, 1.0),
            Point::new(4.0, 0.0),
            Point::new(3.0, 2.0),
            Point::new(4.0, 4.0),
            Point::new(2.0, 3.0),
            Point::new(0.0, 4.0),
            Point::new(1.0, 2.0),
        ];
        let correct = vec![
            Point::new(4.0, 0.0),
            Point::new(3.0, 2.0),
            Point::new(4.0, 4.0),
            Point::new(2.0, 3.0),
            Point::new(0.0, 4.0),
            Point::new(1.0, 2.0),
            Point::new(0.0, 0.0),
            Point::new(2.0, 1.0),
            Point::new(4.0, 0.0),
        ];
        let concavity = 1.7;
        let res = concave_hull(&v, concavity);
        assert_eq!(res, correct);
    }

    #[test]
    fn consecutive_flex_test() {
        let v = vec![
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
            Point::new(4.0, 4.0),
            Point::new(3.0, 1.0),
            Point::new(3.0, 2.0),
        ];
        let correct = vec![
            Point::new(4.0, 0.0),
            Point::new(4.0, 4.0),
            Point::new(3.0, 2.0),
            Point::new(3.0, 1.0),
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
        ];
        let concavity = 2.0;
        let res = concave_hull(&v, concavity);
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
        let norway_concave_hull: Vec<Point<f64>> = loaded_norway_concave_hull
            .iter()
            .map(|tuple| Point::new(f64::from(tuple[0]), f64::from(tuple[1])))
            .collect();
        let res = norway.concave_hull(2.0);
        let coords: &Vec<Coordinate<f64>> = &res.exterior().0;
        let output_points: Vec<Point<f64>> = coords
            .iter()
            .map(|coord| Point::new(coord.x, coord.y))
            .collect();
        assert_eq!(output_points, norway_concave_hull)
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
