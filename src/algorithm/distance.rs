use std::cmp::Ordering;
use std::collections::BinaryHeap;
use num::{Float, ToPrimitive};
use types::{Point, LineString, Polygon};
use algorithm::contains::Contains;
use num::pow::pow;

/// Returns the distance between two geometries.

pub trait Distance<T, Rhs = Self> {
    /// Returns the distance between two geometries:
    ///
    /// ```
    /// use geo::{COORD_PRECISION, Point};
    /// use geo::algorithm::distance::Distance;
    ///
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.distance(&Point::new(-72.1260, 42.45));
    /// assert!(dist < COORD_PRECISION)
    /// ```
    fn distance(&self, rhs: &Rhs) -> T;
}

impl<T> Distance<T, Point<T>> for Point<T>
    where T: Float
{
    fn distance(&self, p: &Point<T>) -> T {
        let (dx, dy) = (self.x() - p.x(), self.y() - p.y());
        dx.hypot(dy)
    }
}

// Return minimum distance between a Point and a Line segment
// This is a helper for Point-to-LineString and Point-to-Polygon distance
// adapted from http://stackoverflow.com/a/1501725/416626
fn line_segment_distance<T>(point: &Point<T>, start: &Point<T>, end: &Point<T>) -> T
    where T: Float + ToPrimitive
{
    let dist_squared = pow(start.distance(end), 2);
    // Implies that start == end
    if dist_squared == T::zero() {
        return pow(point.distance(start), 2);
    }
    // Consider the line extending the segment, parameterized as start + t (end - start)
    // We find the projection of the point onto the line
    // This falls where t = [(point - start) . (end - start)] / |end - start|^2, where . is the dot product
    // We clamp t from [0.0, 1.0] to handle points outside the segment start, end
    let t = T::zero().max(T::one().min((*point - *start).dot(&(*end - *start)) / dist_squared));
    let projected = Point::new(start.x() + t * (end.x() - start.x()),
                               start.y() + t * (end.y() - start.y()));
    point.distance(&projected)
}

#[derive(PartialEq, Debug)]
struct Mindist<T>
    where T: Float
{
    distance: T,
}
// These impls give us a min-heap when used with BinaryHeap
impl<T> Ord for Mindist<T>
    where T: Float
{
    fn cmp(&self, other: &Mindist<T>) -> Ordering {
        other.distance.partial_cmp(&self.distance).unwrap()
    }
}
impl<T> PartialOrd for Mindist<T>
    where T: Float
{
    fn partial_cmp(&self, other: &Mindist<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Eq for Mindist<T> where T: Float {}

// Minimum distance from a Point to a Polygon
impl<T> Distance<T, Polygon<T>> for Point<T>
    where T: Float
{
    fn distance(&self, polygon: &Polygon<T>) -> T {
        // No need to continue if the point is fully inside
        if polygon.contains(self) { return T::zero() }
        // minimum priority queue
        let mut dist_queue: BinaryHeap<Mindist<T>> = BinaryHeap::new();
        // get exterior ring
        let exterior = &polygon.0;
        // exterior ring as a LineString
        let ext_ring = &exterior.0;
        for chunk in ext_ring.chunks(2) {
            let dist = match chunk.len() {
                2 => line_segment_distance(self, chunk.first().unwrap(), chunk.last().unwrap()),
                _ => {
                    // final point in an odd-numbered exterior ring
                    line_segment_distance(&self, chunk.first().unwrap(), chunk.first().unwrap())
                }
            };
            dist_queue.push(Mindist { distance: dist });
        }
        dist_queue.pop().unwrap().distance
    }
}

// Minimum distance from a Point to a LineString
impl<T> Distance<T, LineString<T>> for Point<T>
    where T: Float
{
    fn distance(&self, linestring: &LineString<T>) -> T {
        // No need to continue if the point is on the LineString
        if linestring.contains(self) { return T::zero() }
        // minimum priority queue
        let mut dist_queue: BinaryHeap<Mindist<T>> = BinaryHeap::new();
        // get points vector
        let points = &linestring.0;
        for chunk in points.chunks(2) {
            let dist = match chunk.len() {
                2 => line_segment_distance(self, chunk.first().unwrap(), chunk.last().unwrap()),
                _ => {
                    // final point in a LineString with an odd number of segments
                    line_segment_distance(&self, chunk.first().unwrap(), chunk.first().unwrap())
                }
            };
            dist_queue.push(Mindist { distance: dist });
        }
        dist_queue.pop().unwrap().distance
    }
}

#[cfg(test)]
mod test {
    use types::{Point, LineString, Polygon};
    use algorithm::distance::{Distance, line_segment_distance};
    #[test]
    fn line_segment_distance_test() {
        let o1 = Point::new(8.0, 0.0);
        let o2 = Point::new(5.5, 0.0);
        let o3 = Point::new(5.0, 0.0);
        let o4 = Point::new(4.5, 1.5);

        let p1 = Point::new(7.2, 2.0);
        let p2 = Point::new(6.0, 1.0);

        let dist = line_segment_distance(&o1, &p1, &p2);
        let dist2 = line_segment_distance(&o2, &p1, &p2);
        let dist3 = line_segment_distance(&o3, &p1, &p2);
        let dist4 = line_segment_distance(&o4, &p1, &p2);
        // Results agree with Shapely
        assert_eq!(dist, 2.048590078926335);
        assert_eq!(dist2, 1.118033988749895);
        assert_eq!(dist3, 1.4142135623730951);
        assert_eq!(dist4, 1.5811388300841898);
        // Point is on the line
        let zero_dist = line_segment_distance(&p1, &p1, &p2);
        assert_eq!(zero_dist, 0.0);
    }
    #[test]
    // Point to Polygon, outside point
    fn point_polygon_distance_outside_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.)
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon(ls, vec![]);
        // A Random point outside the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance(&poly);
        assert_eq!(dist, 2.1213203435596424);
    }
    #[test]
    // Point to Polygon, inside point
    fn point_polygon_distance_inside_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.)
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon(ls, vec![]);
        // A Random point inside the octagon
        let p = Point::new(5.5, 2.1);
        let dist = p.distance(&poly);
        assert_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon, on boundary
    fn point_polygon_distance_boundary_test() {
        // an octagon
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
            (5., 1.)
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon(ls, vec![]);
        // A Random point inside the octagon
        let p = Point::new(5.0, 1.0);
        let dist = p.distance(&poly);
        assert_eq!(dist, 0.0);
    }
    #[test]
    // Point to LineString
    fn point_linestring_distance_test() {
        // like an octagon, but missing the lowest horizontal segment
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        // A Random point "inside" the LineString
        let p = Point::new(5.5, 2.1);
        let dist = p.distance(&ls);
        assert_eq!(dist, 1.1313708498984758);
    }
    #[test]
    // Point to LineString, point lies on the LineString
    fn point_linestring_contains_test() {
        // like an octagon, but missing the lowest horizontal segment
        let points = vec![
            (5., 1.),
            (4., 2.),
            (4., 3.),
            (5., 4.),
            (6., 4.),
            (7., 3.),
            (7., 2.),
            (6., 1.),
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        // A point which lies on the LineString
        let p = Point::new(5.0, 4.0);
        let dist = p.distance(&ls);
        assert_eq!(dist, 0.0);
    }
    #[test]
    fn distance1_test() {
        assert_eq!(Point::<f64>::new(0., 0.).distance(&Point::<f64>::new(1., 0.)), 1.);
    }
    #[test]
    fn distance2_test() {
        // Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612)) = 146.99163308930207
        let dist = Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612));
        assert!(dist < 147. && dist > 146.);
    }
}
