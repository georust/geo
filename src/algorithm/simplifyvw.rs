use std::cmp::Ordering;
use std::collections::BinaryHeap;
use num_traits::Float;
use types::{LineString, Line, MultiLineString, MultiPolygon, Point, Polygon};
use algorithm::boundingbox::BoundingBox;

use spade::SpadeFloat;
use spade::BoundingRect;
use spade::rtree::RTree;

// Store triangle information
// current is the candidate point for removal
#[derive(Debug)]
struct VScore<T>
where
    T: Float,
{
    left: usize,
    current: usize,
    right: usize,
    area: T,
    intersector: bool,
}

// These impls give us a min-heap
impl<T> Ord for VScore<T>
where
    T: Float,
{
    fn cmp(&self, other: &VScore<T>) -> Ordering {
        other.area.partial_cmp(&self.area).unwrap()
    }
}

impl<T> PartialOrd for VScore<T>
where
    T: Float,
{
    fn partial_cmp(&self, other: &VScore<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for VScore<T>
where
    T: Float,
{
}

impl<T> PartialEq for VScore<T>
where
    T: Float,
{
    fn eq(&self, other: &VScore<T>) -> bool
    where
        T: Float,
    {
        self.area == other.area
    }
}

// Geometries that can be simplified using the topology-preserving variant
#[derive(Debug, Clone, Copy)]
enum GeomType {
    Line,
    Ring,
}
// initial min: if we ever have fewer than these, stop immediately
// min_points: if we detect a self-intersection before point removal, and we only
// have min_points left, stop: since a self-intersection causes removal of the spatially previous
// point, THAT could lead to a further self-intersection without the possibility of removing
// more points, potentially leaving the geometry in an invalid state.
#[derive(Debug, Clone, Copy)]
struct GeomSettings {
    initial_min: usize,
    min_points: usize,
    geomtype: GeomType,
}

/// Simplify a line using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm
//
// epsilon is the minimum triangle area
// The paper states that:
// If [the new triangle's] calculated area is less than that of the last point to be
// eliminated, use the latter's area instead.
// (This ensures that the current point cannot be eliminated
// without eliminating previously eliminated points)
// (Visvalingam and Whyatt 2013, p47)
// However, this does *not* apply if you're using a user-defined epsilon;
// It's OK to remove triangles with areas below the epsilon,
// then recalculate the new triangle area and push it onto the heap
// based on Huon Wilson's original implementation:
// https://github.com/huonw/isrustfastyet/blob/25e7a68ff26673a8556b170d3c9af52e1c818288/mem/line_simplify.rs
fn visvalingam<T>(orig: &[Point<T>], epsilon: &T) -> Vec<Point<T>>
where
    T: Float,
{
    // No need to continue without at least three points
    if orig.len() < 3 || orig.is_empty() {
        return orig.to_vec();
    }

    let max = orig.len();

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
    // necessary in the main loop by checking the corresponding entry in
    // adjacent for (0, 0) values
    let mut pq = BinaryHeap::new();
    // Compute the initial triangles, i.e. take all consecutive groups
    // of 3 points and form triangles from them
    for (i, win) in orig.windows(3).enumerate() {
        pq.push(VScore {
            area: area(win.first().unwrap(), &win[1], win.last().unwrap()),
            current: i + 1,
            left: i,
            right: i + 2,
            intersector: false,
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
            //  This triangle's area is below epsilon: eliminate the associated point
            Some(s) => s,
        };
        let (left, right) = adjacent[smallest.current];
        // A point in this triangle has been removed since this VScore
        // was created, so skip it
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

        // Now recompute the adjacent triangle(s), using left and right adjacent points
        let choices = [(ll, left, right), (left, right, rr)];
        for &(ai, current_point, bi) in &choices {
            if ai as usize >= max || bi as usize >= max {
                // Out of bounds, i.e. we're on one edge
                continue;
            }
            let new_left = Point::new(orig[ai as usize].x(), orig[ai as usize].y());
            let new_current = Point::new(
                orig[current_point as usize].x(),
                orig[current_point as usize].y(),
            );
            let new_right = Point::new(orig[bi as usize].x(), orig[bi as usize].y());
            pq.push(VScore {
                area: area(&new_left, &new_current, &new_right),
                current: current_point as usize,
                left: ai as usize,
                right: bi as usize,
                intersector: false,
            });
        }
    }
    // Filter out the points that have been deleted, returning remaining points
    orig.iter()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(*tup) } else { None })
        .collect::<Vec<Point<T>>>()
}

/// Wrap the actual VW function so the R* Tree can be shared.
// this ensures that shell and rings have access to all segments, so
// intersections between outer and inner rings are detected
fn vwp_wrapper<T>(
    geomtype: &GeomSettings,
    exterior: &LineString<T>,
    interiors: Option<&[LineString<T>]>,
    epsilon: &T,
) -> Vec<Vec<Point<T>>>
where
    T: Float + SpadeFloat,
{
    let mut rings = vec![];
    // Populate R* tree with exterior line segments
    // let ls = exterior
    //     .lines()
    //     .map(|line| SimpleEdge::new(line.start, line.end))
    //     .collect();
    let mut tree: RTree<Line<_>> = RTree::bulk_load(exterior.lines().collect());
    // and with interior segments, if any
    if let Some(interior_rings) = interiors {
        for ring in interior_rings {
            for line in ring.lines() {
                tree.insert(line);
            }
        }
    }
    // Simplify shell
    rings.push(visvalingam_preserve(
        geomtype,
        &exterior.0,
        epsilon,
        &mut tree,
    ));
    // Simplify interior rings, if any
    if let Some(interior_rings) = interiors {
        for ring in interior_rings {
            rings.push(visvalingam_preserve(geomtype, &ring.0, epsilon, &mut tree))
        }
    }
    rings
}

/// Visvalingam-Whyatt with self-intersection detection to preserve topologies
// this is a port of the technique at https://www.jasondavies.com/simplify/
fn visvalingam_preserve<T>(
    geomtype: &GeomSettings,
    orig: &[Point<T>],
    epsilon: &T,
    tree: &mut RTree<Line<T>>,
) -> Vec<Point<T>>
where
    T: Float + SpadeFloat,
{
    if orig.is_empty() || orig.len() < 3 {
        return orig.to_vec();
    }
    let max = orig.len();
    let mut counter = orig.len();
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
    // necessary in the main loop by checking the corresponding entry in
    // adjacent for (0, 0) values
    let mut pq = BinaryHeap::new();
    // Compute the initial triangles, i.e. take all consecutive groups
    // of 3 points and form triangles from them
    for (i, win) in orig.windows(3).enumerate() {
        let v = VScore {
            area: area(&win[0], &win[1], &win[2]),
            current: i + 1,
            left: i,
            right: i + 2,
            intersector: false,
        };
        pq.push(v);
    }
    // While there are still points for which the associated triangle
    // has an area below the epsilon
    loop {
        let mut smallest = match pq.pop() {
            // We've exhausted all the possible triangles, so leave the main loop
            None => break,
            // This triangle's area is above epsilon, so skip it
            Some(ref x) if x.area > *epsilon => continue,
            //  This triangle's area is below epsilon: eliminate the associated point
            Some(s) => s,
        };
        if counter <= geomtype.initial_min {
            // we can't remove any more points no matter what
            break;
        }
        let (left, right) = adjacent[smallest.current];
        // A point in this triangle has been removed since this VScore
        // was created, so skip it
        if left as i32 != smallest.left as i32 || right as i32 != smallest.right as i32 {
            continue;
        }
        // if removal of this point causes a self-intersection, we also remove the previous point
        // that removal alters the geometry, removing the self-intersection
        // HOWEVER if we're within 2 points of the absolute minimum, we can't remove this point or the next
        // because we could then no longer form a valid geometry if removal of next also caused an intersection.
        // The simplification process is thus over.
        smallest.intersector = tree_intersect(tree, &smallest, orig);
        if smallest.intersector && counter <= geomtype.min_points {
            break;
        }
        // We've got a valid triangle, and its area is smaller than epsilon, so
        // remove it from the simulated "linked list"
        adjacent[smallest.current as usize] = (0, 0);
        counter -= 1;
        // remove stale segments from R* tree
        // we have to call this twice because only one segment is returned at a time
        // this should be OK because a point can only share at most two segments
        tree.lookup_and_remove(&orig[smallest.right]);
        tree.lookup_and_remove(&orig[smallest.left]);
        // Now recompute the adjacent triangle(s), using left and right adjacent points
        let (ll, _) = adjacent[left as usize];
        let (_, rr) = adjacent[right as usize];
        adjacent[left as usize] = (ll, right);
        adjacent[right as usize] = (left, rr);
        let choices = [(ll, left, right), (left, right, rr)];
        for &(ai, current_point, bi) in &choices {
            if ai as usize >= max || bi as usize >= max {
                // Out of bounds, i.e. we're on one edge
                continue;
            }
            let new_left = Point::new(orig[ai as usize].x(), orig[ai as usize].y());
            let new_current = Point::new(
                orig[current_point as usize].x(),
                orig[current_point as usize].y(),
            );
            let new_right = Point::new(orig[bi as usize].x(), orig[bi as usize].y());
            // The current point causes a self-intersection, and this point precedes it
            // we ensure it gets removed next by demoting its area to negative epsilon
            let temp_area = if smallest.intersector && (current_point as usize) < smallest.current {
                -*epsilon
            } else {
                area(&new_left, &new_current, &new_right)
            };
            let new_triangle = VScore {
                area: temp_area,
                current: current_point as usize,
                left: ai as usize,
                right: bi as usize,
                intersector: false,
            };
            // add re-computed line segments to the tree
            tree.insert(Line::new(
                orig[ai as usize],
                orig[current_point as usize],
            ));
            tree.insert(Line::new(
                orig[current_point as usize],
                orig[bi as usize],
            ));
            // push re-computed triangle onto heap
            pq.push(new_triangle);
        }
    }
    // Filter out the points that have been deleted, returning remaining points
    orig.iter()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(*tup) } else { None })
        .collect::<Vec<Point<T>>>()
}

