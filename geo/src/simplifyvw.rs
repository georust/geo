use crate::prelude::*;
use crate::{
    Coordinate, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon, Triangle,
};
use num_traits::Float;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use rstar::{RTree, RTreeNum};

/// Store triangle information
// current is the candidate point for removal
#[derive(Debug)]
struct VScore<T, I>
where
    T: Float,
{
    left: usize,
    current: usize,
    right: usize,
    area: T,
    // `visvalingam_preserve` uses `intersector`, `visvalingam` does not
    intersector: I,
}

// These impls give us a min-heap
impl<T, I> Ord for VScore<T, I>
where
    T: Float,
{
    fn cmp(&self, other: &VScore<T, I>) -> Ordering {
        other.area.partial_cmp(&self.area).unwrap()
    }
}

impl<T, I> PartialOrd for VScore<T, I>
where
    T: Float,
{
    fn partial_cmp(&self, other: &VScore<T, I>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, I> Eq for VScore<T, I> where T: Float {}

impl<T, I> PartialEq for VScore<T, I>
where
    T: Float,
{
    fn eq(&self, other: &VScore<T, I>) -> bool
    where
        T: Float,
    {
        self.area == other.area
    }
}

/// Geometries that can be simplified using the topology-preserving variant
#[derive(Debug, Clone, Copy)]
enum GeomType {
    Line,
    Ring,
}

/// Settings for Ring and Line geometries
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
// This method returns the **indices** of the simplified line
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
fn visvalingam_indices<T>(orig: &LineString<T>, epsilon: &T) -> Vec<usize>
where
    T: Float,
{
    // No need to continue without at least three points
    if orig.0.len() < 3 {
        return orig.0.iter().enumerate().map(|(idx, _)| idx).collect();
    }

    let max = orig.0.len();

    // Adjacent retained points. Simulating the points in a
    // linked list with indices into `orig`. Big number (larger than or equal to
    // `max`) means no next element, and (0, 0) means deleted element.
    let mut adjacent: Vec<_> = (0..orig.0.len())
        .map(|i| {
            if i == 0 {
                (-1_i32, 1_i32)
            } else {
                ((i - 1) as i32, (i + 1) as i32)
            }
        })
        .collect();

    // Store all the triangles in a minimum priority queue, based on their area.
    //
    // Invalid triangles are *not* removed if / when points are removed; they're
    // handled by skipping them as necessary in the main loop by checking the
    // corresponding entry in adjacent for (0, 0) values

    // Compute the initial triangles
    let mut pq = orig
        .triangles()
        .enumerate()
        .map(|(i, triangle)| VScore {
            area: triangle.unsigned_area(),
            current: i + 1,
            left: i,
            right: i + 2,
            intersector: (),
        })
        .collect::<BinaryHeap<VScore<T, ()>>>();
    // While there are still points for which the associated triangle
    // has an area below the epsilon
    while let Some(smallest) = pq.pop() {
        // This triangle's area is above epsilon, so skip it
        if smallest.area > *epsilon {
            continue;
        }
        //  This triangle's area is below epsilon: eliminate the associated point
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
            let area = Triangle(
                orig.0[ai as usize],
                orig.0[current_point as usize],
                orig.0[bi as usize],
            )
            .unsigned_area();
            pq.push(VScore {
                area,
                current: current_point as usize,
                left: ai as usize,
                right: bi as usize,
                intersector: (),
            });
        }
    }
    // Filter out the points that have been deleted, returning remaining point indices
    orig.0
        .iter()
        .enumerate()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(tup.0) } else { None })
        .collect::<Vec<usize>>()
}

// Wrapper for visvalingam_indices, mapping indices back to points
fn visvalingam<T>(orig: &LineString<T>, epsilon: &T) -> Vec<Coordinate<T>>
where
    T: Float,
{
    // Epsilon must be greater than zero for any meaningful simplification to happen
    if *epsilon <= T::zero() {
        return orig.0.to_vec();
    }
    let subset = visvalingam_indices(orig, epsilon);
    // filter orig using the indices
    // using get would be more robust here, but the input subset is guaranteed to be valid in this case
    orig.0
        .iter()
        .zip(subset.iter())
        .map(|(_, s)| orig[*s])
        .collect()
}

