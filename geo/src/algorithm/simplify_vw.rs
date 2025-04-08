use crate::prelude::*;
use crate::{
    Coord, CoordFloat, GeoFloat, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon,
    Triangle,
};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

use rstar::primitives::CachedEnvelope;
use rstar::{RTree, RTreeNum};

/// Store triangle information. Area is used for ranking in the priority queue and determining removal
#[derive(Debug)]
struct VScore<T>
where
    T: CoordFloat,
{
    left: usize,
    /// The current [Point] index in the original [LineString]: The candidate for removal
    current: usize,
    right: usize,
    area: T,
    // `visvalingam_preserve` uses `intersector`, `visvalingam` does not, so it's always false
    intersector: bool,
}

// These impls give us a min-heap
impl<T> Ord for VScore<T>
where
    T: CoordFloat,
{
    fn cmp(&self, other: &VScore<T>) -> Ordering {
        other.area.partial_cmp(&self.area).unwrap()
    }
}

impl<T> PartialOrd for VScore<T>
where
    T: CoordFloat,
{
    fn partial_cmp(&self, other: &VScore<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Eq for VScore<T> where T: CoordFloat {}

impl<T> PartialEq for VScore<T>
where
    T: CoordFloat,
{
    fn eq(&self, other: &VScore<T>) -> bool
    where
        T: CoordFloat,
    {
        self.area == other.area
    }
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
fn visvalingam_indices<T>(orig: &LineString<T>, epsilon: T) -> Vec<usize>
where
    T: CoordFloat,
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
            intersector: false,
        })
        .collect::<BinaryHeap<VScore<T>>>();
    // While there are still points for which the associated triangle
    // has an area below the epsilon
    while let Some(smallest) = pq.pop() {
        if smallest.area > epsilon {
            // no need to keep trying: the min-heap ensures that we process triangles in order
            // so if we see one that exceeds the tolerance we're done: everything else is too big
            break;
        }
        //  This triangle's area is below epsilon: the associated point is a candidate for removal
        let (left, right) = adjacent[smallest.current];
        // A point in this triangle has been removed since this VScore
        // was created, so skip it
        if left != smallest.left as i32 || right != smallest.right as i32 {
            continue;
        }
        // We've got a valid triangle, and its area is smaller than epsilon, so
        // remove it from the simulated "linked list"
        let (ll, _) = adjacent[left as usize];
        let (_, rr) = adjacent[right as usize];
        adjacent[left as usize] = (ll, right);
        adjacent[right as usize] = (left, rr);
        adjacent[smallest.current] = (0, 0);

        // Recompute the adjacent triangle(s), using left and right adjacent points
        // this may add new triangles to the heap
        recompute_triangles(&smallest, orig, &mut pq, ll, left, right, rr, max, epsilon);
    }
    // Filter out the points that have been deleted, returning remaining point indices
    orig.0
        .iter()
        .enumerate()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(tup.0) } else { None })
        .collect::<Vec<usize>>()
}

/// Recompute adjacent triangle(s) using left and right adjacent points, and push onto heap
///
/// This is used for both standard and topology-preserving variants.
#[allow(clippy::too_many_arguments)]
fn recompute_triangles<T>(
    smallest: &VScore<T>,
    orig: &LineString<T>,
    pq: &mut BinaryHeap<VScore<T>>,
    ll: i32,
    left: i32,
    right: i32,
    rr: i32,
    max: usize,
    epsilon: T,
) where
    T: CoordFloat,
{
    let choices = [(ll, left, right), (left, right, rr)];
    for &(ai, current_point, bi) in &choices {
        if ai as usize >= max || bi as usize >= max {
            // Out of bounds, i.e. we're on one edge
            continue;
        }
        let area = Triangle::new(
            orig.0[ai as usize],
            orig.0[current_point as usize],
            orig.0[bi as usize],
        )
        .unsigned_area();

        // This logic only applies to VW-Preserve
        // smallest.current's removal causes a self-intersection, and this point precedes it
        // we ensure it gets removed next by demoting its area to negative epsilon
        // we check that current_point is less than smallest.current because
        // if it's larger the point in question comes AFTER smallest.current: we only want to remove
        // the point that comes BEFORE smallest.current
        let area = if smallest.intersector && (current_point as usize) < smallest.current {
            -epsilon
        } else {
            area
        };

        let v = VScore {
            area,
            current: current_point as usize,
            left: ai as usize,
            right: bi as usize,
            intersector: false,
        };
        pq.push(v)
    }
}

