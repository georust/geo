use num_traits::Float;
use types::{Point, Polygon, MultiPolygon, LineString, MultiPoint, MultiLineString};
use std::mem;

fn swap_remove_to_first<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    let tmp = mem::replace(slice, &mut []);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}
fn swap_remove_to_last<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    let tmp = mem::replace(slice, &mut []);
    let len = tmp.len();
    tmp.swap(len - 1, idx);
    let (h, t) = tmp.split_last_mut().unwrap();
    *slice = t;
    h
}
// slice[..result] have pred(e) == true, slice[result..] have pred(e) == false
fn partition<T, F: FnMut(&T) -> bool>(mut slice: &mut [T], mut pred: F) -> usize {
    let mut i = 0;
    loop {
        let test = match slice.first() {
            Some(e) => pred(e),
            None => break,
        };
        if test {
            swap_remove_to_first(&mut slice, 0);
            i += 1;
        } else {
            swap_remove_to_last(&mut slice, 0);
        }
    }
    i
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
fn cross_prod<T>(p_a: &Point<T>, p_b: &Point<T>, p_c: &Point<T>) -> T
    where T: Float
{
    (p_b.x() - p_a.x()) * (p_c.y() - p_a.y()) - (p_b.y() - p_a.y()) * (p_c.x() - p_a.x())
}
fn point_location<T>(p_a: &Point<T>, p_b: &Point<T>, p_c: &Point<T>) -> bool
    where T: Float
{
    cross_prod(p_a, p_b, p_c) > T::zero()
}

// Fast distance between line segment (p_a, p_b), and point p_c
fn pseudo_distance<T>(p_a: &Point<T>, p_b: &Point<T>, p_c: &Point<T>) -> T
    where T: Float
{
    let abx = p_b.x() - p_a.x();
    let aby = p_b.y() - p_a.y();
    let dist = abx * (p_a.y() - p_c.y()) - aby * (p_a.x() - p_c.x());
    if dist < T::zero() { -dist } else { dist }
}

// Adapted from http://www.ahristov.com/tutorial/geometry-games/convex-hull.html
fn quick_hull<T>(mut points: &mut [Point<T>]) -> Vec<Point<T>>
    where T: Float
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
    let last = partition(&mut points, |p| point_location(max, min, p));
    hull_set(max, min, &mut points[..last], &mut hull);
    hull.push(*max);
    let last = partition(&mut points, |p| point_location(min, max, p));
    hull_set(min, max, &mut points[..last], &mut hull);
    hull.push(*min);
    // close the polygon
    let final_element = *hull.first().unwrap();
    hull.push(final_element);
    hull
}

// recursively calculate the convex hull of a subset of points
fn hull_set<T>(p_a: &Point<T>, p_b: &Point<T>, mut set: &mut [Point<T>], hull: &mut Vec<Point<T>>)
    where T: Float
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
        let current_distance = pseudo_distance(p_a, p_b, point);
        if current_distance > furthest_distance {
            furthest_distance = current_distance;
            furthest_idx = idx
        }
    }
    // move Point at furthest_point from set into hull
    let furthest_point = swap_remove_to_first(&mut set, furthest_idx);
    // points over PB
    let last = partition(set, |p| point_location(&furthest_point, p_b, p));
    hull_set(&furthest_point, p_b, &mut set[..last], hull);
    hull.push(*furthest_point);
    // points over AP
    let last = partition(set, |p| point_location(p_a, &furthest_point, p));
    hull_set(p_a, &furthest_point, &mut set[..last], hull);
}

