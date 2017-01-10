use std::cmp::Ordering;
use std::collections::BinaryHeap;
use num::Float;
use types::{Point, LineString};

// A helper struct for `visvalingam`, defined out here because
// #[deriving] doesn't work inside functions.
#[derive(PartialEq, Debug)]
struct VScore<T>
    where T: Float
{
    area: T,
    current: usize,
    left: usize,
    right: usize,
}

// These impls give us a min-heap
impl<T> Ord for VScore<T>
    where T: Float
{
    fn cmp(&self, other: &VScore<T>) -> Ordering {
        other.area.partial_cmp(&self.area).unwrap()
    }
}

impl<T> PartialOrd for VScore<T>
    where T: Float
{
    fn partial_cmp(&self, other: &VScore<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for VScore<T> where T: Float {}

/// Simplify a line using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm
///
/// epsilon is the minimum triangle area
pub fn visvalingam<T>(orig: &[Point<T>], epsilon: &T) -> Vec<Point<T>>
    where T: Float
{
    // No need to continue without at least three points
    if orig.len() < 3 || orig.is_empty() {
        return orig.to_vec();
    }

    let max = orig.len();
    // used to store the area of the previously eliminated point for comparison
    let mut previous_area;

    // Adjacent retained points. Simulating the points in a
    // linked list with indices into `orig`. Big number (larger than or equal to
    // `max`) means no next element, and (0, 0) means deleted element.
    let mut adjacent: Vec<(_)> = (0..orig.len())
        .map(|i| {
            if i == 0 {
                (-1_i32, 1_i32)
            } else {
                ((i - 1) as i32, (i + 1) as i32)
            }
        })
        .collect();

    // Store all the triangles in a minimum priority queue, based on their area.
    // Invalid triangles are *not* removed if / when points
    // are removed; they're handled by skipping them as
    // necessary in the main loop. (This is handled by recording the
    // state in the VScore)
    let mut pq = BinaryHeap::new();
    // Compute the initial triangles, i.e. take all consecutive groups
    // of 3 points and form triangles from them
    for (i, win) in orig.windows(3).enumerate() {
        pq.push(VScore {
            area: area(win[0], win[1], win[2]),
            current: i + 1,
            left: i,
            right: i + 2,
        });
    }
    // While there are still points for which the associated triangle
    // has an area below the epsilon
    loop {
        let smallest = match pq.pop() {
            // We've exhausted all the possible triangles, so leave the main loop
            None => break,
            // This triangle's area is above epsilon, so skip it
            Some(ref x) if x.area > *epsilon => continue,
            //  This triangle's area is below epsilon, so use it to recalculate
            Some(s) => s,
        };
        let (left, right) = adjacent[smallest.current];
        // A point in this triangle has been removed since this VScore
        // was created, so just skip it
        if left as i32 != smallest.left as i32 || right as i32 != smallest.right as i32 {
            continue;
        }
        // We've got a valid triangle, and its area is smaller than epsilon, so
        // remove it from the simulated "linked list"
        let (ll, _) = adjacent[left as usize];
        let (_, rr) = adjacent[right as usize];
        adjacent[left as usize] = (ll, right);
        adjacent[right as usize] = (left, rr);
        adjacent[smallest.current as usize] = (0, 0);
        // store its area for comparison with the next triangle to be eliminated
        previous_area = smallest.area;

        // Now recompute the triangles, using left and right adjacent points
        let choices = [(ll, left, right), (left, right, rr)];
        for &(ai, current_point, bi) in &choices {
            if ai as usize >= max || bi as usize >= max {
                // Out of bounds, i.e. we're on one edge
                continue;
            }
            let new_left = Point::new(orig[ai as usize].x(), orig[ai as usize].y());
            let new_current = Point::new(orig[current_point as usize].x(),
                                         orig[current_point as usize].y());
            let new_right = Point::new(orig[bi as usize].x(), orig[bi as usize].y());


            // Store re-calculated triangle in priority queue
            // If its calculated area is less than that of the last point to be
            // eliminated, use the latter's area instead.
            // (This ensures that the current point cannot be eliminated
            // without eliminating previously eliminated points)
            // (Visvalingam and Whyatt 2013, p47)
            pq.push(VScore {
                area: previous_area.max(area(new_left, new_current, new_right)),
                current: current_point as usize,
                left: ai as usize,
                right: bi as usize,
            });
        }
    }
    // Filter out the points that have been deleted, returning remaining points
    let simplified: Vec<Point<T>> = orig.iter()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| { if *adj != (0, 0) { Some(*tup) } else { None } })
        .collect();
    simplified
}

// Area of a triangle given three vertices
fn area<T>(p1: Point<T>, p2: Point<T>, p3: Point<T>) -> T
    where T: Float
{
    ((p1.x() - p3.x()) * (p2.y() - p3.y()) - (p2.x() - p3.x()) * (p1.y() - p3.y())).abs() /
    (T::one() + T::one())
}

pub trait SimplifyVW<T, Epsilon = T> {
    /// Returns the simplified representation of a LineString, using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm  
    /// 
    /// See [here](https://bost.ocks.org/mike/simplify/) for a graphical explanation 
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::simplifyvw::{SimplifyVW};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(5.0, 2.0));
    /// vec.push(Point::new(3.0, 8.0));
    /// vec.push(Point::new(6.0, 20.0));
    /// vec.push(Point::new(7.0, 25.0));
    /// vec.push(Point::new(10.0, 10.0));
    /// let linestring = LineString(vec);
    /// let mut compare = Vec::new();
    /// compare.push(Point::new(5.0, 2.0));
    /// compare.push(Point::new(7.0, 25.0));
    /// compare.push(Point::new(10.0, 10.0));
    /// let ls_compare = LineString(compare);
    /// let simplified = linestring.simplifyvw(&30.0);
    /// assert_eq!(simplified, ls_compare)
    /// ```
    fn simplifyvw(&self, epsilon: &T) -> Self where T: Float;
}

impl<T> SimplifyVW<T> for LineString<T>
    where T: Float
{
    fn simplifyvw(&self, epsilon: &T) -> LineString<T> {
        LineString(visvalingam(&self.0, epsilon))
    }
}

#[cfg(test)]
mod test {
    use types::Point;
    use super::visvalingam;

    #[test]
    fn visvalingam_test() {
        // this is the PostGIS example
        let points = vec![(5.0, 2.0), (3.0, 8.0), (6.0, 20.0), (7.0, 25.0), (10.0, 10.0)];
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e.0, e.1)).collect();

        let correct = vec![(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();

        let simplified = visvalingam(&points_ls, &30.);
        assert_eq!(simplified, correct_ls);
    }
    #[test]
    fn visvalingam_test_empty_linestring() {
        let vec = Vec::new();
        let compare = Vec::new();
        let simplified = visvalingam(&vec, &1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn visvalingam_test_two_point_linestring() {
        let mut vec = Vec::new();
        vec.push(Point::new(0.0, 0.0));
        vec.push(Point::new(27.8, 0.1));
        let mut compare = Vec::new();
        compare.push(Point::new(0.0, 0.0));
        compare.push(Point::new(27.8, 0.1));
        let simplified = visvalingam(&vec, &1.0);
        assert_eq!(simplified, compare);
    }
}