// Wrapper for visvalingam_indices, mapping indices back to points
fn visvalingam<T>(orig: &LineString<T>, epsilon: T) -> Vec<Coord<T>>
where
    T: CoordFloat,
{
    // Epsilon must be greater than zero for any meaningful simplification to happen
    if epsilon <= T::zero() {
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

// Wrap the actual VW function so the R* Tree can be shared.
// this ensures that shell and rings have access to all segments, so
// intersections between outer and inner rings are detected
//
// Constants:
//
// * `INITIAL_MIN`
//   * If we ever have fewer than these, stop immediately
// * `MIN_POINTS`
//   * If we detect a self-intersection before point removal, and we only have `MIN_POINTS` left,
//     stop: since a self-intersection causes removal of the spatially previous point, THAT could
//     lead to a further self-intersection without the possibility of removing more points,
//     potentially leaving the geometry in an invalid state.
fn vwp_wrapper<T, const INITIAL_MIN: usize, const MIN_POINTS: usize>(
    exterior: &LineString<T>,
    interiors: Option<&[LineString<T>]>,
    epsilon: T,
) -> Vec<Vec<Coord<T>>>
where
    T: GeoFloat + RTreeNum,
{
    let mut rings = vec![];
    // Populate R* tree with exterior and interior samples, if any
    let mut tree: RTree<CachedEnvelope<_>> = RTree::bulk_load(
        exterior
            .lines()
            .chain(
                interiors
                    .iter()
                    .flat_map(|ring| *ring)
                    .flat_map(|line_string| line_string.lines()),
            )
            .map(CachedEnvelope::new)
            .collect::<Vec<_>>(),
    );

    // Simplify shell
    rings.push(visvalingam_preserve::<T, INITIAL_MIN, MIN_POINTS>(
        exterior, epsilon, &mut tree,
    ));
    // Simplify interior rings, if any
    if let Some(interior_rings) = interiors {
        for ring in interior_rings {
            rings.push(visvalingam_preserve::<T, INITIAL_MIN, MIN_POINTS>(
                ring, epsilon, &mut tree,
            ))
        }
    }
    rings
}

/// Visvalingam-Whyatt with self-intersection detection to preserve topologies
/// this is a port of the technique at https://www.jasondavies.com/simplify/
//
// Constants:
//
// * `INITIAL_MIN`
//   * If we ever have fewer than these, stop immediately
// * `MIN_POINTS`
//   * If we detect a self-intersection before point removal, and we only have `MIN_POINTS` left,
//     stop: since a self-intersection causes removal of the spatially previous point, THAT could
//     lead to a further self-intersection without the possibility of removing more points,
//     potentially leaving the geometry in an invalid state.
fn visvalingam_preserve<T, const INITIAL_MIN: usize, const MIN_POINTS: usize>(
    orig: &LineString<T>,
    epsilon: T,
    tree: &mut RTree<CachedEnvelope<Line<T>>>,
) -> Vec<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    if orig.0.len() < 3 || epsilon <= T::zero() {
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
        .collect::<BinaryHeap<VScore<T>>>();

    // While there are still points for which the associated triangle
    // has an area below the epsilon
    while let Some(mut smallest) = pq.pop() {
        if smallest.area > epsilon {
            // No need to continue: we've already seen all the candidate triangles;
            // the min-heap guarantees it
            break;
        }
        if counter <= INITIAL_MIN {
            // we can't remove any more points no matter what
            break;
        }
        let (left, right) = adjacent[smallest.current];
        // A point in this triangle has been removed since this VScore
        // was created, so skip it
        if left != smallest.left as i32 || right != smallest.right as i32 {
            continue;
        }
        // if removal of this point causes a self-intersection, we also remove the previous point
        // that removal alters the geometry, removing the self-intersection
        // HOWEVER if we're within 1 point of the absolute minimum, we can't remove this point or the next
        // because we could then no longer form a valid geometry if removal of next also caused an intersection.
        // The simplification process is thus over.
        smallest.intersector = tree_intersect(tree, &smallest, &orig.0);
        if smallest.intersector && counter <= MIN_POINTS {
            break;
        }
        let (ll, _) = adjacent[left as usize];
        let (_, rr) = adjacent[right as usize];
        adjacent[left as usize] = (ll, right);
        adjacent[right as usize] = (left, rr);
        // We've got a valid triangle, and its area is smaller than the tolerance, so
        // remove it from the simulated "linked list"
        adjacent[smallest.current] = (0, 0);
        counter -= 1;
        // Remove stale segments from R* tree
        let left_point = Point::from(orig.0[left as usize]);
        let middle_point = Point::from(orig.0[smallest.current]);
        let right_point = Point::from(orig.0[right as usize]);

        let line_1 = CachedEnvelope::new(Line::new(left_point, middle_point));
        let line_2 = CachedEnvelope::new(Line::new(middle_point, right_point));
        assert!(tree.remove(&line_1).is_some());
        assert!(tree.remove(&line_2).is_some());

        // Restore continuous line segment
        tree.insert(CachedEnvelope::new(Line::new(left_point, right_point)));

        // Recompute the adjacent triangle(s), using left and right adjacent points
        // this may add new triangles to the heap
        recompute_triangles(&smallest, orig, &mut pq, ll, left, right, rr, max, epsilon);
    }
    // Filter out the points that have been deleted, returning remaining points
    orig.0
        .iter()
        .zip(adjacent.iter())
        .filter_map(|(tup, adj)| if *adj != (0, 0) { Some(*tup) } else { None })
        .collect()
}

/// Check whether the new candidate line segment intersects with any existing geometry line segments
///
/// In order to do this efficiently, the rtree is queried for any existing segments which fall within
/// the bounding box of the new triangle created by the candidate segment
fn tree_intersect<T>(
    tree: &RTree<CachedEnvelope<Line<T>>>,
    triangle: &VScore<T>,
    orig: &[Coord<T>],
) -> bool
where
    T: GeoFloat + RTreeNum,
{
    let new_segment_start = orig[triangle.left];
    let new_segment_end = orig[triangle.right];
    // created by candidate point removal
    let new_segment = CachedEnvelope::new(Line::new(
        Point::from(orig[triangle.left]),
        Point::from(orig[triangle.right]),
    ));
    let bounding_rect = Triangle::new(
        orig[triangle.left],
        orig[triangle.current],
        orig[triangle.right],
    )
    .bounding_rect();
    tree.locate_in_envelope_intersecting(&rstar::AABB::from_corners(
        bounding_rect.min().into(),
        bounding_rect.max().into(),
    ))
    .any(|candidate| {
        // line start point, end point
        let (candidate_start, candidate_end) = candidate.points();
        candidate_start.0 != new_segment_start
            && candidate_start.0 != new_segment_end
            && candidate_end.0 != new_segment_start
            && candidate_end.0 != new_segment_end
            && new_segment.intersects(&**candidate)
    })
}

/// Simplifies a geometry.
///
/// Polygons are simplified by running the algorithm on all their constituent rings. This may
/// result in invalid Polygons, and has no guarantee of preserving topology. Multi* objects are
/// simplified by simplifying all their constituent geometries individually.
///
/// An epsilon less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyVw<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm
    ///
    /// See [here](https://bost.ocks.org/mike/simplify/) for a graphical explanation
    ///
    /// # Note
    /// The tolerance used to remove a point is `epsilon`, in keeping with GEOS. JTS uses `epsilon ^ 2`.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::SimplifyVw;
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
    /// let simplified = line_string.simplify_vw(30.0);
    ///
    /// let expected = line_string![
    ///     (x: 5.0, y: 2.0),
    ///     (x: 7.0, y: 25.0),
    ///     (x: 10.0, y: 10.0),
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplify_vw(&self, epsilon: T) -> Self
    where
        T: CoordFloat;
}

/// Simplifies a geometry, returning the retained _indices_ of the output
///
/// This operation uses the Visvalingam-Whyatt algorithm,
/// and does **not** guarantee that the returned geometry is valid.
///
/// A larger `epsilon` means being more aggressive about removing points with less concern for
/// maintaining the existing shape. Specifically, when you consider whether to remove a point, you
/// can draw a triangle consisting of the candidate point and the points before and after it.
/// If the area of this triangle is less than `epsilon`, we will remove the point.
///
/// An `epsilon` less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyVwIdx<T, Epsilon = T> {
    /// Returns the simplified representation of a geometry, using the [Visvalingam-Whyatt](http://www.tandfonline.com/doi/abs/10.1179/000870493786962263) algorithm
    ///
    /// See [here](https://bost.ocks.org/mike/simplify/) for a graphical explanation
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::SimplifyVwIdx;
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
    /// let simplified = line_string.simplify_vw_idx(30.0);
    ///
    /// let expected = vec![
    ///     0_usize,
    ///     3_usize,
    ///     4_usize,
    /// ];
    ///
    /// assert_eq!(expected, simplified);
    /// ```
    fn simplify_vw_idx(&self, epsilon: T) -> Vec<usize>
    where
        T: CoordFloat;
}

