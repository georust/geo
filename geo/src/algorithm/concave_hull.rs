use crate::convex_hull::qhull;
use crate::utils::partial_min;
use crate::{
    Contains, Coord, Distance, Euclidean, GeoFloat, Intersects, Length, Line, LineString,
    MultiLineString, MultiPoint, MultiPolygon, Polygon, Triangle, coord, point,
};
use rstar::{AABB, Envelope, ParentNode, RTree, RTreeNode, RTreeNum};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, VecDeque},
};

/// Returns a polygon which covers a geometry. Unlike convex hulls, which also cover
/// their geometry, a concave hull does so while trying to further minimize its area by
/// constructing edges such that the exterior of the polygon incorporates points that would
/// be interior points in a convex hull.
///
/// This implementation is a port of Volodymyr Agafonkin's <https://github.com/mapbox/concaveman> which is based on ideas from
/// the paper [A New Concave Hull Algorithm and Concaveness Measure for n-dimensional Datasets, 2012](https://jise.iis.sinica.edu.tw/JISESearch/fullText?pId=245&code=5A9B97538372AA1).
///
/// # Arguments
/// * `concave_hull_options` - Optional configuration for the concave hull algorithm:
///   - `concavity` - A relative measure of how concave the hull should be. Lower values result in a more
///     concave hull. Infinity would result in a convex hull. 2.0 results in a relatively detailed shape. (default: 2.0)
///   - `length_threshold` - The minimum length of constituent hull edges. Edges shorter than this will not be
///     drilled down any further. Set to 0.0 for no threshold. (default: 0.0)
///
/// # Returns
/// * A `Polygon` representing a concave hull of the geometry.
///
/// # Algorithm
///
/// The algorithm works as follows:
/// 1. Start with the convex hull of all input points as the initial boundary
/// 2. For each edge of the hull boundary:
///    - If the edge length exceeds the `length_threshold`, attempt to "drill inward"
///    - Search for the closest interior point within `max_length = edge_length / concavity` from edge
///    - Verify the candidate is closer to this edge than adjacent hull edges
///    - Verify that connecting to this point won't cause intersections with existing hull edges
///    - Continue searching until a valid candidate is found or no more points are within the `max_length`
///    - Verify that adding this point won't cause any previously checked interior points to be excluded from the hull and if one is excluded use that point as the candidate
/// 3. If a valid candidate is found:
///    - Create two new edges: start→candidate and candidate→end
///    - Verify at least one of the new edges is less than `max_length`
///    - Replace the original edge with the two new edges
///    - Remove the candidate point from further selection
///    - Add the new edges to the boundary and processing queue for potential further drilling
/// 4. Repeat until no more edges can be drilled
///
/// # Examples
/// ```
/// use geo::{polygon, MultiPoint};
/// use geo::ConcaveHull;
///
/// // a collection of points
/// let points: MultiPoint<_> = vec![
///     (0.0, 0.0),
///     (3.0, 0.0),
///     (1.0, 2.0),
///     (0.0, 4.0),
/// ].into();
///
/// let correct_hull = polygon![
///     (x: 3.0, y: 0.0),
///     (x: 1.0, y: 2.0),
///     (x: 0.0, y: 4.0),
///     (x: 0.0, y: 0.0),
///     (x: 3.0, y: 0.0),
/// ];
///
/// let hull = points.concave_hull();
/// assert_eq!(hull, correct_hull);
///
/// ```
/// `ConcaveHull` can also be used with custom options.
/// ```
/// use geo::{polygon, ConcaveHull, MultiPoint};
/// use geo::concave_hull::ConcaveHullOptions;
///
/// // a collection of points
/// let points: MultiPoint<_> = vec![
///     (0.0, 0.0),
///     (2.0, 0.0),
///     (1.5, 1.0),
///     (2.0, 2.0),
///     (0.0, 2.0),
/// ].into();
///
/// let correct_hull = polygon![
///     (x: 2.0, y: 0.0),
///     (x: 1.5, y: 1.0),
///     (x: 2.0, y: 2.0),
///     (x: 0.0, y: 2.0),
///     (x: 0.0, y: 0.0),
///     (x: 2.0, y: 0.0),
/// ];
///
/// let hull = points.concave_hull_with_options(ConcaveHullOptions {
///     concavity: 1.0,
///     length_threshold: 0.0,
/// });
/// assert_eq!(hull, correct_hull);
/// ```
pub trait ConcaveHull {
    type Scalar: GeoFloat + RTreeNum;
    /// Create a concave hull around the geometry set.
    ///
    /// See the [module-level documentation](self) for details on the algorithm.
    fn concave_hull(&self) -> Polygon<Self::Scalar> {
        self.concave_hull_with_options(ConcaveHullOptions::default())
    }