pub trait ConvexHull<T> {
    /// Returns the convex hull of a Polygon. The hull is always oriented counter-clockwise.
    ///
    /// This implementation uses the QuickHull algorithm,
    /// based on [Barber, C. Bradford; Dobkin, David P.; Huhdanpaa, Hannu (1 December 1996)](https://dx.doi.org/10.1145%2F235815.235821)
    /// Original paper here: http://www.cs.princeton.edu/~dpd/Papers/BarberDobkinHuhdanpaa.pdf
    ///
    /// ```
    /// use geo::{Point, LineString, Polygon};
    /// use geo::convexhull::ConvexHull;
    /// // an L shape
    /// let coords = vec![(0.0, 0.0), (4.0, 0.0), (4.0, 1.0), (1.0, 1.0), (1.0, 4.0), (0.0, 4.0), (0.0, 0.0)];
    /// let ls = LineString(coords.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// let poly = Polygon::new(ls, vec![]);
    ///
    /// // The correct convex hull coordinates
    /// let hull_coords = vec![(4.0, 0.0), (4.0, 1.0), (1.0, 4.0), (0.0, 4.0), (0.0, 0.0), (4.0, 0.0)];
    /// let correct_hull = LineString(hull_coords.iter().map(|e| Point::new(e.0, e.1)).collect());
    ///
    /// let res = poly.convex_hull();
    /// assert_eq!(res.exterior, correct_hull);
    /// ```
    fn convex_hull(&self) -> Polygon<T> where T: Float;
}

impl<T> ConvexHull<T> for Polygon<T>
    where T: Float
{
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(LineString(quick_hull(&mut self.exterior.0.clone())), vec![])
    }
}

impl<T> ConvexHull<T> for MultiPolygon<T>
    where T: Float
{
    fn convex_hull(&self) -> Polygon<T> {
        let mut aggregated: Vec<Point<T>> = self.0.iter()
            .flat_map(|elem| elem.exterior.0.iter().cloned())
            .collect();
        Polygon::new(LineString(quick_hull(&mut aggregated)), vec![])
    }
}

impl<T> ConvexHull<T> for LineString<T>
    where T: Float
{
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(LineString(quick_hull(&mut self.0.clone())), vec![])
    }
}

impl<T> ConvexHull<T> for MultiLineString<T>
    where T: Float
{
    fn convex_hull(&self) -> Polygon<T> {
        let mut aggregated: Vec<Point<T>> = self.0.iter()
            .flat_map(|elem| elem.0.iter().cloned())
            .collect();
        Polygon::new(LineString(quick_hull(&mut aggregated)), vec![])
    }
}