/// Simplifies a geometry, attempting to preserve its topology by removing self-intersections
///
/// A larger `epsilon` means being more aggressive about removing points with less concern for
/// maintaining the existing shape. Specifically, when you consider whether to remove a point, you
/// can draw a triangle consisting of the candidate point and the points before and after it.
/// If the area of this triangle is less than `epsilon`, we will remove the point.
///
/// An `epsilon` less than or equal to zero will return an unaltered version of the geometry.
pub trait SimplifyVwPreserve<T, Epsilon = T> {
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
    /// # Notes
    ///
    /// - It is possible for the simplification algorithm to displace a Polygon's interior ring outside its shell.
    /// - The algorithm does **not** guarantee a valid output geometry, especially on smaller geometries.
    /// - If removal of a point causes a self-intersection, but the geometry only has `n + 1`
    ///   points remaining (3 for a `LineString`, 5 for a `Polygon`), the point is retained and the
    ///   simplification process ends. This is because there is no guarantee that removal of two points will remove
    ///   the intersection, but removal of further points would leave too few points to form a valid geometry.
    /// - The tolerance used to remove a point is `epsilon`, in keeping with GEOS. JTS uses `epsilon ^ 2`
    ///
    /// # Examples
    ///
    /// ```
    /// use approx::assert_relative_eq;
    /// use geo::SimplifyVwPreserve;
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
    /// let simplified = line_string.simplify_vw_preserve(668.6);
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
    /// assert_relative_eq!(expected, simplified, epsilon = 1e-6);
    /// ```
    fn simplify_vw_preserve(&self, epsilon: T) -> Self
    where
        T: CoordFloat + RTreeNum;
}