    /// Create a concave hull around the geometry set with specified options.
    ///
    /// See the [module-level documentation](self) for details on the algorithm and parameters.
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<Self::Scalar>;
}

impl<T> ConcaveHull for MultiPoint<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.iter().map(|point| point.0).collect();
        concave_hull_with_options(&mut coords, concave_hull_options)
    }
}

impl<T> ConcaveHull for Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the interior of the original geometry boundaries.
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.exterior().0.clone();
        concave_hull_with_options(&mut coords, concave_hull_options)
    }
}

impl<T> ConcaveHull for MultiPolygon<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the interior of the original geometry boundaries.
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = Vec::new();
        for polygon in self.0.iter() {
            coords.extend(polygon.exterior().0.iter().skip(1));
        }
        concave_hull_with_options(&mut coords, concave_hull_options)
    }
}

impl<T> ConcaveHull for LineString<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the original geometry boundaries.
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.0.clone();
        concave_hull_with_options(&mut coords, concave_hull_options)
    }
}

impl<T> ConcaveHull for MultiLineString<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the original geometry boundaries.
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.iter().flat_map(|elem| elem.0.clone()).collect();
        concave_hull_with_options(&mut coords, concave_hull_options)
    }
}

impl<T> ConcaveHull for Vec<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.clone();
        concave_hull_with_options(&mut coords, concave_hull_options)
    }
}

impl<T> ConcaveHull for [Coord<T>]
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull_with_options(
        &self,
        concave_hull_options: ConcaveHullOptions<Self::Scalar>,
    ) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.to_vec();
        concave_hull_with_options(&mut coords, concave_hull_options)
    }
}

/// The options for creating a concave hull composed of `concavity` and `length_threshold`. See arguments in [ConcaveHull] for full details.
pub struct ConcaveHullOptions<T>
where
    T: GeoFloat + RTreeNum,
{
    pub concavity: T,
    pub length_threshold: T,
}

impl<T> Default for ConcaveHullOptions<T>
where
    T: GeoFloat + RTreeNum,
{
    fn default() -> Self {
        Self {
            concavity: T::from(2.0).unwrap(),
            length_threshold: T::zero(),
        }
    }
}

impl<T> ConcaveHullOptions<T>
where
    T: GeoFloat + RTreeNum,
{
    pub fn concavity(mut self, concavity: T) -> Self {
        self.concavity = concavity;
        self
    }

    pub fn length_threshold(mut self, length_threshold: T) -> Self {
        self.length_threshold = length_threshold;
        self
    }
}