/// Wrap the actual VW function so the R* Tree can be shared.
// this ensures that shell and rings have access to all segments, so
// intersections between outer and inner rings are detected
fn vwp_wrapper<T>(
    geomtype: &GeomSettings,
    exterior: &LineString<T>,
    interiors: Option<&[LineString<T>]>,
    epsilon: &T,
) -> Vec<Vec<Coordinate<T>>>
where
    T: Float + RTreeNum,
{
    let mut rings = vec![];
    // Populate R* tree with exterior and interior samples, if any
    let mut tree: RTree<Line<_>> = RTree::bulk_load(
        exterior
            .lines()
            .chain(
                interiors
                    .iter()
                    .flat_map(|ring| *ring)
                    .flat_map(|line_string| line_string.lines()),
            )
            .collect::<Vec<_>>(),
    );

    // Simplify shell
    rings.push(visvalingam_preserve(
        geomtype, &exterior, epsilon, &mut tree,
    ));
    // Simplify interior rings, if any
    if let Some(interior_rings) = interiors {
        for ring in interior_rings {
            rings.push(visvalingam_preserve(geomtype, &ring, epsilon, &mut tree))
        }
    }
    rings
}

/// Visvalingam-Whyatt with self-intersection detection to preserve topologies
/// this is a port of the technique at https://www.jasondavies.com/simplify/
fn visvalingam_preserve<T>(
    geomtype: &GeomSettings,
    orig: &LineString<T>,
    epsilon: &T,
    tree: &mut RTree<Line<T>>,
) -> Vec<Coordinate<T>>
where
    T: Float + RTreeNum,
{
    if orig.0.len() < 3 || *epsilon <= T::zero() {
        return orig.0.to_vec();
    }
    let max = orig.0.len();
    let mut counter = orig.0.len();

    // Adjacent retained points. Simulating the points in a
    // linked list with indices into `orig`. Big number (larger than or equal to
    // `max`) means no next element, and (0, 0) means deleted element.
    let mut adjacent: Vec<_> = (0..orig.0.len())
        .map(|i| {
            if i == 0 {
                (-1_i32, 1_i32)
            } else {
                ((i - 1) as i32, (i + 1) as i32)
            }
        })
        .collect();
    // Store all the triangles in a minimum priority queue, based on their area.
    //
    // Invalid triangles are *not* removed if / when points are removed; they're
    // handled by skipping them as necessary in the main loop by checking the
    // corresponding entry in adjacent for (0, 0) values

    // Compute the initial triangles
    let mut pq = orig
        .triangles()
        .enumerate()
        .map(|(i, triangle)| VScore {
            area: triangle.unsigned_area(),
            current: i + 1,
            left: i,
            right: i + 2,
            intersector: false,
        })
        .collect::<BinaryHeap<VScore<T, bool>>>();

    // While there are still points for which the associated triangle
    // has an area below the epsilon
    while let Some(mut smallest) = pq.pop() {
        if smallest.area > *epsilon {
            continue;
        }
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
        smallest.intersector = tree_intersect(tree, &smallest, &orig.0);
        if smallest.intersector && counter <= geomtype.min_points {
            break;
        }
        // We've got a valid triangle, and its area is smaller than epsilon, so
        // remove it from the simulated "linked list"
        adjacent[smallest.current as usize] = (0, 0);
        counter -= 1;
        // Remove stale segments from R* tree
        let left_point = Point(orig.0[left as usize]);
        let middle_point = Point(orig.0[smallest.current]);
        let right_point = Point(orig.0[right as usize]);

        let line_1 = Line::new(left_point, middle_point);
        let line_2 = Line::new(middle_point, right_point);
        assert!(tree.remove(&line_1).is_some());
        assert!(tree.remove(&line_2).is_some());

        // Restore continous line segment
        tree.insert(Line::new(left_point, right_point));

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
            let new = Triangle(
                orig.0[ai as usize],
                orig.0[current_point as usize],
                orig.0[bi as usize],
            );
            // The current point causes a self-intersection, and this point precedes it
            // we ensure it gets removed next by demoting its area to negative epsilon
            let temp_area = if smallest.intersector && (current_point as usize) < smallest.current {
                -*epsilon
            } else {
                new.unsigned_area()
            };
            let new_triangle = VScore {
                area: temp_area,
                current: current_point as usize,
                left: ai as usize,
                right: bi as usize,
                intersector: false,
            };

            // push re-computed triangle onto heap
            pq.push(new_triangle);
        }
    }
    // Filter out the points that have been deleted, returning remaining points
    orig.0
        .iter()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(*tup) } else { None })
        .collect()
}