impl<T> SimplifyVwPreserve<T> for LineString<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: T) -> LineString<T> {
        let mut simplified = vwp_wrapper::<_, 2, 4>(self, None, epsilon);
        LineString::from(simplified.pop().unwrap())
    }
}

impl<T> SimplifyVwPreserve<T> for MultiLineString<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: T) -> MultiLineString<T> {
        MultiLineString::new(
            self.0
                .iter()
                .map(|l| l.simplify_vw_preserve(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVwPreserve<T> for Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: T) -> Polygon<T> {
        let mut simplified =
        // min_points was formerly 6, but that's too conservative for small polygons
            vwp_wrapper::<_, 4, 5>(self.exterior(), Some(self.interiors()), epsilon);
        let exterior = LineString::from(simplified.remove(0));
        let interiors = simplified.into_iter().map(LineString::from).collect();
        Polygon::new(exterior, interiors)
    }
}

impl<T> SimplifyVwPreserve<T> for MultiPolygon<T>
where
    T: GeoFloat + RTreeNum,
{
    fn simplify_vw_preserve(&self, epsilon: T) -> MultiPolygon<T> {
        MultiPolygon::new(
            self.0
                .iter()
                .map(|p| p.simplify_vw_preserve(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVw<T> for LineString<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: T) -> LineString<T> {
        LineString::from(visvalingam(self, epsilon))
    }
}

impl<T> SimplifyVwIdx<T> for LineString<T>
where
    T: CoordFloat,
{
    fn simplify_vw_idx(&self, epsilon: T) -> Vec<usize> {
        visvalingam_indices(self, epsilon)
    }
}

impl<T> SimplifyVw<T> for MultiLineString<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: T) -> MultiLineString<T> {
        MultiLineString::new(self.iter().map(|l| l.simplify_vw(epsilon)).collect())
    }
}

impl<T> SimplifyVw<T> for Polygon<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: T) -> Polygon<T> {
        Polygon::new(
            self.exterior().simplify_vw(epsilon),
            self.interiors()
                .iter()
                .map(|l| l.simplify_vw(epsilon))
                .collect(),
        )
    }
}

impl<T> SimplifyVw<T> for MultiPolygon<T>
where
    T: CoordFloat,
{
    fn simplify_vw(&self, epsilon: T) -> MultiPolygon<T> {
        MultiPolygon::new(self.iter().map(|p| p.simplify_vw(epsilon)).collect())
    }
}

#[cfg(test)]
mod test {
    use super::{visvalingam, vwp_wrapper, SimplifyVw, SimplifyVwPreserve};
    use crate::{
        line_string, polygon, Coord, LineString, MultiLineString, MultiPolygon, Point, Polygon,
    };

    // See https://github.com/georust/geo/issues/1049
    #[test]
    #[should_panic]
    fn vwp_bug() {
        let pol = polygon![
            (x: 1., y: 4.),
            (x: 3., y: 4.),
            (x: 1., y: 1.),
            (x: 7., y: 0.),
            (x: 1., y: 0.),
            (x: 0., y: 1.),
            (x: 1., y: 4.),
        ];
        let simplified = pol.simplify_vw_preserve(2.25);
        assert_eq!(
            simplified,
            polygon![
                (x: 1., y: 4.),
                (x: 3., y: 4.),
                (x: 1., y: 1.),
                (x: 7., y: 0.),
                (x: 1., y: 0.),
                (x: 1., y: 4.),
            ]
        );
    }

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

