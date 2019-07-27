use crate::algorithm::euclidean_distance::EuclideanDistance;
use crate::utils::partition_slice;
use crate::{Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
use num_traits::Float;
use std::mem;

fn swap_remove_to_first<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    let tmp = mem::replace(slice, &mut []);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}

// Determine whether a point lies on one side of a line segment, or the other.
// The cross product v x w of two vectors v and w is a vector whose length is
// |v||w|sin φ, (where |v| is the length of v and φ is the angle between the vectors),
// and which is orthogonal (perpendicular) to both v and w.  Since there are two
// such possible vectors, the definition arbitrarily selects the one that matches
// the direction in which a screw would move if rotated from v to w

// Mathematically, if the coordinates of vectors v and w are (vx, vy) and (wx, wy)
// respectively, the cross product will be (vxwy - vywx). If a segment is
// defined by points A B and, we wish to check on which side of AB a third point C falls,
// we can compute the cross product AB x AC and check its sign:
// If it's negative, it will be on the "right" side of AB
// (when standing on A and looking towards B). If positive, it will be on the left side
fn point_location<T>(p_a: Point<T>, p_b: Point<T>, p_c: Point<T>) -> bool
where
    T: Float,
{
    p_a.cross_prod(p_b, p_c) > T::zero()
}

// Adapted from https://web.archive.org/web/20180409175413/http://www.ahristov.com/tutorial/geometry-games/convex-hull.html
fn quick_hull<T>(mut points: &mut [Point<T>]) -> Vec<Point<T>>
where
    T: Float,
{
    // can't build a hull from fewer than four points
    if points.len() < 4 {
        return points.to_vec();
    }
    let mut hull = vec![];
    let min = swap_remove_to_first(&mut points, 0);
    let max = swap_remove_to_first(&mut points, 0);
    if min.x() > max.x() {
        mem::swap(min, max);
    }
    for point in points.iter_mut() {
        if point.x() < min.x() {
            mem::swap(point, min);
        }
        if point.x() > max.x() {
            mem::swap(point, max);
        }
    }
    {
        let (mut points, _) = partition_slice(&mut points, |p| point_location(*max, *min, *p));
        hull_set(*max, *min, &mut points, &mut hull);
    }
    hull.push(*max);
    let (mut points, _) = partition_slice(&mut points, |p| point_location(*min, *max, *p));
    hull_set(*min, *max, &mut points, &mut hull);
    hull.push(*min);
    // close the polygon
    let final_element = *hull.first().unwrap();
    hull.push(final_element);
    hull
}

// recursively calculate the convex hull of a subset of points
fn hull_set<T>(p_a: Point<T>, p_b: Point<T>, mut set: &mut [Point<T>], hull: &mut Vec<Point<T>>)
where
    T: Float,
{
    if set.is_empty() {
        return;
    }
    if set.len() == 1 {
        hull.push(set[0]);
        return;
    }
    let mut furthest_distance = Float::min_value();
    let mut furthest_idx = 0;
    for (idx, point) in set.iter().enumerate() {
        // let current_distance = pseudo_distance(p_a, p_b, point);
        let current_distance = point.euclidean_distance(&Line::new(p_a.0, p_b.0));
        if current_distance > furthest_distance {
            furthest_distance = current_distance;
            furthest_idx = idx
        }
    }
    // move Point at furthest_point from set into hull
    let furthest_point = swap_remove_to_first(&mut set, furthest_idx);
    // points over PB
    {
        let (mut points, _) = partition_slice(set, |p| point_location(*furthest_point, p_b, *p));
        hull_set(*furthest_point, p_b, &mut points, hull);
    }
    hull.push(*furthest_point);
    // points over AP
    let (mut points, _) = partition_slice(set, |p| point_location(p_a, *furthest_point, *p));
    hull_set(p_a, *furthest_point, &mut points, hull);
}