// is p1 -> p2 -> p3 wound counterclockwise?
fn ccw<T>(p1: &Point<T>, p2: &Point<T>, p3: &Point<T>) -> bool
where
    T: Float,
{
    (p3.y() - p1.y()) * (p2.x() - p1.x()) > (p2.y() - p1.y()) * (p3.x() - p1.x())
}

// checks whether line segments with p1-p4 as their start and endpoints touch or cross
fn cartesian_intersect<T>(p1: &Point<T>, p2: &Point<T>, p3: &Point<T>, p4: &Point<T>) -> bool
where
    T: Float,
{
    (ccw(p1, p3, p4) ^ ccw(p2, p3, p4)) & (ccw(p1, p2, p3) ^ ccw(p1, p2, p4))
}

// check whether a triangle's edges intersect with any other edges of the LineString
fn tree_intersect<T>(tree: &RTree<Line<T>>, triangle: &VScore<T>, orig: &[Point<T>]) -> bool
where
    T: Float + SpadeFloat,
{
    let point_a = orig[triangle.left];
    let point_b = orig[triangle.current];
    let point_c = orig[triangle.right];
    let bbox = LineString(vec![
        orig[triangle.left],
        orig[triangle.current],
        orig[triangle.right],
    ]).bbox()
        .unwrap();
    let br = Point::new(bbox.xmin, bbox.ymin);
    let tl = Point::new(bbox.xmax, bbox.ymax);
    let candidates = tree.lookup_in_rectangle(&BoundingRect::from_corners(&br, &tl));
    candidates.iter().any(|c| {
        // triangle start point, end point
        let ca = c.start;
        let cb = c.end;
        if ca != point_a && ca != point_c && cb != point_a && cb != point_c
            && cartesian_intersect(&ca, &cb, &point_a, &point_c)
        {
            true
        } else {
            ca != point_b && ca != point_c && cb != point_b && cb != point_c
                && cartesian_intersect(&ca, &cb, &point_b, &point_c)
        }
    })
}