/// is p1 -> p2 -> p3 wound counterclockwise?
fn ccw<T>(p1: Point<T>, p2: Point<T>, p3: Point<T>) -> bool
where
    T: Float,
{
    (p3.y() - p1.y()) * (p2.x() - p1.x()) > (p2.y() - p1.y()) * (p3.x() - p1.x())
}

/// checks whether line segments with p1-p4 as their start and endpoints touch or cross
fn cartesian_intersect<T>(p1: Point<T>, p2: Point<T>, p3: Point<T>, p4: Point<T>) -> bool
where
    T: Float,
{
    (ccw(p1, p3, p4) ^ ccw(p2, p3, p4)) & (ccw(p1, p2, p3) ^ ccw(p1, p2, p4))
}

/// check whether a triangle's edges intersect with any other edges of the LineString
fn tree_intersect<T>(
    tree: &RTree<Line<T>>,
    triangle: &VScore<T, bool>,
    orig: &[Coordinate<T>],
) -> bool
where
    T: Float + RTreeNum,
{
    let point_a = orig[triangle.left];
    let point_c = orig[triangle.right];
    let bounding_rect = Triangle(
        orig[triangle.left],
        orig[triangle.current],
        orig[triangle.right],
    )
    .bounding_rect();
    let br = Point::new(bounding_rect.min().x, bounding_rect.min().y);
    let tl = Point::new(bounding_rect.max().x, bounding_rect.max().y);
    tree.locate_in_envelope_intersecting(&rstar::AABB::from_corners(br, tl))
        .any(|c| {
            // triangle start point, end point
            let (ca, cb) = c.points();
            ca.0 != point_a
                && ca.0 != point_c
                && cb.0 != point_a
                && cb.0 != point_c
                && cartesian_intersect(ca, cb, Point(point_a), Point(point_c))
        })
}

/// Simplifies a geometry.
///
/// Polygons are simplified by running the algorithm on all their constituent rings.  This may
/// result in invalid Polygons, and has no guarantee of preserving topology. Multi* objects are
/// simplified by simplifying all their constituent geometries individually.
///
/// An epsilon less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyVW<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm
    ///
    /// See [here](https://bost.ocks.org/mike/simplify/) for a graphical explanation
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::simplifyvw::SimplifyVW;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 5.0, y: 2.0),
    ///     (x: 3.0, y: 8.0),
    ///     (x: 6.0, y: 20.0),
    ///     (x: 7.0, y: 25.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// let simplified = line_string.simplifyvw(&30.0);
    ///
    /// let expected = line_string![
    ///     (x: 5.0, y: 2.0),
    ///     (x: 7.0, y: 25.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplifyvw(&self, epsilon: &T) -> Self
    where
        T: Float;
}

/// Simplifies a geometry, returning the retained _indices_ of the output
///
/// This operation uses the Visvalingam-Whyatt algorithm,
/// and does **not** guarantee that the returned geometry is valid.
///
/// An epsilon less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyVwIdx<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm
    ///
    /// See [here](https://bost.ocks.org/mike/simplify/) for a graphical explanation
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::simplifyvw::SimplifyVwIdx;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 5.0, y: 2.0),
    ///     (x: 3.0, y: 8.0),
    ///     (x: 6.0, y: 20.0),
    ///     (x: 7.0, y: 25.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// let simplified = line_string.simplifyvw_idx(&30.0);
    ///
    /// let expected = vec![
    ///     0_usize,
    ///     3_usize,
    ///     4_usize,
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplifyvw_idx(&self, epsilon: &T) -> Vec<usize>
    where
        T: Float;
}