pub trait ConvexHull<T> {
    /// Returns the convex hull of a Polygon. The hull is always oriented counter-clockwise.
    ///
    /// This implementation uses the QuickHull algorithm,
    /// based on [Barber, C. Bradford; Dobkin, David P.; Huhdanpaa, Hannu (1 December 1996)](https://dx.doi.org/10.1145%2F235815.235821)
    /// Original paper here: http://www.cs.princeton.edu/~dpd/Papers/BarberDobkinHuhdanpaa.pdf
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{line_string, polygon};
    /// use geo::convexhull::ConvexHull;
    ///
    /// // an L shape
    /// let poly = polygon![
    ///     (x: 0.0, y: 0.0),
    ///     (x: 4.0, y: 0.0),
    ///     (x: 4.0, y: 1.0),
    ///     (x: 1.0, y: 1.0),
    ///     (x: 1.0, y: 4.0),
    ///     (x: 0.0, y: 4.0),
    ///     (x: 0.0, y: 0.0),
    /// ];
    ///
    /// // The correct convex hull coordinates
    /// let correct_hull = line_string![
    ///     (x: 4.0, y: 0.0),
    ///     (x: 4.0, y: 1.0),
    ///     (x: 1.0, y: 4.0),
    ///     (x: 0.0, y: 4.0),
    ///     (x: 0.0, y: 0.0),
    ///     (x: 4.0, y: 0.0),
    /// ];
    ///
    /// let res = poly.convex_hull();
    /// assert_eq!(res.exterior(), &correct_hull);
    /// ```
    fn convex_hull(&self) -> Polygon<T>
    where
        T: Float;
}

impl<T> ConvexHull<T> for Polygon<T>
where
    T: Float,
{
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(
            LineString::from(quick_hull(&mut self.exterior().clone().into_points())),
            vec![],
        )
    }
}

impl<T> ConvexHull<T> for MultiPolygon<T>
where
    T: Float,
{
    fn convex_hull(&self) -> Polygon<T> {
        let mut aggregated: Vec<Point<T>> = self
            .0
            .iter()
            .flat_map(|elem| elem.exterior().0.iter().map(|c| Point(*c)))
            .collect();
        Polygon::new(LineString::from(quick_hull(&mut aggregated)), vec![])
    }
}

impl<T> ConvexHull<T> for LineString<T>
where
    T: Float,
{
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(
            LineString::from(quick_hull(&mut self.clone().into_points())),
            vec![],
        )
    }
}

impl<T> ConvexHull<T> for MultiLineString<T>
where
    T: Float,
{
    fn convex_hull(&self) -> Polygon<T> {
        let mut aggregated: Vec<Point<T>> = self
            .0
            .iter()
            .flat_map(|elem| elem.clone().into_points())
            .collect();
        Polygon::new(LineString::from(quick_hull(&mut aggregated)), vec![])
    }
}