impl<T> ConvexHull<T> for MultiPoint<T>
    where T: Float
{
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(LineString(quick_hull(&mut self.0.clone())), vec![])
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use super::*;

    #[test]
    fn quick_hull_test1() {
        let mut v = vec![Point::new(0.0, 0.0),
                         Point::new(4.0, 0.0),
                         Point::new(4.0, 1.0),
                         Point::new(1.0, 1.0),
                         Point::new(1.0, 4.0),
                         Point::new(0.0, 4.0),
                         Point::new(0.0, 0.0)];
        let correct = vec![Point::new(4.0, 0.0),
                           Point::new(4.0, 1.0),
                           Point::new(1.0, 4.0),
                           Point::new(0.0, 4.0),
                           Point::new(0.0, 0.0),
                           Point::new(4.0, 0.0)];
        let res = quick_hull(&mut v);
        assert_eq!(res, correct);
    }
    #[test]
    fn quick_hull_test2() {
        let mut v = vec![Point::new(0.0, 10.0),
                         Point::new(1.0, 1.0),
                         Point::new(10.0, 0.0),
                         Point::new(1.0, -1.0),
                         Point::new(0.0, -10.0),
                         Point::new(-1.0, -1.0),
                         Point::new(-10.0, 0.0),
                         Point::new(-1.0, 1.0),
                         Point::new(0.0, 10.0)];
        let correct = vec![Point::new(0.0, -10.0),
                           Point::new(10.0, 0.0),
                           Point::new(0.0, 10.0),
                           Point::new(-10.0, 0.0),
                           Point::new(0.0, -10.0)];
        let res = quick_hull(&mut v);
        assert_eq!(res, correct);
    }
    #[test]
    // test whether output is ccw
    fn quick_hull_test_ccw() {
        let initial = vec![(1.0, 0.0), (2.0, 1.0), (1.75, 1.1), (1.0, 2.0), (0.0, 1.0), (1.0, 0.0)];
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
        let initial = vec![(0., 0.), (2., 0.), (2.5, 1.75), (2.3, 1.7), (1.75, 2.5), (1.3, 2.), (0., 2.), (0., 0.)];
        let mut v: Vec<_> = initial.iter().map(|e| Point::new(e.0, e.1)).collect();
        let correct = vec![(2.0, 0.0), (2.5, 1.75), (1.75, 2.5), (0.0, 2.0), (0.0, 0.0), (2.0, 0.0)];
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
        let v = vec![Point::new(0.0, 10.0),
                         Point::new(1.0, 1.0),
                         Point::new(10.0, 0.0),
                         Point::new(1.0, -1.0),
                         Point::new(0.0, -10.0),
                         Point::new(-1.0, -1.0),
                         Point::new(-10.0, 0.0),
                         Point::new(-1.0, 1.0),
                         Point::new(0.0, 10.0)];
        let mp = MultiPoint(v);
        let correct = vec![Point::new(0.0, -10.0),
                           Point::new(10.0, 0.0),
                           Point::new(0.0, 10.0),
                           Point::new(-10.0, 0.0),
                           Point::new(0.0, -10.0)];
        let res = mp.convex_hull();
        assert_eq!(res.exterior.0, correct);
    }
    #[test]
    fn quick_hull_linestring_test() {
        let v = vec![
            Point::new(0.0, 10.0),
            Point::new(1.0, 1.0),
            Point::new(10.0, 0.0),
            Point::new(1.0, -1.0),
            Point::new(0.0, -10.0),
            Point::new(-1.0, -1.0),
            Point::new(-10.0, 0.0),
            Point::new(-1.0, 1.0),
            Point::new(0.0, 10.0)];
        let mp = LineString(v);
        let correct = vec![
            Point::new(0.0, -10.0),
            Point::new(10.0, 0.0),
            Point::new(0.0, 10.0),
            Point::new(-10.0, 0.0),
            Point::new(0.0, -10.0)];
        let res = mp.convex_hull();
        assert_eq!(res.exterior.0, correct);
    }
    #[test]
    fn quick_hull_multilinestring_test() {
        let v1 = LineString(vec![Point::new(0.0, 0.0), Point::new(1.0, 10.0)]);
        let v2 = LineString(vec![Point::new(1.0, 10.0), Point::new(2.0, 0.0), Point::new(3.0, 1.0)]);
        let mls = MultiLineString(vec![v1, v2]);
        let correct = vec![
            Point::new(2.0, 0.0),
            Point::new(3.0, 1.0),
            Point::new(1.0, 10.0),
            Point::new(0.0, 0.0),
            Point::new(2.0, 0.0)];
        let res = mls.convex_hull();
        assert_eq!(res.exterior.0, correct);
    }
    #[test]
    fn quick_hull_multipolygon_test() {
        let ls1 = LineString(vec![Point::new(0.0, 0.0), Point::new(1.0, 10.0), Point::new(2.0, 0.0), Point::new(0.0, 0.0)]);
        let ls2 = LineString(vec![Point::new(3.0, 0.0), Point::new(4.0, 10.0), Point::new(5.0, 0.0), Point::new(3.0, 0.0)]);
        let p1 = Polygon::new(ls1, vec![]);
        let p2 = Polygon::new(ls2, vec![]);
        let mp = MultiPolygon(vec![p1, p2]);
        let correct = vec![
            Point::new(5.0, 0.0),
            Point::new(4.0, 10.0),
            Point::new(1.0, 10.0),
            Point::new(0.0, 0.0),
            Point::new(5.0, 0.0)
        ];
        let res = mp.convex_hull();
        assert_eq!(res.exterior.0, correct);
    }
}
