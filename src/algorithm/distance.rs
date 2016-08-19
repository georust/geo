use std::cmp::Ordering;
use std::collections::BinaryHeap;
use num::{Float, ToPrimitive};
use types::{Point, LineString, Polygon};
use algorithm::contains::Contains;
use num::pow::pow;

/// Returns the distance between two geometries.

pub trait Distance<T, Rhs = Self> {
    /// Returns the distance between two geometries
    ///
    /// If a `Point` is contained by a `Polygon`, the distance is `0.0`  
    /// If a `Point` lies on a `Polygon`'s exterior ring, the distance is `0.0`  
    /// If a `Point` lies on a `LineString`, the distance is `0.0`  
    /// The distance between a `Point` and an empty `LineString` is `0.0`  
    ///
    /// ```
    /// use geo::{COORD_PRECISION, Point, LineString, Polygon};
    /// use geo::algorithm::distance::Distance;
    ///
    /// // Point to Point example
    /// let p = Point::new(-72.1235, 42.3521);
    /// let dist = p.distance(&Point::new(-72.1260, 42.45));
    /// assert!(dist < COORD_PRECISION);
    ///
    /// // Point to Polygon example
    /// let points = vec![
    ///     (5., 1.),
    ///     (4., 2.),
    ///     (4., 3.),
    ///     (5., 4.),
    ///     (6., 4.),
    ///     (7., 3.),
    ///     (7., 2.),
    ///     (6., 1.),
    ///     (5., 1.)
    /// ];
    /// let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// let poly = Polygon(ls, vec![]);
    /// // A Random point outside the polygon
    /// let p = Point::new(2.5, 0.5);
    /// let dist = p.distance(&poly);
    /// assert_eq!(dist, 2.1213203435596424);
    ///
    /// // Point to LineString example
    /// let points = vec![
    ///     (5., 1.),
    ///     (4., 2.),
    ///     (4., 3.),
    ///     (5., 4.),
    ///     (6., 4.),
    ///     (7., 3.),
    ///     (7., 2.),
    ///     (6., 1.),
    /// ];
    /// let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
    /// // A Random point outside the LineString
    /// let p = Point::new(5.5, 2.1);
    /// let dist = p.distance(&ls);
    /// assert_eq!(dist, 1.1313708498984758);
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
// adapted from http://stackoverflow.com/a/1501725/416626. Quoting the author:
//
// The projection of point p onto a line is the point on the line closest to p.
// (and a perpendicular to the line at the projection will pass through p).
// The number t is how far along the line segment from start to end that the projection falls:
// If t is 0, the projection falls right on start; if it's 1, it falls on end; if it's 0.5,
// then it's halfway between. If t is less than 0 or greater than 1, it
// falls on the line past one end or the other of the segment. In that case the
// distance to the segment will be the distance to the nearer end
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
        // get exterior ring
        let exterior = &polygon.0;
        // exterior ring as a LineString
        let ext_ring = &exterior.0;
        // No need to continue if the polygon contains the point, or is zero-length
        if polygon.contains(self) || ext_ring.len() == 0 {
            return T::zero();
        }
        // minimum priority queue
        let mut dist_queue: BinaryHeap<Mindist<T>> = BinaryHeap::new();
        // we've got interior rings
        if polygon.1.len() > 0 {
            for ring in &polygon.1 {
                dist_queue.push(Mindist { distance: self.distance(ring) })
            }
        }
        for chunk in ext_ring.chunks(2) {
            let dist = line_segment_distance(self, &chunk[0], &chunk.last().unwrap_or(&chunk[0]));
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
        // No need to continue if the point is on the LineString, or it's empty
        if linestring.contains(self) || linestring.0.len() == 0 {
            return T::zero();
        }
        // minimum priority queue
        let mut dist_queue: BinaryHeap<Mindist<T>> = BinaryHeap::new();
        // get points vector
        let points = &linestring.0;
        for chunk in points.chunks(2) {
            let dist = line_segment_distance(self, &chunk[0], &chunk.last().unwrap_or(&chunk[0]));
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
        assert_eq!(dist, 2.0485900789263356);
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
        let points = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.), (7., 2.),
                          (6., 1.), (5., 1.)];
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
        let points = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.), (7., 2.),
                          (6., 1.), (5., 1.)];
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
        let points = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.), (7., 2.),
                          (6., 1.), (5., 1.)];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon(ls, vec![]);
        // A point on the octagon
        let p = Point::new(5.0, 1.0);
        let dist = p.distance(&poly);
        assert_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon, empty Polygon
    fn point_polygon_empty_test() {
        // an empty Polygon
        let points = vec![];
        let ls = LineString(points);
        let poly = Polygon(ls, vec![]);
        // A point on the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance(&poly);
        assert_eq!(dist, 0.0);
    }
    #[test]
    // Point to Polygon with an interior ring
    fn point_polygon_interior_cutout_test() {
        // an octagon
        let ext_points = vec![
            (4., 1.),
            (5., 2.),
            (5., 3.),
            (4., 4.),
            (3., 4.),
            (2., 3.),
            (2., 2.),
            (3., 1.),
            (4., 1.),
        ];
        // cut out a triangle inside octagon
        let int_points = vec![
            (3.5, 3.5),
            (4.4, 1.5),
            (2.6, 1.5),
            (3.5, 3.5)
        ];
        let ls_ext = LineString(ext_points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let ls_int = LineString(int_points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon(ls_ext, vec![ls_int]);
        // A point inside the cutout triangle
        let p = Point::new(3.5, 2.5);
        let dist = p.distance(&poly);
        assert_eq!(dist, 0.41036467732879767);
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
    // Point to LineString, empty LineString
    fn point_linestring_empty_test() {
        let points = vec![];
        let ls = LineString(points);
        let p = Point::new(5.0, 4.0);
        let dist = p.distance(&ls);
        assert_eq!(dist, 0.0);
    }
    #[test]
    fn distance1_test() {
        assert_eq!(Point::<f64>::new(0., 0.).distance(&Point::<f64>::new(1., 0.)),
                   1.);
    }
    #[test]
    fn distance2_test() {
        // Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612)) = 146.99163308930207
        let dist = Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612));
        assert!(dist < 147. && dist > 146.);
    }
}