impl<T> ConvexHull<T> for MultiPoint<T>
where
    T: Float,
{
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(LineString::from(quick_hull(&mut self.0.clone())), vec![])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, polygon, Coordinate, Point};

    #[test]
    fn quick_hull_test1() {
        let mut v = vec![
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
            Point::new(4.0, 1.0),
            Point::new(1.0, 1.0),
            Point::new(1.0, 4.0),
            Point::new(0.0, 4.0),
            Point::new(0.0, 0.0),
        ];
        let correct = vec![
            Point::new(4.0, 0.0),
            Point::new(4.0, 1.0),
            Point::new(1.0, 4.0),
            Point::new(0.0, 4.0),
            Point::new(0.0, 0.0),
            Point::new(4.0, 0.0),
        ];
        let res = quick_hull(&mut v);
        assert_eq!(res, correct);
    }
    #[test]
    fn quick_hull_test2() {
        let mut v = vec![
            Point::new(0.0, 10.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 0.0),
            Point::new(1.0, -1.0),
            Point::new(0.0, -10.0),
            Point::new(-1.0, -1.0),
            Point::new(-10.0, 0.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 10.0),
        ];
        let correct = vec![
            Point::new(0.0, -10.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(-10.0, 0.0),
            Point::new(0.0, -10.0),
        ];
        let res = quick_hull(&mut v);
        assert_eq!(res, correct);
    }
    #[test]
    // test whether output is ccw
    fn quick_hull_test_ccw() {
        let initial = vec![
            (1.0, 0.0),
            (2.0, 1.0),
            (1.75, 1.1),
            (1.0, 2.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| Point::new(e.0, e.1)).collect();
        let correct = vec![(1.0, 0.0), (2.0, 1.0), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
        let v_correct: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();
        let res = quick_hull(&mut v);
        assert_eq!(res, v_correct);
    }
    #[test]
    // test that output isn't rotated
    fn quick_hull_test_ccw_maintain() {
        // initial input begins at min y, is oriented ccw
        let initial = vec![
            (0., 0.),
            (2., 0.),
            (2.5, 1.75),
            (2.3, 1.7),
            (1.75, 2.5),
            (1.3, 2.),
            (0., 2.),
            (0., 0.),
        ];
        let mut v: Vec<_> = initial.iter().map(|e| Point::new(e.0, e.1)).collect();
        let correct = vec![
            (2.0, 0.0),
            (2.5, 1.75),
            (1.75, 2.5),
            (0.0, 2.0),
            (0.0, 0.0),
            (2.0, 0.0),
        ];
        let v_correct: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();
        let res = quick_hull(&mut v);
        assert_eq!(res, v_correct);
    }
    #[test]
    fn quick_hull_test_complex() {
        let coords = include!("test_fixtures/poly1.rs");
        let mut v: Vec<_> = coords.iter().map(|e| Point::new(e.0, e.1)).collect();
        let correct = include!("test_fixtures/poly1_hull.rs");
        let v_correct: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();
        let res = quick_hull(&mut v);
        assert_eq!(res, v_correct);
    }
    #[test]
    fn quick_hull_test_complex_2() {
        let coords = include!("test_fixtures/poly2.rs");
        let mut v: Vec<_> = coords.iter().map(|e| Point::new(e.0, e.1)).collect();
        let correct = include!("test_fixtures/poly2_hull.rs");
        let v_correct: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();
        let res = quick_hull(&mut v);
        assert_eq!(res, v_correct);
    }
    #[test]
    fn quick_hull_multipoint_test() {
        let v = vec![
            Point::new(0.0, 10.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 0.0),
            Point::new(1.0, -1.0),
            Point::new(0.0, -10.0),
            Point::new(-1.0, -1.0),
            Point::new(-10.0, 0.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 10.0),
        ];
        let mp = MultiPoint(v);
        let correct = vec![
            Coordinate::from((0.0, -10.0)),
            Coordinate::from((10.0, 0.0)),
            Coordinate::from((0.0, 10.0)),
            Coordinate::from((-10.0, 0.0)),
            Coordinate::from((0.0, -10.0)),
        ];
        let res = mp.convex_hull();
        assert_eq!(res.exterior().0, correct);
    }
    #[test]
    fn quick_hull_linestring_test() {
        let mp = line_string![
            (x: 0.0, y: 10.0),
            (x: 1.0, y: 1.0),
            (x: 10.0, y: 0.0),
            (x: 1.0, y: -1.0),
            (x: 0.0, y: -10.0),
            (x: -1.0, y: -1.0),
            (x: -10.0, y: 0.0),
            (x: -1.0, y: 1.0),
            (x: 0.0, y: 10.0),
        ];
        let correct = vec![
            Coordinate::from((0.0, -10.0)),
            Coordinate::from((10.0, 0.0)),
            Coordinate::from((0.0, 10.0)),
            Coordinate::from((-10.0, 0.0)),
            Coordinate::from((0.0, -10.0)),
        ];
        let res = mp.convex_hull();
        assert_eq!(res.exterior().0, correct);
    }
    #[test]
    fn quick_hull_multilinestring_test() {
        let v1 = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0)];
        let v2 = line_string![(x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 3.0, y: 1.0)];
        let mls = MultiLineString(vec![v1, v2]);
        let correct = vec![
            Coordinate::from((2.0, 0.0)),
            Coordinate::from((3.0, 1.0)),
            Coordinate::from((1.0, 10.0)),
            Coordinate::from((0.0, 0.0)),
            Coordinate::from((2.0, 0.0)),
        ];
        let res = mls.convex_hull();
        assert_eq!(res.exterior().0, correct);
    }
    #[test]
    fn quick_hull_multipolygon_test() {
        let p1 = polygon![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 0.0, y: 0.0)];
        let p2 = polygon![(x: 3.0, y: 0.0), (x: 4.0, y: 10.0), (x: 5.0, y: 0.0), (x: 3.0, y: 0.0)];
        let mp = MultiPolygon(vec![p1, p2]);
        let correct = vec![
            Coordinate::from((5.0, 0.0)),
            Coordinate::from((4.0, 10.0)),
            Coordinate::from((1.0, 10.0)),
            Coordinate::from((0.0, 0.0)),
            Coordinate::from((5.0, 0.0)),
        ];
        let res = mp.convex_hull();
        assert_eq!(res.exterior().0, correct);
    }
}