enum RTreeNodeRef<'a, T>
where
    T: GeoFloat + RTreeNum,
{
    Parent(&'a ParentNode<Coord<T>>),
    Leaf(&'a Coord<T>),
}

struct NodeQueueItem<'a, T>
where
    T: GeoFloat + RTreeNum,
{
    tree_node: RTreeNodeRef<'a, T>,
    distance: T,
}

impl<'a, T> Ord for NodeQueueItem<'a, T>
where
    T: GeoFloat + RTreeNum,
{
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.total_cmp(&self.distance)
    }
}
impl<'a, T> PartialOrd for NodeQueueItem<'a, T>
where
    T: GeoFloat + RTreeNum,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<'a, T> PartialEq for NodeQueueItem<'a, T>
where
    T: GeoFloat + RTreeNum,
{
    fn eq(&self, other: &Self) -> bool {
        self.distance.total_cmp(&other.distance) == Ordering::Equal
    }
}
impl<'a, T> Eq for NodeQueueItem<'a, T> where T: GeoFloat + RTreeNum {}

#[derive(Clone)]
struct CurrentHullEdge<T: GeoFloat + RTreeNum> {
    line: Line<T>,
    i: usize,
    prev_i: usize,
    next_i: usize,
}

fn line_to_bbox_distance<T>(line: &Line<T>, bbox: &AABB<Coord<T>>) -> T
where
    T: GeoFloat + RTreeNum,
{
    // Calculate the euclidean distance from a line to a bounding box.
    // This function is equivalent to `Euclidean.distance` between a `Rect` and a `Line`,
    // but is optimized for the R-tree depth-first search used here.
    // Since lines are likely to be intersecting or contained within the bounding box, resulting
    // in a distance of zero, calculate each seperately and return early if zero is found.

    // If either line endpoint is contained within the bbox, distance is zero
    if bbox.contains_point(&line.start) || bbox.contains_point(&line.end) {
        return T::zero();
    }

    // If any distances are zero, then return as no further distance calculations needed
    let c1 = coord! {x: bbox.lower().x, y: bbox.lower().y};
    let c2 = coord! {x: bbox.lower().x, y: bbox.upper().y};
    let c3 = coord! {x: bbox.upper().x, y: bbox.upper().y};
    let c4 = coord! {x: bbox.upper().x, y: bbox.lower().y};
    let d1 = Euclidean.distance(line, &Line::new(c1, c4));
    if d1 == T::zero() {
        return d1;
    }
    let d2 = Euclidean.distance(line, &Line::new(c1, c2));
    if d2 == T::zero() {
        return d2;
    }
    let d3 = Euclidean.distance(line, &Line::new(c4, c3));
    if d3 == T::zero() {
        return d3;
    }
    let d4 = Euclidean.distance(line, &Line::new(c2, c3));
    if d4 == T::zero() {
        return d4;
    }

    // If the line is contained within or intersecting the bounding box, return the minimum distance
    partial_min(partial_min(d1, d2), partial_min(d3, d4))
}

fn no_hull_intersections<T>(line: &Line<T>, current_hull_tree: &RTree<Line<T>>) -> bool
where
    T: GeoFloat + RTreeNum,
{
    // Check if the line intersects with any existing hull line
    // Hull lines which share an endpoint with the line are skipped

    // Create bounding box for the line
    let min_x = T::min(line.start.x, line.end.x);
    let max_x = T::max(line.start.x, line.end.x);
    let min_y = T::min(line.start.y, line.end.y);
    let max_y = T::max(line.start.y, line.end.y);
    let bbox = AABB::from_corners(point!([min_x, min_y]), point!([max_x, max_y]));

    // Iterate over all lines in the hull which intersect with the bounding box
    let hull_lines = current_hull_tree.locate_in_envelope_intersecting(&bbox);
    for hull_line in hull_lines {
        // Skip lines which share an endpoint
        if hull_line.start == line.start
            || hull_line.start == line.end
            || hull_line.end == line.start
            || hull_line.end == line.end
        {
            continue;
        }
        if line.intersects(hull_line) {
            return false;
        }
    }
    true
}

fn find_excluded_point<T>(
    start_line: Line<T>,
    end_line: Line<T>,
    previously_checked_points: &[Coord<T>],
) -> Option<Coord<T>>
where
    T: GeoFloat,
{
    // Check previously checked interior points to see if any would lie outside the hull if the new lines were added.
    // If so, return that point as the candidate to ensure all interior points remain within the hull.

    // Create a triangle which represents the area which would be excluded from the hull
    let triangle = Triangle::new(start_line.start, start_line.end, end_line.end);
    for point in previously_checked_points {
        // If the point is contained within the triangle, it would be outside the hull and so return it as the candidate
        if triangle.contains(point) {
            return Some(*point);
        }
    }
    None
}

fn find_candidate<T>(
    hull_edge: &CurrentHullEdge<T>,
    max_length: &T,
    current_hull_edges: &[CurrentHullEdge<T>],
    interior_points_tree: &RTree<Coord<T>>,
    current_hull_tree: &RTree<Line<T>>,
) -> Option<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    let line = hull_edge.line;

    // Initialize priority queue with R-tree root node
    let mut queue: BinaryHeap<NodeQueueItem<T>> = BinaryHeap::new();
    queue.push(NodeQueueItem {
        tree_node: RTreeNodeRef::Parent(interior_points_tree.root()),
        distance: T::zero(),
    });

    // Keep track of nearest interior points which failed checks
    let mut previously_checked_points: Vec<Coord<T>> = Vec::new();

    // Perform depth-first search through the R-tree
    while let Some(node) = queue.pop() {
        match node.tree_node {
            RTreeNodeRef::Parent(parent) => {
                // Add the children of a parent node to the queue if they are within the max distance
                for child in parent.children() {
                    match child {
                        RTreeNode::Parent(p) => {
                            let distance = line_to_bbox_distance(&line, &p.envelope());
                            if distance <= *max_length {
                                queue.push(NodeQueueItem {
                                    tree_node: RTreeNodeRef::Parent(p),
                                    distance,
                                });
                            }
                        }
                        RTreeNode::Leaf(l) => {
                            let distance = Euclidean.distance(*l, &line);
                            if distance <= *max_length {
                                queue.push(NodeQueueItem {
                                    tree_node: RTreeNodeRef::Leaf(l),
                                    distance,
                                });
                            }
                        }
                    }
                }
            }
            RTreeNodeRef::Leaf(leaf) => {
                // Check if candidate point is further from adjacent hull lines
                if node.distance
                    >= Euclidean.distance(*leaf, &current_hull_edges[hull_edge.prev_i].line)
                    || node.distance
                        >= Euclidean.distance(*leaf, &current_hull_edges[hull_edge.next_i].line)
                {
                    previously_checked_points.push(*leaf);
                    continue;
                }

                let start_line = Line::new(line.start, *leaf);
                let end_line = Line::new(*leaf, line.end);

                // Check if using candidate point would cause intersections with hull lines
                if no_hull_intersections(&start_line, current_hull_tree)
                    && no_hull_intersections(&end_line, current_hull_tree)
                {
                    // Check if any of the previously checked interior points would lie outside the hull if the new lines
                    // were added and use that point as the candidate if so
                    if let Some(point) =
                        find_excluded_point(start_line, end_line, &previously_checked_points)
                    {
                        return Some(point);
                    }
                    return Some(*leaf);
                }
                previously_checked_points.push(*leaf);
            }
        }
    }
    None
}