// Area of a triangle given three vertices
fn area<T>(p1: &Point<T>, p2: &Point<T>, p3: &Point<T>) -> T
where
    T: Float,
{
    ((p1.x() - p3.x()) * (p2.y() - p3.y()) - (p2.x() - p3.x()) * (p1.y() - p3.y())).abs()
        / (T::one() + T::one()).abs()
}

/// Simplifies a geometry.
///
/// Polygons are simplified by running the algorithm on all their constituent rings.  This may
/// result in invalid Polygons, and has no guarantee of preserving topology. Multi* objects are
/// simplified by simplifying all their constituent geometries individually.
pub trait SimplifyVW<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm
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
    fn simplifyvw(&self, epsilon: &T) -> Self
    where
        T: Float;
}

/// Simplifies a geometry, preserving its topology by removing self-intersections
pub trait SimplifyVWPreserve<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using a topology-preserving variant of the
    /// [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm.
    ///
    /// See [here](https://www.jasondavies.com/simplify/) for a graphical explanation.
    ///
    /// The topology-preserving algorithm uses an [R* tree](../../../spade/rtree/struct.RTree.html) to
    /// efficiently find candidate line segments which are tested for intersection with a given triangle.
    /// If intersections are found, the previous point (i.e. the left component of the current triangle)
    /// is also removed, altering the geometry and removing the intersection.
    ///
    /// In the example below, `(135.0, 68.0)` would be retained by the standard algorithm,
    /// forming triangle `(0, 1, 3),` which intersects with the segments `(280.0, 19.0),
    /// (117.0, 48.0)` and `(117.0, 48.0), (300,0, 40.0)`. By removing it,
    /// a new triangle with indices `(0, 3, 4)` is formed, which does not cause a self-intersection.
    ///
    /// **Note**: it is possible for the simplification algorithm to displace a Polygon's interior ring outside its shell.
    ///
    /// **Note**: if removal of a point causes a self-intersection, but the geometry only has `n + 2`
    /// points remaining (4 for a `LineString`, 6 for a `Polygon`), the point is retained and the
    /// simplification process ends. This is because there is no guarantee that removal of two points will remove
    /// the intersection, but removal of further points would leave too few points to form a valid geometry.
    ///
    /// ```
    /// use geo::{Point, LineString};
    /// use geo::algorithm::simplifyvw::{SimplifyVWPreserve};
    ///
    /// let mut vec = Vec::new();
    /// vec.push(Point::new(10., 60.));
    /// vec.push(Point::new(135., 68.));
    /// vec.push(Point::new(94., 48.));
    /// vec.push(Point::new(126., 31.));
    /// vec.push(Point::new(280., 19.));
    /// vec.push(Point::new(117., 48.));
    /// vec.push(Point::new(300., 40.));
    /// vec.push(Point::new(301., 10.));
    /// let linestring = LineString(vec);
    /// let mut compare = Vec::new();
    /// compare.push(Point::new(10., 60.));
    /// compare.push(Point::new(126., 31.));
    /// compare.push(Point::new(280., 19.));
    /// compare.push(Point::new(117., 48.));
    /// compare.push(Point::new(300., 40.));
    /// compare.push(Point::new(301., 10.));
    /// let ls_compare = LineString(compare);
    /// let simplified = linestring.simplifyvw_preserve(&668.6);
    /// assert_eq!(simplified, ls_compare)
    /// ```
    fn simplifyvw_preserve(&self, epsilon: &T) -> Self
    where
        T: Float + SpadeFloat;
}

