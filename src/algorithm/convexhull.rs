use num_traits::Float;
use types::{Point, Polygon, LineString};
use std::collections::BTreeSet;

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
fn point_location<T>(p_a: &Point<T>, p_b: &Point<T>, p_c: &Point<T>) -> bool
    where T: Float
{
    (p_b.x() - p_a.x()) * (p_c.y() - p_a.y()) - (p_b.y() - p_a.y()) * (p_c.x() - p_a.x()) >
    T::zero()
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
fn quick_hull<T>(points: &[Point<T>]) -> Vec<Point<T>>
    where T: Float
{
    // can't build a hull from fewer than four points
    if points.len() < 4 {
        return points.to_vec();
    }
    let mut hull = vec![];
    let mut min_x_idx = 0;
    let mut max_x_idx = 0;
    let mut min_x = Float::max_value();
    let mut max_x = Float::min_value();
    let to_retain: BTreeSet<_> = (0..points.len() - 1).collect();
    let mut to_remove: BTreeSet<_> = BTreeSet::new();
    for (idx, point) in points.iter().enumerate() {
        if point.x() < min_x {
            min_x = point.x();
            min_x_idx = idx;
        }
        if point.x() > max_x {
            max_x = point.x();
            max_x_idx = idx;
        }
    }
    to_remove.insert(min_x_idx);
    to_remove.insert(max_x_idx);

    let p_a = points[min_x_idx];
    let p_b = points[max_x_idx];
    // min x and max x points are always part of the hull
    hull.push(p_a);
    hull.push(p_b);
    let mut left_set = vec![];
    let mut right_set = vec![];
    // divide remaining points into left and right
    // this is a bit hairy
    for point in to_retain.difference(&to_remove)
        .map(|&idx| points[idx]) {
        if !point_location(&p_a, &p_b, &point) {
            left_set.push(point);
        } else {
            right_set.push(point);
        }
    }
    hull_set(&p_a, &p_b, &mut right_set, &mut hull);
    hull_set(&p_b, &p_a, &mut left_set, &mut hull);
    // close the polygon
    let final_element = *hull.first().unwrap();
    hull.push(final_element);
    hull
}

// recursively calculate the convex hull of a subset of points
fn hull_set<T>(p_a: &Point<T>, p_b: &Point<T>, set: &mut Vec<Point<T>>, hull: &mut Vec<Point<T>>)
    where T: Float
{
    let insert_position = hull.iter().position(|r| r == p_b).unwrap();
    if set.is_empty() {
        return;
    }
    if set.len() == 1 {
        hull.insert(insert_position, set[0]);
        set.remove(0);
        return;
    }
    let mut furthest_distance = Float::min_value();
    let mut furthest_idx = <usize>::min_value();
    for (idx, point) in set.iter().enumerate() {
        let current_distance = pseudo_distance(p_a, p_b, point);
        if current_distance > furthest_distance {
            furthest_distance = current_distance;
            furthest_idx = idx
        }
    }
    // move Point at furthest_point from set into hull
    let furthest_point = set[furthest_idx];
    hull.insert(insert_position, set[furthest_idx]);
    set.remove(furthest_idx);
    // Determine points to the left of A, furthest_point
    let mut left_ap: Vec<Point<T>> = vec![];
    let mut left_pb: Vec<Point<T>> = vec![];
    for point in set.iter() {
        if point_location(p_a, &furthest_point, point) {
            left_ap.push(*point);
        }
    }
    // Determine points to the left of furthest_point, B
    for point in set.iter() {
        if point_location(&furthest_point, p_b, point) {
            left_pb.push(*point);
        }
    }
    // recur
    hull_set(p_a, &furthest_point, &mut left_ap, hull);
    hull_set(&furthest_point, p_b, &mut left_pb, hull);
}

pub trait ConvexHull<T> {
    /// Returns the convex hull of a Polygon
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
    /// let hull_coords = vec![(0.0, 0.0), (0.0, 4.0), (1.0, 4.0), (4.0, 1.0), (4.0, 0.0), (0.0, 0.0)];
    /// let correct_hull = LineString(hull_coords.iter().map(|e| Point::new(e.0, e.1)).collect());
    ///
    /// let res = poly.convex_hull();
    /// assert_eq!(res.exterior, correct_hull);
    /// ```
    fn convex_hull(&self) -> Self where T: Float;
}

impl<T> ConvexHull<T> for Polygon<T>
    where T: Float
{
    fn convex_hull(&self) -> Polygon<T> {
        Polygon::new(LineString(quick_hull(&self.exterior.0)), vec![])
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use super::*;

    #[test]
    fn quick_hull_test() {
        let v = vec![Point::new(0.0, 0.0),
                     Point::new(4.0, 0.0),
                     Point::new(4.0, 1.0),
                     Point::new(1.0, 1.0),
                     Point::new(1.0, 4.0),
                     Point::new(0.0, 4.0),
                     Point::new(0.0, 0.0)];
        let correct = vec![Point::new(0.0, 0.0),
                           Point::new(0.0, 4.0),
                           Point::new(1.0, 4.0),
                           Point::new(4.0, 1.0),
                           Point::new(4.0, 0.0),
                           Point::new(0.0, 0.0)];
        let res = quick_hull(&v);
        assert_eq!(res, correct);
    }
    #[test]
    fn quick_hull_test_complex() {
        let coords = include!("test_fixtures/poly1.rs");
        let v = coords.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<Point<_>>>();
        let correct = include!("test_fixtures/poly1_hull.rs");
        let v_correct = correct.iter().map(|e| Point::new(e.0, e.1)).collect::<Vec<Point<_>>>();
        let res = quick_hull(&v);
        assert_eq!(res, v_correct);
    }
}
