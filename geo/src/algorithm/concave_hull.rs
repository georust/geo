use crate::convex_hull::qhull;
use crate::utils::partial_min;
use crate::{
    Coord, CoordNum, Distance, Euclidean, GeoFloat, Intersects, Length, Line, LineString,
    MultiLineString, MultiPoint, MultiPolygon, Polygon, coord, point,
};
use rstar::{AABB, Envelope, ParentNode, RTree, RTreeNode, RTreeNum};
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, VecDeque},
};

/// Returns a polygon which covers a geometry. Unlike convex hulls, which also cover
/// their geometry, a concave hull does so while trying to further minimize its area by
/// constructing edges such that the exterior of the polygon incorporates points that would
/// be interior points in a convex hull.
///
/// This implementation is a port of <https://github.com/mapbox/concaveman>
///
/// # Examples
/// ```
/// use geo::{line_string, MultiPoint};
/// use geo::ConcaveHull;
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
/// let hull = points.concave_hull(1.0, 0.0);
/// assert_eq!(hull, correct_hull);
/// ```
pub trait ConcaveHull {
    type Scalar: CoordNum;

    /// Create a new polygon as the concave hull of the geometry.
    ///
    /// # Arguments
    /// * `concavity` - A relative measure of how concave the hull should be. Higher values result in a more
    ///   concave hull. Inifinity would result in a convex hull. 2.0 results in a relatively detailed shape.
    ///
    /// * `length_threshold` - The minimum length of constituent hull lines. Lines shorter than this will not be
    ///   drilled down any further. Set to 0.0 for no threshold.
    ///
    /// # Returns
    /// * A Polygon representing the concave hull of the geometry.
    fn concave_hull(
        &self,
        concavity: Self::Scalar,
        length_threshold: Self::Scalar,
    ) -> Polygon<Self::Scalar>;
}

impl<T> ConcaveHull for MultiPoint<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(&self, concavity: Self::Scalar, length_threshold: Self::Scalar) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.iter().map(|point| point.0).collect();
        concave_hull(&mut coords, concavity, length_threshold)
    }
}

impl<T> ConcaveHull for Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the interior the original geometry boundaries.
    fn concave_hull(
        &self,
        concavity: Self::Scalar,
        length_threshold: Self::Scalar,
    ) -> Polygon<Self::Scalar> {
        let mut coords: Vec<Coord<T>> = self.exterior().0.clone();
        concave_hull(&mut coords, concavity, length_threshold)
    }
}

impl<T> ConcaveHull for MultiPolygon<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the interior the original geometry boundaries.
    fn concave_hull(
        &self,
        concavity: Self::Scalar,
        length_threshold: Self::Scalar,
    ) -> Polygon<Self::Scalar> {
        let mut coords: Vec<Coord<T>> = Vec::new();
        for polygon in self.0.iter() {
            coords.extend(polygon.exterior().0.iter().skip(1));
        }
        concave_hull(&mut coords, concavity, length_threshold)
    }
}

impl<T> ConcaveHull for LineString<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the original geometry boundaries.
    fn concave_hull(
        &self,
        concavity: Self::Scalar,
        length_threshold: Self::Scalar,
    ) -> Polygon<Self::Scalar> {
        concave_hull(&mut self.0.clone(), concavity, length_threshold)
    }
}

impl<T> ConcaveHull for MultiLineString<T>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    /// Note that the concave hull may intersect with the original geometry boundaries.
    fn concave_hull(&self, concavity: Self::Scalar, length_threshold: Self::Scalar) -> Polygon<T> {
        let mut coords: Vec<Coord<T>> = self.iter().flat_map(|elem| elem.0.clone()).collect();
        concave_hull(&mut coords, concavity, length_threshold)
    }
}

impl<T> ConcaveHull for Vec<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(
        &self,
        concavity: Self::Scalar,
        length_threshold: Self::Scalar,
    ) -> Polygon<Self::Scalar> {
        let mut coords: Vec<Coord<T>> = self.clone();
        concave_hull(&mut coords, concavity, length_threshold)
    }
}

impl<T> ConcaveHull for [Coord<T>]
where
    T: GeoFloat + RTreeNum,
{
    type Scalar = T;
    fn concave_hull(
        &self,
        concavity: Self::Scalar,
        length_threshold: Self::Scalar,
    ) -> Polygon<Self::Scalar> {
        let mut coords: Vec<Coord<T>> = self.to_vec();
        concave_hull(&mut coords, concavity, length_threshold)
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
        other.distance.partial_cmp(&self.distance).unwrap()
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
        self.distance == other.distance
    }
}
impl<'a, T> Eq for NodeQueueItem<'a, T> where T: GeoFloat + RTreeNum {}