fn order_concave_hull<T>(current_hull_edges: Vec<CurrentHullEdge<T>>) -> LineString<T>
where
    T: GeoFloat,
{
    // Order the constituent concave hull edges and return as a `LineString`
    let mut ordered_coords: Vec<Coord<T>> = Vec::with_capacity(current_hull_edges.len() + 1);
    let mut current_i = 0;
    ordered_coords.push(current_hull_edges[current_i].line.start);

    for _ in 0..current_hull_edges.len() {
        let next_i = current_hull_edges[current_i].next_i;
        let line = current_hull_edges[current_i].line;
        ordered_coords.push(line.end);
        current_i = next_i;
    }
    LineString::from(ordered_coords)
}

fn remove_interior_point<T>(coord: &Coord<T>, tree: &mut RTree<Coord<T>>)
where
    T: GeoFloat + RTreeNum,
{
    // Remove all instances of the point from the R-tree
    let n = tree.nearest_neighbors(coord).len();
    for _ in 0..n {
        tree.remove(coord);
    }
}

fn concave_hull_with_options<T>(
    coords: &mut [Coord<T>],
    concave_hull_options: ConcaveHullOptions<T>,
) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    debug_assert!(concave_hull_options.concavity >= T::zero());
    // Ensure concavity is non-negative
    let concavity: T = T::max(T::zero(), concave_hull_options.concavity);

    // Compute initial convex hull
    let convex_hull = qhull::quick_hull(coords);

    // Return convex hull if less than 4 points
    if coords.len() < 4 {
        return Polygon::new(convex_hull, vec![]);
    }

    // Build R-trees for interior points and hull lines
    let mut interior_points_tree: RTree<Coord<T>> = RTree::bulk_load(coords.to_owned());
    let mut current_hull_tree: RTree<Line<T>> = RTree::bulk_load(convex_hull.lines().collect());

    let mut edge_queue: VecDeque<CurrentHullEdge<T>> = VecDeque::new();
    let mut current_hull_edges: Vec<CurrentHullEdge<T>> = Vec::new();

    // Populate edge queue and current hull edges with convex hull edges
    let hull_length = convex_hull.lines().len();
    for (i, line) in convex_hull.lines().enumerate() {
        let edge = CurrentHullEdge {
            line,
            i,
            prev_i: if i == 0 { hull_length - 1 } else { i - 1 },
            next_i: if i == hull_length - 1 { 0 } else { i + 1 },
        };
        current_hull_edges.push(edge.clone());
        edge_queue.push_back(edge);

        // Remove hull points from interior points
        if i == 0 {
            remove_interior_point(&line.start, &mut interior_points_tree);
        }
        remove_interior_point(&line.end, &mut interior_points_tree);
    }

    while let Some(hull_edge) = edge_queue.pop_front() {
        let line = hull_edge.line;
        let length = Euclidean.length(&line);

        // Only consider drilling down if line length exceeds threshold
        if length > concave_hull_options.length_threshold {
            // Calculate maximum length for new hull edges
            let max_length = length / concavity;

            if let Some(candidate_point) = find_candidate(
                &hull_edge,
                &max_length,
                &current_hull_edges,
                &interior_points_tree,
                &current_hull_tree,
            ) {
                // Create new hull lines from start→candidate and candidate→end
                let start_line = Line::new(line.start, candidate_point);
                let end_line = Line::new(candidate_point, line.end);

                // Verify at least one of the new edges is less than max_length
                if partial_min(Euclidean.length(&start_line), Euclidean.length(&end_line))
                    < max_length
                {
                    // Remove candidate point from interior points
                    remove_interior_point(&candidate_point, &mut interior_points_tree);

                    // Update current hull tree
                    current_hull_tree.remove(&line);
                    current_hull_tree.insert(start_line);
                    current_hull_tree.insert(end_line);

                    // Set end edges' index as the length of current hull
                    let end_edge_i = current_hull_edges.len();

                    // Set new hull edges with indexes of adjacent edges
                    let start_hull_edge = CurrentHullEdge {
                        line: start_line,
                        i: hull_edge.i,
                        prev_i: hull_edge.prev_i,
                        next_i: end_edge_i,
                    };
                    let end_hull_edge = CurrentHullEdge {
                        line: end_line,
                        i: end_edge_i,
                        prev_i: hull_edge.i,
                        next_i: hull_edge.next_i,
                    };

                    // Replace the current edge with the new start edge
                    current_hull_edges[hull_edge.i] = start_hull_edge.clone();

                    // Push new end edge to current hull edges
                    current_hull_edges.push(end_hull_edge.clone());

                    // Push both new edges to queue
                    edge_queue.push_back(start_hull_edge);
                    edge_queue.push_back(end_hull_edge);

                    continue;
                }
            }
        }
    }
    Polygon::new(order_concave_hull(current_hull_edges), vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, line_string, polygon};

    #[test]
    fn test_concavity() {
        let coords: Vec<Coord<f64>> = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 2.0, y: 0.0 },
            coord! { x: 1.5, y: 1.0 },
            coord! { x: 2.0, y: 2.0 },
            coord! { x: 0.0, y: 2.0 },
        ];
        let hull_1 = coords.concave_hull_with_options(ConcaveHullOptions::default().concavity(1.0));
        assert_eq!(hull_1.exterior().0.len(), 6);

        let hull_2 = coords.concave_hull();
        assert_eq!(hull_2.exterior().0.len(), 5);
    }

    #[test]
    fn test_length_threshold() {
        let coords: Vec<Coord<f64>> = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 2.0, y: 0.0 },
            coord! { x: 1.5, y: 1.0 },
            coord! { x: 2.0, y: 2.0 },
            coord! { x: 0.0, y: 2.0 },
        ];
        let hull = coords.concave_hull_with_options(ConcaveHullOptions {
            concavity: 1.0,
            length_threshold: 3.0,
        });
        assert_eq!(hull.exterior().0.len(), 5);
    }

    #[test]
    fn test_empty_coords() {
        let coords: Vec<Coord<f64>> = vec![];
        let hull = coords.concave_hull();
        assert!(hull.exterior().0.is_empty());
    }

    #[test]
    fn test_norway_mainland() {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let correct_hull: LineString = geo_test_fixtures::norway_concave_hull::<f64>();
        let hull = norway.concave_hull();
        assert_eq!(hull.exterior(), &correct_hull);
    }

    #[test]
    fn test_polygon() {
        let poly = polygon![
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 0.0),
            (x: 1.5, y: 1.0),
            (x: 2.0, y: 2.0),
            (x: 0.0, y: 2.0),
        ];
        let correct_hull = polygon![
            (x: 2.0, y: 0.0),
            (x: 1.5, y: 1.0),
            (x: 2.0, y: 2.0),
            (x: 0.0, y: 2.0),
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 0.0),
        ];
        let hull = poly.concave_hull_with_options(ConcaveHullOptions::default().concavity(1.0));
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_multipolygon() {
        let mp: MultiPolygon<f64> = vec![
            polygon![
                (x: 0.0, y: 0.0),
                (x: 0.0, y: 2.0),
                (x: 2.0, y: 2.0),
                (x: 2.0, y: 0.0),
                (x: 0.0, y: 0.0),
            ],
            polygon![
                (x: 3.0, y: 0.0),
                (x: 3.0, y: 5.0),
                (x: 4.0, y: 5.0),
                (x: 4.0, y: 0.0),
                (x: 3.0, y: 0.0),
            ],
        ]
        .into();
        let hull = mp.concave_hull();
        let correct_hull = polygon![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 5.0),
            (x: 3.0, y: 5.0),
            (x: 2.0, y: 2.0),
            (x: 0.0, y: 2.0),
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 0.0),
            (x: 3.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_linestring() {
        let linestring = line_string![
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 1.0),
            (x: 3.0, y: 2.0)
        ];
        let hull = linestring.concave_hull();
        let correct_hull = polygon![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 2.0),
            (x: 3.0, y: 1.0),
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_multilinestring() {
        let v1 = line_string![
                (x: 0.0, y: 0.0),
                (x: 4.0, y: 0.0)
        ];
        let v2 = line_string![
                (x: 4.0, y: 4.0),
                (x: 3.0, y: 1.0),
                (x: 3.0, y: 2.0)
        ];
        let mls = MultiLineString::new(vec![v1, v2]);
        let correct_hull = polygon![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 2.0),
            (x: 3.0, y: 1.0),
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];
        let hull = mls.concave_hull();
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_consecutive_drilling() {
        let coords = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 3.0, y: 1.0 },
            coord! { x: 3.0, y: 2.0 },
        ];
        let correct_hull = polygon![
            (x: 4.0, y: 0.0),
            (x: 4.0, y: 4.0),
            (x: 3.0, y: 2.0),
            (x: 3.0, y: 1.0),
            (x: 0.0, y: 0.0),
            (x: 4.0, y: 0.0),
        ];
        let hull = coords.concave_hull();
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_only_collinear_points() {
        let linestring: LineString<f64> = line_string![
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 2.0),
            (x: 6.0, y: 6.0),
        ];
        let correct_hull = polygon![
            (x: 0.0, y: 0.0),
            (x: 6.0, y: 6.0),
            (x: 0.0, y: 0.0),
        ];
        let hull = linestring.concave_hull();
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_identical_points() {
        let coords = vec![
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 1.0, y: 1.0 },
            coord! { x: 1.0, y: 1.0 },
        ];
        let correct_hull = polygon![
            (x: 1.0, y: 1.0),
            (x: 1.0, y: 1.0),
        ];
        let hull = coords.concave_hull();
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_four_drills() {
        let coords = vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 2.0, y: 1.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 3.0, y: 2.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 2.0, y: 3.0 },
            coord! { x: 0.0, y: 4.0 },
            coord! { x: 1.0, y: 2.0 },
        ];
        let correct_hull = polygon![
            (x: 4.0, y: 0.0),
            (x: 3.0, y: 2.0),
            (x: 4.0, y: 4.0),
            (x: 2.0, y: 3.0),
            (x: 0.0, y: 4.0),
            (x: 1.0, y: 2.0),
            (x: 0.0, y: 0.0),
            (x: 2.0, y: 1.0),
            (x: 4.0, y: 0.0),
        ];
        let hull = coords.concave_hull_with_options(ConcaveHullOptions::default().concavity(0.0));
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_all_points_in_hull() {
        let coords = vec![
            coord! { x: 8.206, y: 7.705 },
            coord! { x: 6.929, y: 6.919 },
            coord! { x: 8.036, y: 8.394 },
            coord! { x: 7.376, y: 1.512 },
            coord! { x: 0.487, y: 1.839 },
            coord! { x: 9.317, y: 8.696 },
        ];
        let hull = coords.concave_hull();
        for coord in coords.iter() {
            assert!(hull.intersects(coord));
        }
    }
}