/// Simplifies a geometry, preserving its topology by removing self-intersections
///
/// An epsilon less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyVWPreserve<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using a topology-preserving variant of the
    /// [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm.
    ///
    /// See [here](https://www.jasondavies.com/simplify/) for a graphical explanation.
    ///
    /// The topology-preserving algorithm uses an [R* tree](../../../rstar/struct.RTree.html) to
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
    /// # Examples
    ///
    /// ```
    /// use geo::algorithm::simplifyvw::SimplifyVWPreserve;
    /// use geo::line_string;
    ///
    /// let line_string = line_string![
    ///     (x: 10., y: 60.),
    ///     (x: 135., y: 68.),
    ///     (x: 94., y: 48.),
    ///     (x: 126., y: 31.),
    ///     (x: 280., y: 19.),
    ///     (x: 117., y: 48.),
    ///     (x: 300., y: 40.),
    ///     (x: 301., y: 10.),
    /// ];
    ///
    /// let simplified = line_string.simplifyvw_preserve(&668.6);
    ///
    /// let expected = line_string![
    ///     (x: 10., y: 60.),
    ///     (x: 126., y: 31.),
    ///     (x: 280., y: 19.),
    ///     (x: 117., y: 48.),
    ///     (x: 300., y: 40.),
    ///     (x: 301., y: 10.),
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplifyvw_preserve(&self, epsilon: &T) -> Self
    where
        T: Float + RTreeNum;
}

impl<T> SimplifyVWPreserve<T> for LineString<T>
where
    T: Float + RTreeNum,
{
    fn simplifyvw_preserve(&self, epsilon: &T) -> LineString<T> {
        let gt = GeomSettings {
            initial_min: 2,
            min_points: 4,
            geomtype: GeomType::Line,
        };
        let mut simplified = vwp_wrapper(&gt, self, None, epsilon);
        LineString::from(simplified.pop().unwrap())
    }
}

impl<T> SimplifyVWPreserve<T> for MultiLineString<T>
where
    T: Float + RTreeNum,
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
    T: Float + RTreeNum,
{
    fn simplifyvw_preserve(&self, epsilon: &T) -> Polygon<T> {
        let gt = GeomSettings {
            initial_min: 4,
            min_points: 6,
            geomtype: GeomType::Ring,
        };
        let mut simplified = vwp_wrapper(&gt, self.exterior(), Some(self.interiors()), epsilon);
        let exterior = LineString::from(simplified.remove(0));
        let interiors = simplified.into_iter().map(LineString::from).collect();
        Polygon::new(exterior, interiors)
    }
}

impl<T> SimplifyVWPreserve<T> for MultiPolygon<T>
where
    T: Float + RTreeNum,
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
        LineString::from(visvalingam(self, epsilon))
    }
}

impl<T> SimplifyVwIdx<T> for LineString<T>
where
    T: Float,
{
    fn simplifyvw_idx(&self, epsilon: &T) -> Vec<usize> {
        visvalingam_indices(self, epsilon)
    }
}

impl<T> SimplifyVW<T> for MultiLineString<T>
where
    T: Float,
{
    fn simplifyvw(&self, epsilon: &T) -> MultiLineString<T> {
        MultiLineString(self.iter().map(|l| l.simplifyvw(epsilon)).collect())
    }
}