struct LineQueueItem<T: GeoFloat + RTreeNum> {
    line: Line<T>,
    i: usize,
    prev_i: usize,
    next_i: usize,
}

fn line_to_bbox_distance<T>(line: &Line<T>, aabb: &AABB<Coord<T>>) -> T
where
    T: GeoFloat + RTreeNum,
{
    if aabb.contains_point(&line.start) || aabb.contains_point(&line.end) {
        return T::zero();
    }
    let c1 = coord! {x: aabb.lower().x, y: aabb.lower().y};
    let c2 = coord! {x: aabb.lower().x, y: aabb.upper().y};
    let c3 = coord! {x: aabb.upper().x, y: aabb.upper().y};
    let c4 = coord! {x: aabb.upper().x, y: aabb.lower().y};
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
    partial_min(partial_min(d1, d2), partial_min(d3, d4))
}

fn no_hull_intersections<T>(line: &Line<T>, current_hull_tree: &RTree<Line<T>>) -> bool
where
    T: GeoFloat + RTreeNum,
{
    let min_x = T::min(line.start.x, line.end.x);
    let max_x = T::max(line.start.x, line.end.x);
    let min_y = T::min(line.start.y, line.end.y);
    let max_y = T::max(line.start.y, line.end.y);
    let bbox = AABB::from_corners(point!([min_x, min_y]), point!([max_x, max_y]));
    let hull_lines = current_hull_tree.locate_in_envelope_intersecting(&bbox);
    for hull_line in hull_lines {
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

fn find_candidate<T>(
    line_queue_item: &LineQueueItem<T>,
    max_length: &T,
    hull_lines: &HashMap<usize, Line<T>>,
    interior_points_tree: &RTree<Coord<T>>,
    current_hull_tree: &RTree<Line<T>>,
) -> Option<Coord<T>>
where
    T: GeoFloat + RTreeNum,
{
    let line = hull_lines.get(&line_queue_item.i).unwrap();

    // Initialize priority queue with R-tree root node
    let mut queue: BinaryHeap<NodeQueueItem<T>> = BinaryHeap::new();
    queue.push(NodeQueueItem {
        tree_node: RTreeNodeRef::Parent(interior_points_tree.root()),
        distance: T::zero(),
    });

    // Perform depth first search through R-tree
    while let Some(node) = queue.pop() {
        match node.tree_node {
            RTreeNodeRef::Parent(parent) => {
                for child in parent.children() {
                    match child {
                        RTreeNode::Parent(p) => {
                            let envelope = p.envelope();
                            let distance = line_to_bbox_distance(line, &envelope);
                            if distance <= *max_length {
                                queue.push(NodeQueueItem {
                                    tree_node: RTreeNodeRef::Parent(p),
                                    distance,
                                });
                            }
                        }
                        RTreeNode::Leaf(l) => {
                            let distance = Euclidean.distance(*l, line);
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
                // Skip candidate points that are as close to adjacent hull lines
                if node.distance
                    >= Euclidean.distance(*leaf, hull_lines.get(&line_queue_item.prev_i).unwrap())
                    || node.distance
                        >= Euclidean
                            .distance(*leaf, hull_lines.get(&line_queue_item.next_i).unwrap())
                {
                    continue;
                }
                let start_line = Line::new(line.start, *leaf);
                let end_line = Line::new(*leaf, line.end);

                // Skip candidate point if it would cause intersections with hull lines
                if no_hull_intersections(&start_line, current_hull_tree)
                    && no_hull_intersections(&end_line, current_hull_tree)
                {
                    return Some(*leaf);
                }
            }
        }
    }
    None
}

fn order_concave_hull<T>(
    hull_order: HashMap<usize, usize>,
    hull_lines: HashMap<usize, Line<T>>,
) -> LineString<T>
where
    T: GeoFloat,
{
    let mut ordered_coords: Vec<Coord<T>> = vec![];
    let mut current_i = 0;
    ordered_coords.push(hull_lines.get(&current_i).unwrap().start);

    for _ in 0..hull_order.len() {
        let next_i = hull_order.get(&current_i).unwrap();
        let line = hull_lines.get(&current_i).unwrap();
        ordered_coords.push(line.end);
        current_i = *next_i;
    }
    LineString::from(ordered_coords)
}

fn remove_interior_point<T>(coord: &Coord<T>, tree: &mut RTree<Coord<T>>)
where
    T: GeoFloat + RTreeNum,
{
    // Remove all instances of the coordinate from the R-tree
    let n = tree.nearest_neighbors(coord).len();
    for _ in 0..n {
        tree.remove(coord);
    }
}

fn concave_hull<T>(coords: &mut [Coord<T>], concavity: T, length_threshold: T) -> Polygon<T>
where
    T: GeoFloat + RTreeNum,
{
    // Ensure concavity is non-negative
    let concavity: T = T::max(T::zero(), concavity);

    // Compute initial convex hull
    let hull = qhull::quick_hull(coords);

    // Return convex hull if less than 4 points
    if coords.len() < 4 {
        return Polygon::new(hull, vec![]);
    }

    // Build R-trees for interior points and hull lines
    let mut interior_points_tree: RTree<Coord<T>> = RTree::bulk_load(coords.to_owned());
    let mut current_hull_tree: RTree<Line<T>> = RTree::bulk_load(hull.lines().collect());

    let mut line_queue: VecDeque<LineQueueItem<T>> = VecDeque::new();
    let mut hull_lines: HashMap<usize, Line<T>> = HashMap::new();
    let mut hull_order: HashMap<usize, usize> = HashMap::new();

    // Populate line queue with initial hull lines
    let hull_length = hull.lines().len();
    for (i, line) in hull.lines().enumerate() {
        hull_lines.insert(i, line);
        line_queue.push_back(LineQueueItem {
            line,
            i,
            prev_i: if i == 0 { hull_length - 1 } else { i - 1 },
            next_i: if i == hull_length - 1 { 0 } else { i + 1 },
        });

        // Remove hull points from interior points
        if i == 0 {
            remove_interior_point(&line.start, &mut interior_points_tree);
        }
        remove_interior_point(&line.end, &mut interior_points_tree);
    }

    // Set current hull line index for new lines
    let mut current_i = hull_length;

    while let Some(line_queue_item) = line_queue.pop_front() {
        let line = line_queue_item.line;
        let length = Euclidean.length(&line);

        // Only consider drilling down if line length exceeds threshold
        if length > length_threshold {
            let max_length = length / concavity;

            if let Some(candidate_point) = find_candidate(
                &line_queue_item,
                &max_length,
                &hull_lines,
                &interior_points_tree,
                &current_hull_tree,
            ) {
                let start_line = Line::new(line.start, candidate_point);
                let end_line = Line::new(candidate_point, line.end);

                if partial_min(Euclidean.length(&start_line), Euclidean.length(&end_line))
                    < max_length
                {
                    // Remove candidate point from interior points
                    remove_interior_point(&candidate_point, &mut interior_points_tree);

                    // Update current hull tree
                    current_hull_tree.remove(&line);
                    current_hull_tree.insert(start_line);
                    current_hull_tree.insert(end_line);

                    // Update hull lines
                    hull_lines.insert(line_queue_item.i, start_line);
                    hull_lines.insert(current_i, end_line);

                    // Push new lines to queue
                    line_queue.push_back(LineQueueItem {
                        line: start_line,
                        i: line_queue_item.i,
                        prev_i: line_queue_item.prev_i,
                        next_i: current_i,
                    });
                    line_queue.push_back(LineQueueItem {
                        line: end_line,
                        i: current_i,
                        prev_i: line_queue_item.i,
                        next_i: line_queue_item.next_i,
                    });

                    // Increment current_i for next new potential hull line
                    current_i += 1;
                    continue;
                }
            }
        }

        hull_order.insert(line_queue_item.i, line_queue_item.next_i);
    }
    Polygon::new(order_concave_hull(hull_order, hull_lines), vec![])
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
        let hull_1 = coords.concave_hull(1.0, 0.0);
        assert_eq!(hull_1.exterior().0.len(), 6);

        let hull_2 = coords.concave_hull(2.0, 0.0);
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
        let hull = coords.concave_hull(1.0, 3.0);
        assert_eq!(hull.exterior().0.len(), 5);
    }

    #[test]
    fn test_empty_coords() {
        let coords: Vec<Coord<f64>> = vec![];
        let hull = coords.concave_hull(2.0, 0.0);
        assert!(hull.exterior().0.is_empty());
    }

    #[test]
    fn test_norway_mainland() {
        let norway = geo_test_fixtures::norway_main::<f64>();
        let norway_concave_hull: LineString = geo_test_fixtures::norway_concave_hull::<f64>();
        let hull = norway.concave_hull(2.0, 0.0);
        assert_eq!(hull.exterior(), &norway_concave_hull);
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
        let hull = poly.concave_hull(1.0, 0.0);
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
        let hull = mp.concave_hull(2.0, 0.0);
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
        let hull = linestring.concave_hull(2.0, 0.0);
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
        let hull = mls.concave_hull(2.0, 0.0);
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
        let hull = coords.concave_hull(2.0, 0.0);
        assert_eq!(hull, correct_hull);
    }

    #[test]
    fn test_straight_line() {
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
        let hull = linestring.concave_hull(2.0, 0.0);
        assert_eq!(hull, correct_hull);
    }
}