impl<T> SimplifyVWPreserve<T> for LineString<T>
where
    T: Float + SpadeFloat,
{
    fn simplifyvw_preserve(&self, epsilon: &T) -> LineString<T> {
        let gt = GeomSettings {
            initial_min: 2,
            min_points: 4,
            geomtype: GeomType::Line,
        };
        let mut simplified = vwp_wrapper(&gt, self, None, epsilon);
        LineString(simplified.pop().unwrap())
    }
}

impl<T> SimplifyVWPreserve<T> for MultiLineString<T>
where
    T: Float + SpadeFloat,
{
    fn simplifyvw_preserve(&self, epsilon: &T) -> MultiLineString<T> {
        MultiLineString(
            self.0
                .iter()
                .map(|l| l.simplifyvw_preserve(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVWPreserve<T> for Polygon<T>
where
    T: Float + SpadeFloat,
{
    fn simplifyvw_preserve(&self, epsilon: &T) -> Polygon<T> {
        let gt = GeomSettings {
            initial_min: 4,
            min_points: 6,
            geomtype: GeomType::Ring,
        };
        let mut simplified = vwp_wrapper(&gt, &self.exterior, Some(&self.interiors), epsilon);
        let exterior = LineString(simplified.remove(0));
        let interiors = simplified.into_iter().map(LineString).collect();
        Polygon::new(exterior, interiors)
    }
}

impl<T> SimplifyVWPreserve<T> for MultiPolygon<T>
where
    T: Float + SpadeFloat,
{
    fn simplifyvw_preserve(&self, epsilon: &T) -> MultiPolygon<T> {
        MultiPolygon(
            self.0
                .iter()
                .map(|p| p.simplifyvw_preserve(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVW<T> for LineString<T>
where
    T: Float,
{
    fn simplifyvw(&self, epsilon: &T) -> LineString<T> {
        LineString(visvalingam(&self.0, epsilon))
    }
}

impl<T> SimplifyVW<T> for MultiLineString<T>
where
    T: Float,
{
    fn simplifyvw(&self, epsilon: &T) -> MultiLineString<T> {
        MultiLineString(self.0.iter().map(|l| l.simplifyvw(epsilon)).collect())
    }
}

impl<T> SimplifyVW<T> for Polygon<T>
where
    T: Float,
{
    fn simplifyvw(&self, epsilon: &T) -> Polygon<T> {
        Polygon::new(
            self.exterior.simplifyvw(epsilon),
            self.interiors
                .iter()
                .map(|l| l.simplifyvw(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVW<T> for MultiPolygon<T>
where
    T: Float,
{
    fn simplifyvw(&self, epsilon: &T) -> MultiPolygon<T> {
        MultiPolygon(self.0.iter().map(|p| p.simplifyvw(epsilon)).collect())
    }
}

#[cfg(test)]
mod test {
    use types::{LineString, MultiLineString, MultiPolygon, Point, Polygon};
    use super::{cartesian_intersect, visvalingam, vwp_wrapper, GeomSettings, GeomType, SimplifyVW,
                SimplifyVWPreserve};

    #[test]
    fn visvalingam_test() {
        // this is the PostGIS example
        let points = vec![
            (5.0, 2.0),
            (3.0, 8.0),
            (6.0, 20.0),
            (7.0, 25.0),
            (10.0, 10.0),
        ];
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e.0, e.1)).collect();

        let correct = vec![(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();

        let simplified = visvalingam(&points_ls, &30.);
        assert_eq!(simplified, correct_ls);
    }
    #[test]
    fn vwp_intersection_test() {
        // does the intersection check always work
        let a = Point::new(1., 3.);
        let b = Point::new(3., 1.);
        let c = Point::new(3., 3.);
        let d = Point::new(1., 1.);
        // cw + ccw
        assert_eq!(cartesian_intersect(&a, &b, &c, &d), true);
        // ccw + ccw
        assert_eq!(cartesian_intersect(&b, &a, &c, &d), true);
        // cw + cw
        assert_eq!(cartesian_intersect(&a, &b, &d, &c), true);
        // ccw + cw
        assert_eq!(cartesian_intersect(&b, &a, &d, &c), true);
    }
    #[test]
    fn simple_vwp_test() {
        // this LineString will have a self-intersection if the point with the
        // smallest associated area is removed
        // the associated triangle is (1, 2, 3), and has an area of 668.5
        // the new triangle (0, 1, 3) self-intersects with triangle (3, 4, 5)
        // Point 1 must also be removed giving a final, valid
        // LineString of (0, 3, 4, 5, 6, 7)
        let points = vec![
            (10., 60.),
            (135., 68.),
            (94., 48.),
            (126., 31.),
            (280., 19.),
            (117., 48.),
            (300., 40.),
            (301., 10.),
        ];
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e.0, e.1)).collect();
        let gt = &GeomSettings {
            initial_min: 2,
            min_points: 4,
            geomtype: GeomType::Line,
        };
        let simplified = vwp_wrapper(&gt, &points_ls.into(), None, &668.6);
        // this is the correct, non-intersecting LineString
        let correct = vec![
            (10., 60.),
            (126., 31.),
            (280., 19.),
            (117., 48.),
            (300., 40.),
            (301., 10.),
        ];
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();
        assert_eq!(simplified[0], correct_ls);
    }
    #[test]
    fn retained_vwp_test() {
        // we would expect outer[2] to be removed, as its associated area
        // is below epsilon. However, this causes a self-intersection
        // with the inner ring, which would also trigger removal of outer[1],
        // leaving the geometry below min_points. It is thus retained.
        // Inner should also be reduced, but has points == initial_min for the Polygon type
        let outer = LineString(vec![
            Point::new(-54.4921875, 21.289374355860424),
            Point::new(-33.5, 56.9449741808516),
            Point::new(-22.5, 44.08758502824516),
            Point::new(-19.5, 23.241346102386135),
            Point::new(-54.4921875, 21.289374355860424),
        ]);
        let inner = LineString(vec![
            Point::new(-24.451171875, 35.266685523707665),
            Point::new(-29.513671875, 47.32027765985069),
            Point::new(-22.869140625, 43.80817468459856),
            Point::new(-24.451171875, 35.266685523707665),
        ]);
        let poly = Polygon::new(outer.clone(), vec![inner]);
        let simplified = poly.simplifyvw_preserve(&95.4);
        assert_eq!(simplified.exterior, outer);
    }
    #[test]
    fn remove_inner_point_vwp_test() {
        // we would expect outer[2] to be removed, as its associated area
        // is below epsilon. However, this causes a self-intersection
        // with the inner ring, which would also trigger removal of outer[1],
        // leaving the geometry below min_points. It is thus retained.
        // Inner should be reduced to four points by removing inner[2]
        let outer = LineString(vec![
            Point::new(-54.4921875, 21.289374355860424),
            Point::new(-33.5, 56.9449741808516),
            Point::new(-22.5, 44.08758502824516),
            Point::new(-19.5, 23.241346102386135),
            Point::new(-54.4921875, 21.289374355860424),
        ]);
        let inner = LineString(vec![
            Point::new(-24.451171875, 35.266685523707665),
            Point::new(-40.0, 45.),
            Point::new(-29.513671875, 47.32027765985069),
            Point::new(-22.869140625, 43.80817468459856),
            Point::new(-24.451171875, 35.266685523707665),
        ]);
        let correct_inner = LineString(vec![
            Point::new(-24.451171875, 35.266685523707665),
            Point::new(-40.0, 45.0),
            Point::new(-22.869140625, 43.80817468459856),
            Point::new(-24.451171875, 35.266685523707665),
        ]);
        let poly = Polygon::new(outer.clone(), vec![inner]);
        let simplified = poly.simplifyvw_preserve(&95.4);
        assert_eq!(simplified.exterior, outer);
        assert_eq!(simplified.interiors[0], correct_inner);
    }
    #[test]
    fn very_long_vwp_test() {
        // simplify an 8k-point LineString, eliminating self-intersections
        let points = include!("test_fixtures/norway_main.rs");
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e[0], e[1])).collect();
        let gt = &GeomSettings {
            initial_min: 2,
            min_points: 4,
            geomtype: GeomType::Line,
        };
        let simplified = vwp_wrapper(&gt, &points_ls.into(), None, &0.0005);
        assert_eq!(simplified[0].len(), 3276);
    }
    #[test]
    fn visvalingam_test_long() {
        // simplify a longer LineString
        let points = include!("test_fixtures/vw_orig.rs");
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e[0], e[1])).collect();
        let correct = include!("test_fixtures/vw_simplified.rs");
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e[0], e[1])).collect();
        let simplified = visvalingam(&points_ls, &0.0005);
        assert_eq!(simplified, correct_ls);
    }
    #[test]
    fn visvalingam_preserve_test_long() {
        // simplify a longer LineString using the preserve variant
        let points = include!("test_fixtures/vw_orig.rs");
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e[0], e[1])).collect();
        let correct = include!("test_fixtures/vw_simplified.rs");
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e[0], e[1])).collect();
        let simplified = LineString(points_ls).simplifyvw_preserve(&0.0005);
        assert_eq!(simplified, LineString(correct_ls));
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

    #[test]
    fn multilinestring() {
        // this is the PostGIS example
        let points = vec![
            (5.0, 2.0),
            (3.0, 8.0),
            (6.0, 20.0),
            (7.0, 25.0),
            (10.0, 10.0),
        ];
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e.0, e.1)).collect();

        let correct = vec![(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();

        let mline = MultiLineString(vec![LineString(points_ls)]);
        assert_eq!(
            mline.simplifyvw(&30.),
            MultiLineString(vec![LineString(correct_ls)])
        );
    }

    #[test]
    fn polygon() {
        let poly = Polygon::new(
            LineString(vec![
                Point::new(0., 0.),
                Point::new(0., 10.),
                Point::new(5., 11.),
                Point::new(10., 10.),
                Point::new(10., 0.),
                Point::new(0., 0.),
            ]),
            vec![],
        );

        let poly2 = poly.simplifyvw(&10.);

        assert_eq!(
            poly2,
            Polygon::new(
                LineString(vec![
                    Point::new(0., 0.),
                    Point::new(0., 10.),
                    Point::new(10., 10.),
                    Point::new(10., 0.),
                    Point::new(0., 0.),
                ]),
                vec![],
            )
        );
    }

    #[test]
    fn multipolygon() {
        let mpoly = MultiPolygon(vec![
            Polygon::new(
                LineString(vec![
                    Point::new(0., 0.),
                    Point::new(0., 10.),
                    Point::new(5., 11.),
                    Point::new(10., 10.),
                    Point::new(10., 0.),
                    Point::new(0., 0.),
                ]),
                vec![],
            ),
        ]);

        let mpoly2 = mpoly.simplifyvw(&10.);

        assert_eq!(
            mpoly2,
            MultiPolygon(vec![
                Polygon::new(
                    LineString(vec![
                        Point::new(0., 0.),
                        Point::new(0., 10.),
                        Point::new(10., 10.),
                        Point::new(10., 0.),
                        Point::new(0., 0.),
                    ]),
                    vec![],
                ),
            ])
        );
    }
}
