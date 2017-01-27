use std::cmp::Ordering;
use std::collections::BinaryHeap;
use num_traits::Float;
use types::Point;
use traits::{PointTrait, LineStringTrait, PolygonTrait};
use num_traits::pow::pow;

pub fn point_to_point<'a, P1, P2, T>(point1: &'a P1, point2: &'a P2) -> T
    where T: 'a + Float ,
          P1: 'a + PointTrait<T> + ?Sized,
          P2: 'a + PointTrait<T> + ?Sized,
{
    let (dx, dy) = (point1.x() - point2.x(), point1.y() - point2.y());
    dx.hypot(dy)
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
pub fn line_segment_distance<'a, P1, P2, P3, T>(point: &'a P1, start: &'a P2, end: &'a P3) -> T
    where T: 'a + Float ,
          P1: 'a + PointTrait<T> + ?Sized,
          P2: 'a + PointTrait<T> + ?Sized,
          P3: 'a + PointTrait<T> + ?Sized,
{
    let dist_squared = pow(start.distance_to_point(end), 2);
    // Implies that start == end
    if dist_squared.is_zero() {
        return pow(point.distance_to_point(start), 2);
    }
    // Consider the line extending the segment, parameterized as start + t (end - start)
    // We find the projection of the point onto the line
    // This falls where t = [(point - start) . (end - start)] / |end - start|^2, where . is the dot product
    // We constrain t to a 0, 1 interval to handle points outside the segment start, end
    let t = T::zero().max(T::one().min(point.sub(start).dot(&end.sub(start)) / dist_squared));
    let projected = Point::new(start.x() + t * (end.x() - start.x()),
                               start.y() + t * (end.y() - start.y()));
    point.distance_to_point(&projected)
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
impl<T> Eq for Mindist<T> where T: Float  {}

// Minimum distance from a Point to a Polygon
pub fn polygon_to_point<'a, P1, P2, T>(polygon: &'a P1, point: &'a P2) -> T
    where T: 'a + Float ,
          P1: 'a + PolygonTrait<'a, T> + ?Sized,
          P2: 'a + PointTrait<T> + ?Sized,
{
    // get exterior ring
    // exterior ring as a LineString
    let mut rings = polygon.rings();
    // TODO: remove `collect`
    let ext_ring = rings.next().expect("no outer ring").points().collect::<Vec<_>>();

    // No need to continue if the polygon contains the point, or is zero-length
    if polygon.contains_point(point) || ext_ring.is_empty() {
        return T::zero();
    }
    // minimum priority queue
    let mut dist_queue: BinaryHeap<Mindist<T>> = BinaryHeap::new();
    // we've got interior rings
    for ring in rings {
        dist_queue.push(Mindist { distance: ring.distance_to_point(point) })
    }
    for chunk in ext_ring.windows(2) {
        let dist = line_segment_distance(point, chunk[0], chunk[1]);
        dist_queue.push(Mindist { distance: dist });
    }
    dist_queue.pop().unwrap().distance
}

// Minimum distance from a Point to a LineString
pub fn line_string_to_point<'a, L, P, T>(line_string: &'a L, point: &'a P) -> T
    where T: 'a + Float ,
          L: 'a + LineStringTrait<'a, T> + ?Sized,
          P: 'a + PointTrait<T> + ?Sized,
{
    // No need to continue if the point is on the LineString, or it's empty
    if line_string.contains_point(point) || line_string.points().next().is_none() {
        return T::zero();
    }
    // minimum priority queue
    let mut dist_queue: BinaryHeap<Mindist<T>> = BinaryHeap::new();
    // get points vector
    // TODO: remove `collect`
    let points = line_string.points().collect::<Vec<_>>();
    for chunk in points.windows(2) {
        let dist = line_segment_distance(point, chunk[0], chunk[1]);
        dist_queue.push(Mindist { distance: dist });
    }
    dist_queue.pop().unwrap().distance
}

#[cfg(test)]
mod test {
    use types::{Point, LineString, Polygon};
    use algorithm::distance::{line_segment_distance};
    use test_helpers::within_epsilon;
    use traits::PointTrait;

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
        assert!(within_epsilon(dist, 2.0485900789263356, 1.0e-15));
        assert!(within_epsilon(dist2, 1.118033988749895, 1.0e-15));
        assert!(within_epsilon(dist3, 1.4142135623730951, 1.0e-15));
        assert!(within_epsilon(dist4, 1.5811388300841898, 1.0e-15));
        // Point is on the line
        let zero_dist = line_segment_distance(&p1, &p1, &p2);
        assert!(within_epsilon(zero_dist, 0.0, 1.0e-15));
    }
    #[test]
    // Point to Polygon, outside point
    fn point_polygon_distance_outside_test() {
        // an octagon
        let points = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.), (7., 2.),
                          (6., 1.), (5., 1.)];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A Random point outside the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance_to_polygon(&poly);
        assert!(within_epsilon(dist, 2.1213203435596424, 1.0e-15));
    }
    #[test]
    // Point to Polygon, inside point
    fn point_polygon_distance_inside_test() {
        // an octagon
        let points = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.), (7., 2.),
                          (6., 1.), (5., 1.)];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A Random point inside the octagon
        let p = Point::new(5.5, 2.1);
        let dist = p.distance_to_polygon(&poly);
        assert!(within_epsilon(dist, 0.0, 1.0e-15));
    }
    #[test]
    // Point to Polygon, on boundary
    fn point_polygon_distance_boundary_test() {
        // an octagon
        let points = vec![(5., 1.), (4., 2.), (4., 3.), (5., 4.), (6., 4.), (7., 3.), (7., 2.),
                          (6., 1.), (5., 1.)];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let poly = Polygon::new(ls, vec![]);
        // A point on the octagon
        let p = Point::new(5.0, 1.0);
        let dist = p.distance_to_polygon(&poly);
        assert!(within_epsilon(dist, 0.0, 1.0e-15));
    }
    #[test]
    // Point to Polygon, empty Polygon
    fn point_polygon_empty_test() {
        // an empty Polygon
        let points = vec![];
        let ls = LineString(points);
        let poly = Polygon::new(ls, vec![]);
        // A point on the octagon
        let p = Point::new(2.5, 0.5);
        let dist = p.distance_to_polygon(&poly);
        assert!(within_epsilon(dist, 0.0, 1.0e-15));
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
        let poly = Polygon::new(ls_ext, vec![ls_int]);
        // A point inside the cutout triangle
        let p = Point::new(3.5, 2.5);
        let dist = p.distance_to_polygon(&poly);
                      // 0.41036467732879783 <-- Shapely
        assert!(within_epsilon(dist, 0.41036467732879767, 1.0e-15));
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
        let dist = p.distance_to_line_string(&ls);
        assert!(within_epsilon(dist, 1.1313708498984758, 1.0e-15));
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
        let dist = p.distance_to_line_string(&ls);
        assert!(within_epsilon(dist, 0.0, 1.0e-15));
    }
    #[test]
    // Point to LineString, closed triangle
    fn point_linestring_triangle_test() {
        let points = vec![
            (3.5, 3.5),
            (4.4, 2.0),
            (2.6, 2.0),
            (3.5, 3.5)
        ];
        let ls = LineString(points.iter().map(|e| Point::new(e.0, e.1)).collect());
        let p = Point::new(3.5, 2.5);
        let dist = p.distance_to_line_string(&ls);
        assert!(within_epsilon(dist, 0.5, 1.0e-15));
    }
    #[test]
    // Point to LineString, empty LineString
    fn point_linestring_empty_test() {
        let points = vec![];
        let ls = LineString(points);
        let p = Point::new(5.0, 4.0);
        let dist = p.distance_to_line_string(&ls);
        assert!(within_epsilon(dist, 0.0, 1.0e-15));
    }
    #[test]
    fn distance1_test() {
        assert_eq!(Point::<f64>::new(0., 0.).distance_to_point(&Point::<f64>::new(1., 0.)),
                   1.);
    }
    #[test]
    fn distance2_test() {
        // Point::new(-72.1235, 42.3521).distance(&Point::new(72.1260, 70.612)) = 146.99163308930207
        let dist = Point::new(-72.1235, 42.3521).distance_to_point(&Point::new(72.1260, 70.612));
        assert!(dist < 147. && dist > 146.);
    }
}