        let correct = [(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct.iter().map(|e| Coord::from((e.0, e.1))).collect();

        let simplified = visvalingam(&ls, 30.);
        assert_eq!(simplified, correct_ls);
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
        let simplified = vwp_wrapper::<_, 2, 4>(&ls, None, 668.6);
        // this is the correct, non-intersecting LineString
        let correct = [
            (10., 60.),
            (126., 31.),
            (280., 19.),
            (117., 48.),
            (300., 40.),
            (301., 10.),
        ];
        let correct_ls: Vec<_> = correct.iter().map(|e| Coord::from((e.0, e.1))).collect();
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
        let simplified = poly.simplify_vw_preserve(95.4);
        assert_relative_eq!(simplified.exterior(), &outer, epsilon = 1e-6);
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
        let simplified = poly.simplify_vw_preserve(95.4);
        assert_eq!(simplified.exterior(), &outer);
        assert_eq!(simplified.interiors()[0], correct_inner);
    }
    #[test]
    fn very_long_vwp_test() {
        // simplify an 8k-point LineString, eliminating self-intersections
        let points_ls = geo_test_fixtures::norway_main::<f64>();
        let simplified = vwp_wrapper::<_, 2, 4>(&points_ls, None, 0.0005);
        assert_eq!(simplified[0].len(), 3278);
    }

    #[test]
    fn visvalingam_test_long() {
        // simplify a longer LineString
        let points_ls = geo_test_fixtures::vw_orig::<f64>();
        let correct_ls = geo_test_fixtures::vw_simplified::<f64>();
        let simplified = visvalingam(&points_ls, 0.0005);
        assert_eq!(simplified, correct_ls.0);
    }
    #[test]
    fn visvalingam_preserve_test_long() {
        // simplify a longer LineString using the preserve variant
        let points_ls = geo_test_fixtures::vw_orig::<f64>();
        let correct_ls = geo_test_fixtures::vw_simplified::<f64>();
        let simplified = points_ls.simplify_vw_preserve(0.0005);
        assert_relative_eq!(simplified, correct_ls, epsilon = 1e-6);
    }
    #[test]
    fn visvalingam_test_empty_linestring() {
        let vec: Vec<[f32; 2]> = Vec::new();
        let compare = Vec::new();
        let simplified = visvalingam(&LineString::from(vec), 1.0);
        assert_eq!(simplified, compare);
    }
    #[test]
    fn visvalingam_test_two_point_linestring() {
        let vec = vec![Point::new(0.0, 0.0), Point::new(27.8, 0.1)];
        let compare = vec![Coord::from((0.0, 0.0)), Coord::from((27.8, 0.1))];
        let simplified = visvalingam(&LineString::from(vec), 1.0);
        assert_eq!(simplified, compare);
    }

    #[test]
    fn multilinestring() {
        // this is the PostGIS example
        let points = [
            (5.0, 2.0),
            (3.0, 8.0),
            (6.0, 20.0),
            (7.0, 25.0),
            (10.0, 10.0),
        ];
        let points_ls: Vec<_> = points.iter().map(|e| Point::new(e.0, e.1)).collect();

        let correct = [(5.0, 2.0), (7.0, 25.0), (10.0, 10.0)];
        let correct_ls: Vec<_> = correct.iter().map(|e| Point::new(e.0, e.1)).collect();

        let mline = MultiLineString::new(vec![LineString::from(points_ls)]);
        assert_relative_eq!(
            mline.simplify_vw(30.),
            MultiLineString::new(vec![LineString::from(correct_ls)]),
            epsilon = 1e-6
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

        let poly2 = poly.simplify_vw(10.);

        assert_relative_eq!(
            poly2,
            polygon![
                (x: 0., y: 0.),
                (x: 0., y: 10.),
                (x: 10., y: 10.),
                (x: 10., y: 0.),
                (x: 0., y: 0.),
            ],
            epsilon = 1e-6
        );
    }

    #[test]
    fn multipolygon() {
        let mpoly = MultiPolygon::new(vec![Polygon::new(
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

        let mpoly2 = mpoly.simplify_vw(10.);

        assert_relative_eq!(
            mpoly2,
            MultiPolygon::new(vec![Polygon::new(
                LineString::from(vec![(0., 0.), (0., 10.), (10., 10.), (10., 0.), (0., 0.)]),
                vec![],
            )]),
            epsilon = 1e-6
        );
    }
}