impl<T> SimplifyVW<T> for Polygon<T>
where
    T: Float,
{
    fn simplifyvw(&self, epsilon: &T) -> Polygon<T> {
        Polygon::new(
            self.exterior().simplifyvw(epsilon),
            self.interiors()
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
        MultiPolygon(self.iter().map(|p| p.simplifyvw(epsilon)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::{
        cartesian_intersect, visvalingam, vwp_wrapper, GeomSettings, GeomType, SimplifyVW,
        SimplifyVWPreserve,
    };
    use crate::{
        line_string, point, polygon, Coordinate, LineString, MultiLineString, MultiPolygon, Point,
        Polygon,
    };

    #[test]
    fn visvalingam_test() {
        // this is the PostGIS example
        let ls = line_string![
            (x: 5.0, y: 2.0),
            (x: 3.0, y: 8.0),
            (x: 6.0, y: 20.0),
            (x: 7.0, y: 25.0),
            (x: 10.0, y: 10.0)
        ];

        let correct = vec![(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct
            .iter()
            .map(|e| Coordinate::from((e.0, e.1)))
            .collect();

        let simplified = visvalingam(&ls, &30.);
        assert_eq!(simplified, correct_ls);
    }
    #[test]
    fn vwp_intersection_test() {
        // does the intersection check always work
        let a = point!(x: 1., y: 3.);
        let b = point!(x: 3., y: 1.);
        let c = point!(x: 3., y: 3.);
        let d = point!(x: 1., y: 1.);
        // cw + ccw
        assert_eq!(cartesian_intersect(a, b, c, d), true);
        // ccw + ccw
        assert_eq!(cartesian_intersect(b, a, c, d), true);
        // cw + cw
        assert_eq!(cartesian_intersect(a, b, d, c), true);
        // ccw + cw
        assert_eq!(cartesian_intersect(b, a, d, c), true);
    }
    #[test]
    fn simple_vwp_test() {
        // this LineString will have a self-intersection if the point with the
        // smallest associated area is removed
        // the associated triangle is (1, 2, 3), and has an area of 668.5
        // the new triangle (0, 1, 3) self-intersects with triangle (3, 4, 5)
        // Point 1 must also be removed giving a final, valid
        // LineString of (0, 3, 4, 5, 6, 7)
        let ls = line_string![
            (x: 10., y:60.),
            (x: 135., y: 68.),
            (x: 94.,  y: 48.),
            (x: 126., y: 31.),
            (x: 280., y: 19.),
            (x: 117., y: 48.),
            (x: 300., y: 40.),
            (x: 301., y: 10.)
        ];
        let gt = &GeomSettings {
            initial_min: 2,
            min_points: 4,
            geomtype: GeomType::Line,
        };
        let simplified = vwp_wrapper(&gt, &ls, None, &668.6);
        // this is the correct, non-intersecting LineString
        let correct = vec![
            (10., 60.),
            (126., 31.),
            (280., 19.),
            (117., 48.),
            (300., 40.),
            (301., 10.),
        ];
        let correct_ls: Vec<_> = correct
            .iter()
            .map(|e| Coordinate::from((e.0, e.1)))
            .collect();
        assert_eq!(simplified[0], correct_ls);
    }
    #[test]
    fn retained_vwp_test() {
        // we would expect outer[2] to be removed, as its associated area
        // is below epsilon. However, this causes a self-intersection
        // with the inner ring, which would also trigger removal of outer[1],
        // leaving the geometry below min_points. It is thus retained.
        // Inner should also be reduced, but has points == initial_min for the Polygon type
        let outer = line_string![
            (x: -54.4921875, y: 21.289374355860424),
            (x: -33.5, y: 56.9449741808516),
            (x: -22.5, y: 44.08758502824516),
            (x: -19.5, y: 23.241346102386135),
            (x: -54.4921875, y: 21.289374355860424)
        ];
        let inner = line_string![
            (x: -24.451171875, y: 35.266685523707665),
            (x: -29.513671875, y: 47.32027765985069),
            (x: -22.869140625, y: 43.80817468459856),
            (x: -24.451171875, y: 35.266685523707665)
        ];
        let poly = Polygon::new(outer.clone(), vec![inner]);
        let simplified = poly.simplifyvw_preserve(&95.4);
        assert_eq!(simplified.exterior(), &outer);
    }
    #[test]
    fn remove_inner_point_vwp_test() {
        // we would expect outer[2] to be removed, as its associated area
        // is below epsilon. However, this causes a self-intersection
        // with the inner ring, which would also trigger removal of outer[1],
        // leaving the geometry below min_points. It is thus retained.
        // Inner should be reduced to four points by removing inner[2]
        let outer = line_string![
            (x: -54.4921875, y: 21.289374355860424),
            (x: -33.5, y: 56.9449741808516),
            (x: -22.5, y: 44.08758502824516),
            (x: -19.5, y: 23.241346102386135),
            (x: -54.4921875, y: 21.289374355860424)
        ];
        let inner = line_string![
            (x: -24.451171875, y: 35.266685523707665),
            (x: -40.0, y: 45.),
            (x: -29.513671875, y: 47.32027765985069),
            (x: -22.869140625, y: 43.80817468459856),
            (x: -24.451171875, y: 35.266685523707665)
        ];
        let correct_inner = line_string![
            (x: -24.451171875, y: 35.266685523707665),
            (x: -40.0, y: 45.0),
            (x: -22.869140625, y: 43.80817468459856),
            (x: -24.451171875, y: 35.266685523707665)
        ];
        let poly = Polygon::new(outer.clone(), vec![inner]);
        let simplified = poly.simplifyvw_preserve(&95.4);
        assert_eq!(simplified.exterior(), &outer);
        assert_eq!(simplified.interiors()[0], correct_inner);
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
        assert_eq!(simplified[0].len(), 3278);
    }

    #[test]
    fn visvalingam_test_long() {
        // simplify a longer LineString
        let points = include!("test_fixtures/vw_orig.rs");
        let points_ls: LineString<_> = points.iter().map(|e| Point::new(e[0], e[1])).collect();
        let correct = include!("test_fixtures/vw_simplified.rs");
        let correct_ls: Vec<_> = correct
            .iter()
            .map(|e| Coordinate::from((e[0], e[1])))
            .collect();
        let simplified = visvalingam(&points_ls, &0.0005);
        assert_eq!(simplified, correct_ls);
    }
    #[test]
    fn visvalingam_preserve_test_long() {
        // simplify a longer LineString using the preserve variant
        let points = include!("test_fixtures/vw_orig.rs");
        let points_ls: LineString<_> = points.iter().map(|e| Point::new(e[0], e[1])).collect();
        let correct = include!("test_fixtures/vw_simplified.rs");
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e[0], e[1])).collect();
        let simplified = LineString::from(points_ls).simplifyvw_preserve(&0.0005);
        assert_eq!(simplified, LineString::from(correct_ls));
    }
    #[test]
    fn visvalingam_test_empty_linestring() {
        let vec: Vec<[f32; 2]> = Vec::new();
        let compare = Vec::new();
        let simplified = visvalingam(&LineString::from(vec), &1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn visvalingam_test_two_point_linestring() {
        let mut vec = Vec::new();
        vec.push(Point::new(0.0, 0.0));
        vec.push(Point::new(27.8, 0.1));
        let mut compare = Vec::new();
        compare.push(Coordinate::from((0.0, 0.0)));
        compare.push(Coordinate::from((27.8, 0.1)));
        let simplified = visvalingam(&LineString::from(vec), &1.0);
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

        let mline = MultiLineString(vec![LineString::from(points_ls)]);
        assert_eq!(
            mline.simplifyvw(&30.),
            MultiLineString(vec![LineString::from(correct_ls)])
        );
    }

    #[test]
    fn polygon() {
        let poly = polygon![
            (x: 0., y: 0.),
            (x: 0., y: 10.),
            (x: 5., y: 11.),
            (x: 10., y: 10.),
            (x: 10., y: 0.),
            (x: 0., y: 0.),
        ];

        let poly2 = poly.simplifyvw(&10.);

        assert_eq!(
            poly2,
            polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.),
            ],
        );
    }

    #[test]
    fn multipolygon() {
        let mpoly = MultiPolygon(vec![Polygon::new(
            LineString::from(vec![
                (0., 0.),
                (0., 10.),
                (5., 11.),
                (10., 10.),
                (10., 0.),
                (0., 0.),
            ]),
            vec![],
        )]);

        let mpoly2 = mpoly.simplifyvw(&10.);

        assert_eq!(
            mpoly2,
            MultiPolygon(vec![Polygon::new(
                LineString::from(vec![(0., 0.), (0., 10.), (10., 10.), (10., 0.), (0., 0.)]),
                vec![],
            )])
        );
    }
}
