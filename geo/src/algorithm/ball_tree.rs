//! # Construction
//!
//! There are three ways to build a ball tree:
//!
//! - **[`BuildBallTree::build_ball_tree`]**: a convenience trait implemented on
//!   `[Point<T>]`, `Vec<Point<T>>`, and `MultiPoint<T>`. Use this when you just
//!   need to index points without associated data.
//!
//!   ```
//!   use geo::{MultiPoint, point};
//!   use geo::algorithm::ball_tree::BuildBallTree;
//!
//!   let points = MultiPoint::new(vec![
//!       point!(x: 0.0, y: 0.0),
//!       point!(x: 1.0, y: 1.0),
//!       point!(x: 2.0, y: 2.0),
//!   ]);
//!   let tree = points.build_ball_tree();
//!   ```
//!
//! - **[`BallTree::new`]**: takes a `Vec<Point<T>>` and a parallel `Vec<D>` of
//!   associated data. Use this when each point carries a label, identifier, or
//!   other payload.
//!
//!   ```
//!   use geo::point;
//!   use geo::algorithm::ball_tree::BallTree;
//!
//!   let points = vec![
//!       point!(x: 0.0, y: 0.0),
//!       point!(x: 1.0, y: 1.0),
//!       point!(x: 2.0, y: 2.0),
//!   ];
//!   let labels = vec!["origin", "middle", "far"];
//!   let tree = BallTree::new(points, labels);
//!   let nearest = tree.nearest_neighbour(&point!(x: 0.1, y: 0.1)).unwrap();
//!   assert_eq!(*nearest.data, "origin");
//!   ```
//!
//! - **[`BallTreeBuilder`]**: a builder that allows configuring parameters such
//!   as the leaf size before construction. Use this when you need to tune tree
//!   structure for your workload.
//!
//!   ```
//!   use geo::point;
//!   use geo::algorithm::ball_tree::BallTreeBuilder;
//!
//!   let points = vec![
//!       point!(x: 0.0, y: 0.0),
//!       point!(x: 1.0, y: 1.0),
//!       point!(x: 2.0, y: 2.0),
//!   ];
//!   let tree = BallTreeBuilder::new()
//!       .leaf_size(8)
//!       .build_pointset(points);
//!   ```
//!
//! # Queries
//!
//! - [`BallTree::nearest_neighbour`] -- single nearest neighbour
//! - [`BallTree::nearest_neighbours`] -- k nearest neighbours, sorted by distance
//! - [`BallTree::within_radius`] -- all points within a given radius

use crate::{Coord, GeoFloat, MultiPoint, Point};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// The default minimum number of points in a leaf node before we stop splitting.
const DEFAULT_LEAF_SIZE: usize = 4;

/// An immutable ball tree built from a set of points, supporting repeated
/// queries without mutation. Points can carry associated data of type `D`
/// (defaulting to `()`).
///
/// See the [module-level documentation](self) for construction options and
/// query methods.
///
/// Construction uses the KD algorithm: recursive median split along the axis
/// of maximum dispersion, producing a balanced binary tree in O(n log n) time.
#[derive(Debug, Clone)]
pub struct BallTree<T: GeoFloat, D = ()> {
    pub(super) nodes: Vec<Node<T>>,
    pub(super) points: Vec<Point<T>>,
    data: Vec<D>,
    pub(super) indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub(super) struct Node<T: GeoFloat> {
    pub(super) center: Coord<T>,
    pub(super) radius: T,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) kind: NodeKind,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum NodeKind {
    Leaf,
    Branch { left: usize, right: usize },
}

#[derive(Debug, Clone, Copy)]
enum SplitAxis {
    X,
    Y,
}

/// Result type for nearest-neighbour queries.
///
/// Contains the matched point, its associated data, the original insertion
/// index, and the Euclidean distance from the query point.
#[derive(Debug, Clone)]
pub struct NearestNeighbour<'a, T: GeoFloat, D> {
    pub point: Point<T>,
    pub data: &'a D,
    pub index: usize,
    pub distance: T,
}

impl<T: GeoFloat, D> BallTree<T, D> {
    /// Build a ball tree from a vector of points and a parallel vector of
    /// associated data.
    ///
    /// If you do not need associated data, the [`BuildBallTree`] trait provides
    /// a convenience method that can be called directly on point collections:
    ///
    /// ```
    /// use geo::point;
    /// use geo::algorithm::ball_tree::BuildBallTree;
    ///
    /// let points = vec![point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
    /// let tree = points.build_ball_tree();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if `points` and `data` have different lengths.
    pub fn new(points: Vec<Point<T>>, data: Vec<D>) -> Self {
        assert_eq!(
            points.len(),
            data.len(),
            "points and data must have the same length"
        );
        let n = points.len();
        let mut indices: Vec<usize> = (0..n).collect();
        let mut nodes = Vec::new();

        if n > 0 {
            build_recursive(&points, &mut indices, 0, n, &mut nodes, DEFAULT_LEAF_SIZE);
        }

        BallTree {
            nodes,
            points,
            data,
            indices,
        }
    }

    /// Returns the number of points in the tree.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns `true` if the tree contains no points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// Compute the squared Euclidean distance between two coords.
#[inline]
pub(super) fn coord_distance_sq<T: GeoFloat>(a: Coord<T>, b: Coord<T>) -> T {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

/// Squared lower-bound Euclidean distance from a query point to the nearest
/// possible point inside a ball with the given `center` and `radius`.
///
/// Returns zero when the query is inside the ball (no pruning possible).
/// This consolidates the sqrt-subtract-re-square pattern that otherwise
/// appears at every ball-bound pruning site.
#[inline]
fn ball_lower_bound_sq<T: GeoFloat>(query: Coord<T>, center: Coord<T>, radius: T) -> T {
    let dist_to_center = coord_distance_sq(query, center).sqrt();
    let gap = dist_to_center - radius;
    if gap > T::zero() {
        gap * gap
    } else {
        T::zero()
    }
}

/// Compare two `GeoFloat` values, treating NaN as equal (for ordering purposes).
#[inline]
fn float_cmp<T: GeoFloat>(a: &T, b: &T) -> Ordering {
    a.partial_cmp(b).unwrap_or(Ordering::Equal)
}

/// Recursively build the ball tree using the KD construction algorithm.
///
/// Operates on the slice `indices[start..end]`, creating a node for the bounding
/// ball, then splitting at the median along the axis of maximum dispersion.
fn build_recursive<T: GeoFloat>(
    points: &[Point<T>],
    indices: &mut [usize],
    start: usize,
    end: usize,
    nodes: &mut Vec<Node<T>>,
    leaf_size: usize,
) -> usize {
    let count = end - start;
    debug_assert!(count > 0);

    // Single pass: compute centroid (sum) and axis dispersion (min/max)
    let mut sum_x = T::zero();
    let mut sum_y = T::zero();
    let mut min_x = T::infinity();
    let mut max_x = T::neg_infinity();
    let mut min_y = T::infinity();
    let mut max_y = T::neg_infinity();
    for &idx in &indices[start..end] {
        let x = points[idx].x();
        let y = points[idx].y();
        sum_x = sum_x + x;
        sum_y = sum_y + y;
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    let n = T::from(count).unwrap();
    let center = Coord {
        x: sum_x / n,
        y: sum_y / n,
    };

    // Compute radius as max distance from centroid
    let radius = indices[start..end]
        .iter()
        .map(|&idx| coord_distance_sq(center, points[idx].0))
        .fold(T::zero(), |acc, d| acc.max(d))
        .sqrt();

    let node_idx = nodes.len();

    // If this subset is small enough, it becomes a leaf
    if count <= leaf_size {
        nodes.push(Node {
            center,
            radius,
            start,
            end,
            kind: NodeKind::Leaf,
        });
        return node_idx;
    }

    // Reserve our slot -- children will be appended after this
    nodes.push(Node {
        center,
        radius,
        start,
        end,
        kind: NodeKind::Leaf, // placeholder, overwritten below
    });

    // Partition at the median along the axis of maximum dispersion
    let mid = start + count / 2;
    let axis = if (max_x - min_x) >= (max_y - min_y) {
        SplitAxis::X
    } else {
        SplitAxis::Y
    };
    let comparator = |&a: &usize, &b: &usize| match axis {
        SplitAxis::X => float_cmp(&points[a].x(), &points[b].x()),
        SplitAxis::Y => float_cmp(&points[a].y(), &points[b].y()),
    };
    indices[start..end].select_nth_unstable_by(mid - start, comparator);

    // Recurse on each half
    let left = build_recursive(points, indices, start, mid, nodes, leaf_size);
    let right = build_recursive(points, indices, mid, end, nodes, leaf_size);

    nodes[node_idx].kind = NodeKind::Branch { left, right };

    node_idx
}

// -- Query: nearest neighbour -------------------------------------------------

impl<T: GeoFloat, D> BallTree<T, D> {
    /// Find the nearest neighbour to `query`.
    ///
    /// Returns `None` if the tree is empty. The returned distance is the true
    /// Euclidean distance (not squared).
    pub fn nearest_neighbour(&self, query: &Point<T>) -> Option<NearestNeighbour<'_, T, D>> {
        if self.is_empty() {
            return None;
        }
        let query_coord = query.0;
        let mut best_dist_sq = T::infinity();
        let mut best_orig_idx = 0usize;
        self.nn_search(0, query_coord, &mut best_dist_sq, &mut best_orig_idx);
        Some(NearestNeighbour {
            point: self.points[best_orig_idx],
            data: &self.data[best_orig_idx],
            index: best_orig_idx,
            distance: best_dist_sq.sqrt(),
        })
    }

    fn nn_search(
        &self,
        node_idx: usize,
        query: Coord<T>,
        best_dist_sq: &mut T,
        best_index: &mut usize,
    ) {
        let Node {
            center,
            radius,
            start,
            end,
            kind,
        } = &self.nodes[node_idx];

        // Prune: if the closest possible point in this ball is further than
        // the current best, skip this subtree.
        if ball_lower_bound_sq(query, *center, *radius) > *best_dist_sq {
            return;
        }

        match kind {
            NodeKind::Leaf => {
                for &idx in &self.indices[*start..*end] {
                    let d = coord_distance_sq(query, self.points[idx].0);
                    if d < *best_dist_sq {
                        *best_dist_sq = d;
                        *best_index = idx;
                    }
                }
            }
            NodeKind::Branch { left, right } => {
                let dl = coord_distance_sq(query, self.nodes[*left].center);
                let dr = coord_distance_sq(query, self.nodes[*right].center);

                let (first, second) = if dl <= dr {
                    (*left, *right)
                } else {
                    (*right, *left)
                };

                self.nn_search(first, query, best_dist_sq, best_index);
                self.nn_search(second, query, best_dist_sq, best_index);
            }
        }
    }
}

// -- Query: k-nearest neighbours ----------------------------------------------

/// Entry in the max-heap used during k-NN search.
/// The largest distance sits at the top and can be evicted first.
///
/// Safety of `Eq`/`Ord`: `dist_sq` is computed from non-NaN coordinates via
/// squaring and addition, so NaN values cannot arise in practice. The
/// `unwrap_or(Ordering::Equal)` in `Ord` is a defensive fallback only.
struct KnnCandidate<T> {
    dist_sq: T,
    /// Index into the `indices` array (not the original index).
    idx: usize,
}

impl<T: PartialEq> PartialEq for KnnCandidate<T> {
    fn eq(&self, other: &Self) -> bool {
        self.dist_sq == other.dist_sq
    }
}

impl<T: PartialEq> Eq for KnnCandidate<T> {}

impl<T: PartialOrd> PartialOrd for KnnCandidate<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: PartialOrd> Ord for KnnCandidate<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist_sq
            .partial_cmp(&other.dist_sq)
            .unwrap_or(Ordering::Equal)
    }
}

impl<T: GeoFloat, D> BallTree<T, D> {
    /// Find the `k` nearest neighbours to `query`, sorted by distance (closest first).
    ///
    /// If `k` is zero or the tree is empty, returns an empty vector.
    /// If `k >= len()`, all points are returned.
    /// Distances are true Euclidean distances (not squared).
    pub fn nearest_neighbours(
        &self,
        query: &Point<T>,
        k: usize,
    ) -> Vec<NearestNeighbour<'_, T, D>> {
        if k == 0 || self.is_empty() {
            return Vec::new();
        }
        let query_coord = query.0;
        let mut heap: BinaryHeap<KnnCandidate<T>> = BinaryHeap::with_capacity(k);
        self.knn_search(0, query_coord, k, &mut heap);

        let mut results: Vec<_> = heap
            .into_iter()
            .map(|c| {
                let orig = self.indices[c.idx];
                NearestNeighbour {
                    point: self.points[orig],
                    data: &self.data[orig],
                    index: orig,
                    distance: c.dist_sq.sqrt(),
                }
            })
            .collect();
        results.sort_by(|a, b| float_cmp(&a.distance, &b.distance));
        results
    }

    fn knn_search(
        &self,
        node_idx: usize,
        query: Coord<T>,
        k: usize,
        heap: &mut BinaryHeap<KnnCandidate<T>>,
    ) {
        let Node {
            center,
            radius,
            start,
            end,
            kind,
        } = &self.nodes[node_idx];

        // Prune using the ball bound
        if heap.len() == k {
            let worst = heap.peek().unwrap().dist_sq;
            if ball_lower_bound_sq(query, *center, *radius) > worst {
                return;
            }
        }

        match kind {
            NodeKind::Leaf => {
                for i in *start..*end {
                    let idx = self.indices[i];
                    let d = coord_distance_sq(query, self.points[idx].0);
                    if heap.len() < k {
                        heap.push(KnnCandidate { dist_sq: d, idx: i });
                    } else if d < heap.peek().unwrap().dist_sq {
                        heap.pop();
                        heap.push(KnnCandidate { dist_sq: d, idx: i });
                    }
                }
            }
            NodeKind::Branch { left, right } => {
                let dl = coord_distance_sq(query, self.nodes[*left].center);
                let dr = coord_distance_sq(query, self.nodes[*right].center);

                let (first, second) = if dl <= dr {
                    (*left, *right)
                } else {
                    (*right, *left)
                };

                self.knn_search(first, query, k, heap);
                self.knn_search(second, query, k, heap);
            }
        }
    }
}

// -- Query: within radius -----------------------------------------------------

impl<T: GeoFloat, D> BallTree<T, D> {
    /// Find all points within `radius` of `query`.
    ///
    /// The returned distances are true Euclidean distances (not squared).
    /// Results are not in any particular order.
    pub fn within_radius(&self, query: &Point<T>, radius: T) -> Vec<NearestNeighbour<'_, T, D>> {
        if self.is_empty() {
            return Vec::new();
        }
        let query_coord = query.0;
        let search_radius_sq = radius * radius;
        let mut results = Vec::new();
        self.radius_search(0, query_coord, radius, search_radius_sq, &mut results);
        results
    }

    fn radius_search<'a>(
        &'a self,
        node_idx: usize,
        query: Coord<T>,
        search_radius: T,
        search_radius_sq: T,
        results: &mut Vec<NearestNeighbour<'a, T, D>>,
    ) {
        let node = &self.nodes[node_idx];
        let dist_to_center = coord_distance_sq(query, node.center).sqrt();

        // Prune: the ball is entirely outside the search radius
        if dist_to_center - node.radius > search_radius {
            return;
        }

        // Bulk include: the ball is entirely within the search radius
        if dist_to_center + node.radius <= search_radius {
            results.extend(self.indices[node.start..node.end].iter().map(|&idx| {
                NearestNeighbour {
                    point: self.points[idx],
                    data: &self.data[idx],
                    index: idx,
                    distance: coord_distance_sq(query, self.points[idx].0).sqrt(),
                }
            }));
            return;
        }

        match node.kind {
            NodeKind::Leaf => {
                results.extend(
                    self.indices[node.start..node.end]
                        .iter()
                        .filter_map(|&idx| {
                            let d = coord_distance_sq(query, self.points[idx].0);
                            (d <= search_radius_sq).then_some(NearestNeighbour {
                                point: self.points[idx],
                                data: &self.data[idx],
                                index: idx,
                                distance: d.sqrt(),
                            })
                        }),
                );
            }
            NodeKind::Branch { left, right } => {
                self.radius_search(left, query, search_radius, search_radius_sq, results);
                self.radius_search(right, query, search_radius, search_radius_sq, results);
            }
        }
    }
}

// -- Builder: BallTreeBuilder -------------------------------------------------

/// A builder for constructing a [`BallTree`] with configurable parameters.
///
/// The main tuneable is the **leaf size**: the maximum number of points stored
/// in a leaf node before the tree splits further. Larger leaves amortise
/// traversal overhead for brute-force-heavy queries; smaller leaves prune
/// more aggressively during search.
///
/// ```
/// use geo::point;
/// use geo::algorithm::ball_tree::BallTreeBuilder;
///
/// let points = vec![
///     point!(x: 0.0, y: 0.0),
///     point!(x: 1.0, y: 1.0),
///     point!(x: 2.0, y: 2.0),
/// ];
/// let labels = vec!["origin", "middle", "far"];
/// let tree = BallTreeBuilder::new()
///     .leaf_size(8)
///     .build(points, labels);
/// let nearest = tree.nearest_neighbour(&point!(x: 0.1, y: 0.1)).unwrap();
/// assert_eq!(*nearest.data, "origin");
/// ```
pub struct BallTreeBuilder {
    leaf_size: usize,
}

impl BallTreeBuilder {
    /// Create a new builder with the default leaf size.
    pub fn new() -> Self {
        Self {
            leaf_size: DEFAULT_LEAF_SIZE,
        }
    }

    /// Set the leaf size (maximum number of points in a leaf node).
    ///
    /// A leaf size of zero is treated as one (every leaf holds at least one point).
    pub fn leaf_size(mut self, size: usize) -> Self {
        self.leaf_size = size.max(1);
        self
    }

    /// Build a [`BallTree`] from points and parallel associated data.
    ///
    /// # Panics
    ///
    /// Panics if `points` and `data` have different lengths.
    pub fn build<T: GeoFloat, D>(self, points: Vec<Point<T>>, data: Vec<D>) -> BallTree<T, D> {
        assert_eq!(
            points.len(),
            data.len(),
            "points and data must have the same length"
        );
        let n = points.len();
        let mut indices: Vec<usize> = (0..n).collect();
        let mut nodes = Vec::new();

        if n > 0 {
            build_recursive(&points, &mut indices, 0, n, &mut nodes, self.leaf_size);
        }

        BallTree {
            nodes,
            points,
            data,
            indices,
        }
    }

    /// Build a [`BallTree`] from points without associated data.
    ///
    /// This is the builder equivalent of [`BuildBallTree::build_ball_tree`].
    pub fn build_pointset<T: GeoFloat>(self, points: Vec<Point<T>>) -> BallTree<T> {
        let data = vec![(); points.len()];
        self.build(points, data)
    }
}

impl Default for BallTreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// -- Trait: BuildBallTree -----------------------------------------------------

/// Construct a [`BallTree`] from a collection of points (without associated data).
///
/// This is the convenience route for building a `BallTree<T, ()>`. To attach
/// per-point data, use [`BallTree::new`] directly.
pub trait BuildBallTree<T: GeoFloat> {
    fn build_ball_tree(&self) -> BallTree<T>;
}

impl<T: GeoFloat> BuildBallTree<T> for [Point<T>] {
    fn build_ball_tree(&self) -> BallTree<T> {
        let data = vec![(); self.len()];
        BallTree::new(self.to_vec(), data)
    }
}

impl<T: GeoFloat> BuildBallTree<T> for Vec<Point<T>> {
    fn build_ball_tree(&self) -> BallTree<T> {
        self.as_slice().build_ball_tree()
    }
}

impl<T: GeoFloat> BuildBallTree<T> for MultiPoint<T> {
    fn build_ball_tree(&self) -> BallTree<T> {
        self.0.build_ball_tree()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    fn make_points() -> Vec<Point<f64>> {
        vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 2.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 1.0),
            point!(x: 0.0, y: 2.0),
            point!(x: 1.0, y: 2.0),
            point!(x: 2.0, y: 2.0),
        ]
    }

    #[test]
    fn test_construction_basic() {
        let tree = make_points().build_ball_tree();
        assert_eq!(tree.len(), 9);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_construction_empty() {
        let tree: BallTree<f64> = BallTree::new(vec![], vec![]);
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_construction_single_point() {
        let tree = vec![point!(x: 1.0_f64, y: 2.0)].build_ball_tree();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn test_bounding_invariant() {
        // Every point should be within the root node's bounding ball
        let points = make_points();
        let tree = points.build_ball_tree();
        let root = &tree.nodes[0];
        for point in &points {
            let d = coord_distance_sq(root.center, point.0).sqrt();
            assert!(
                d <= root.radius + f64::EPSILON,
                "point {:?} outside root ball",
                point
            );
        }
    }

    #[test]
    fn test_nn_exact_match() {
        let tree = make_points().build_ball_tree();
        let result = tree.nearest_neighbour(&point!(x: 1.0, y: 1.0)).unwrap();
        assert_eq!(result.index, 4);
        assert!((result.distance - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_nn_closest_point() {
        let tree = make_points().build_ball_tree();
        let result = tree.nearest_neighbour(&point!(x: 0.1, y: 0.1)).unwrap();
        assert_eq!(result.index, 0); // (0,0) is closest to (0.1, 0.1)
    }

    #[test]
    fn test_nn_empty_tree() {
        let tree: BallTree<f64> = BallTree::new(vec![], vec![]);
        assert!(tree.nearest_neighbour(&point!(x: 0.0, y: 0.0)).is_none());
    }

    #[test]
    fn test_nn_brute_force_comparison() {
        // Compare ball tree NN against brute-force for various query points
        let points = make_points();
        let tree = points.build_ball_tree();

        let queries = vec![
            point!(x: 0.5, y: 0.5),
            point!(x: -1.0, y: -1.0),
            point!(x: 1.5, y: 1.5),
            point!(x: 3.0, y: 0.0),
            point!(x: 1.0, y: 0.5),
        ];

        for query in &queries {
            let tree_result = tree.nearest_neighbour(query).unwrap();

            // Brute-force
            let (_bf_idx, bf_dist) = points
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let dx = p.x() - query.x();
                    let dy = p.y() - query.y();
                    (i, (dx * dx + dy * dy).sqrt())
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();

            assert!(
                (tree_result.distance - bf_dist).abs() < 1e-10,
                "Distance mismatch for query {:?}",
                query
            );
        }
    }

    #[test]
    fn test_knn_basic() {
        let tree = make_points().build_ball_tree();
        let results = tree.nearest_neighbours(&point!(x: 0.0, y: 0.0), 3);
        assert_eq!(results.len(), 3);
        // Should be sorted by distance
        for i in 1..results.len() {
            assert!(results[i].distance >= results[i - 1].distance);
        }
        // Closest should be (0,0) itself
        assert_eq!(results[0].index, 0);
    }

    #[test]
    fn test_knn_k_zero() {
        let tree = make_points().build_ball_tree();
        let results = tree.nearest_neighbours(&point!(x: 0.0, y: 0.0), 0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_knn_k_greater_than_n() {
        let tree = make_points().build_ball_tree();
        let results = tree.nearest_neighbours(&point!(x: 0.0, y: 0.0), 100);
        assert_eq!(results.len(), 9);
    }

    #[test]
    fn test_knn_k_one_equals_nn() {
        let tree = make_points().build_ball_tree();
        let query = point!(x: 0.5, y: 0.5);
        let nn = tree.nearest_neighbour(&query).unwrap();
        let knn = tree.nearest_neighbours(&query, 1);
        assert_eq!(knn.len(), 1);
        assert_eq!(knn[0].index, nn.index);
        assert!((knn[0].distance - nn.distance).abs() < 1e-10);
    }

    #[test]
    fn test_knn_brute_force_comparison() {
        let points = make_points();
        let tree = points.build_ball_tree();
        let query = point!(x: 0.5, y: 0.5);
        let k = 4;

        let results = tree.nearest_neighbours(&query, k);

        // Brute-force k-NN
        let mut dists: Vec<(usize, f64)> = points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let dx = p.x() - query.x();
                let dy = p.y() - query.y();
                (i, (dx * dx + dy * dy).sqrt())
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (i, result) in results.iter().enumerate() {
            assert!(
                (result.distance - dists[i].1).abs() < 1e-10,
                "k-NN distance mismatch at rank {i}: got {} expected {}",
                result.distance,
                dists[i].1,
            );
        }
    }

    #[test]
    fn test_within_radius_basic() {
        let tree = make_points().build_ball_tree();
        // All points within distance 1.0 of origin: (0,0), (1,0), (0,1)
        let results = tree.within_radius(&point!(x: 0.0, y: 0.0), 1.0);
        let mut indices: Vec<usize> = results.iter().map(|r| r.index).collect();
        indices.sort();
        assert_eq!(indices, vec![0, 1, 3]);
    }

    #[test]
    fn test_within_radius_empty_result() {
        let tree = make_points().build_ball_tree();
        let results = tree.within_radius(&point!(x: 10.0, y: 10.0), 0.1);
        assert!(results.is_empty());
    }

    #[test]
    fn test_within_radius_all_points() {
        let tree = make_points().build_ball_tree();
        let results = tree.within_radius(&point!(x: 1.0, y: 1.0), 10.0);
        assert_eq!(results.len(), 9);
    }

    #[test]
    fn test_within_radius_brute_force_comparison() {
        let points = make_points();
        let tree = points.build_ball_tree();
        let query = point!(x: 1.0, y: 1.0);
        let radius = 1.1;

        let results = tree.within_radius(&query, radius);
        let mut tree_indices: Vec<usize> = results.iter().map(|r| r.index).collect();
        tree_indices.sort();

        // Brute-force
        let mut bf_indices: Vec<usize> = points
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                let dx = p.x() - query.x();
                let dy = p.y() - query.y();
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .map(|(i, _)| i)
            .collect();
        bf_indices.sort();

        assert_eq!(tree_indices, bf_indices);
    }

    #[test]
    fn test_with_associated_data() {
        let points = vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ];
        let data = vec!["alpha", "beta", "gamma"];
        let tree = BallTree::new(points, data);

        // NN
        let result = tree.nearest_neighbour(&point!(x: 0.9, y: 0.9)).unwrap();
        assert_eq!(result.index, 1);
        assert_eq!(result.point, point!(x: 1.0, y: 1.0));
        assert_eq!(*result.data, "beta");

        // k-NN
        let results = tree.nearest_neighbours(&point!(x: 0.1, y: 0.1), 2);
        assert_eq!(*results[0].data, "alpha");
        assert_eq!(*results[1].data, "beta");

        // Radius
        let results = tree.within_radius(&point!(x: 0.0, y: 0.0), 1.5);
        let mut data_found: Vec<&str> = results.iter().map(|r| *r.data).collect();
        data_found.sort();
        assert_eq!(data_found, vec!["alpha", "beta"]);
    }

    #[test]
    fn test_build_ball_tree_trait_multipoint() {
        let mp = MultiPoint::new(vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ]);
        let tree = mp.build_ball_tree();
        assert_eq!(tree.len(), 3);
        let result = tree.nearest_neighbour(&point!(x: 0.1, y: 0.1)).unwrap();
        assert_eq!(result.index, 0);
    }

    #[test]
    fn test_build_ball_tree_trait_slice() {
        let points = vec![
            point!(x: 0.0_f64, y: 0.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 2.0, y: 2.0),
        ];
        let tree = points.as_slice().build_ball_tree();
        assert_eq!(tree.len(), 3);
    }

    #[test]
    fn test_duplicate_points() {
        // All points at the same location
        let points: Vec<Point<f64>> = (0..20).map(|_| point!(x: 5.0, y: 5.0)).collect();
        let data: Vec<usize> = (0..20).collect();
        let tree = BallTree::new(points, data);
        assert_eq!(tree.len(), 20);

        let nn = tree.nearest_neighbour(&point!(x: 5.0, y: 5.0)).unwrap();
        assert!((nn.distance - 0.0).abs() < 1e-10);

        let knn = tree.nearest_neighbours(&point!(x: 5.0, y: 5.0), 5);
        assert_eq!(knn.len(), 5);
        for r in &knn {
            assert!((r.distance - 0.0).abs() < 1e-10);
        }

        let within = tree.within_radius(&point!(x: 5.0, y: 5.0), 0.1);
        assert_eq!(within.len(), 20);
    }

    #[test]
    fn test_collinear_points() {
        // All points on the x-axis (zero y-spread)
        let points: Vec<Point<f64>> = (0..20).map(|i| point!(x: i as f64, y: 0.0)).collect();
        let tree = points.build_ball_tree();

        let nn = tree.nearest_neighbour(&point!(x: 5.5, y: 0.0)).unwrap();
        // Points 5 and 6 are both at distance 0.5; either is a valid result
        assert!(nn.index == 5 || nn.index == 6);
        assert_eq!(nn.distance, 0.5);

        let knn = tree.nearest_neighbours(&point!(x: 10.0, y: 0.0), 3);
        assert_eq!(knn.len(), 3);
        let mut indices: Vec<usize> = knn.iter().map(|r| r.index).collect();
        indices.sort();
        assert_eq!(indices, vec![9, 10, 11]);
    }

    #[test]
    fn test_negative_coordinates() {
        let points = vec![
            point!(x: -5.0_f64, y: -5.0),
            point!(x: -1.0, y: -1.0),
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 1.0),
            point!(x: 5.0, y: 5.0),
        ];
        let tree = points.build_ball_tree();

        let nn = tree.nearest_neighbour(&point!(x: -4.0, y: -4.0)).unwrap();
        assert_eq!(nn.index, 0); // (-5,-5) is closest
    }

    #[test]
    fn test_f32_support() {
        let points: Vec<Point<f32>> = vec![
            point!(x: 0.0_f32, y: 0.0),
            point!(x: 1.0_f32, y: 0.0),
            point!(x: 0.0_f32, y: 1.0),
        ];
        let tree = points.build_ball_tree();
        assert_eq!(tree.len(), 3);

        let nn = tree.nearest_neighbour(&point!(x: 0.1_f32, y: 0.1)).unwrap();
        assert_eq!(nn.index, 0);

        let knn = tree.nearest_neighbours(&point!(x: 0.1_f32, y: 0.1), 2);
        assert_eq!(knn.len(), 2);
        assert_eq!(knn[0].index, 0);
    }

    #[test]
    fn test_within_radius_zero() {
        let tree = make_points().build_ball_tree();

        // Radius 0 at an exact point should include that point
        let results = tree.within_radius(&point!(x: 1.0, y: 1.0), 0.0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].index, 4);

        // Radius 0 at a non-point location should be empty
        let results = tree.within_radius(&point!(x: 0.5, y: 0.5), 0.0);
        assert!(results.is_empty());
    }

    #[test]
    fn test_empty_tree_all_queries() {
        let tree: BallTree<f64> = BallTree::new(vec![], vec![]);
        assert!(tree.nearest_neighbour(&point!(x: 0.0, y: 0.0)).is_none());
        assert!(
            tree.nearest_neighbours(&point!(x: 0.0, y: 0.0), 5)
                .is_empty()
        );
        assert!(tree.within_radius(&point!(x: 0.0, y: 0.0), 10.0).is_empty());
    }

    #[test]
    #[should_panic(expected = "points and data must have the same length")]
    fn test_mismatched_lengths_panics() {
        let _tree = BallTree::new(
            vec![point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)],
            vec![()],
        );
    }

    #[test]
    fn test_large_point_set() {
        // Ensure the tree works with more points than DEFAULT_LEAF_SIZE
        let points: Vec<Point<f64>> = (0..100)
            .map(|i| point!(x: (i % 10) as f64, y: (i / 10) as f64))
            .collect();
        let tree = points.build_ball_tree();
        assert_eq!(tree.len(), 100);

        // Verify NN for a few queries against brute-force
        let queries = vec![
            point!(x: 4.5, y: 4.5),
            point!(x: 0.0, y: 0.0),
            point!(x: 9.0, y: 9.0),
        ];
        for query in &queries {
            let tree_result = tree.nearest_neighbour(query).unwrap();
            let (bf_idx, _) = points
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let dx = p.x() - query.x();
                    let dy = p.y() - query.y();
                    (i, dx * dx + dy * dy)
                })
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();
            assert_eq!(tree_result.index, bf_idx);
        }
    }

    // -- BallTreeBuilder tests ------------------------------------------------

    #[test]
    fn test_builder_default_matches_new() {
        let points = make_points();
        let data: Vec<usize> = (0..points.len()).collect();

        let tree_new = BallTree::new(points.clone(), data.clone());
        let tree_builder = BallTreeBuilder::new().build(points, data);

        // Both trees should yield identical NN results for several queries
        let queries = vec![
            point!(x: 0.5, y: 0.5),
            point!(x: 1.5, y: 1.5),
            point!(x: -1.0, y: -1.0),
        ];
        for query in &queries {
            let nn_new = tree_new.nearest_neighbour(query).unwrap();
            let nn_builder = tree_builder.nearest_neighbour(query).unwrap();
            assert_eq!(nn_new.index, nn_builder.index);
            assert!((nn_new.distance - nn_builder.distance).abs() < 1e-10);
        }
    }

    /// Helper: brute-force nearest neighbour for verification.
    fn brute_force_nn(points: &[Point<f64>], query: &Point<f64>) -> (usize, f64) {
        points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let dx = p.x() - query.x();
                let dy = p.y() - query.y();
                (i, (dx * dx + dy * dy).sqrt())
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }

    /// Helper: brute-force k-NN for verification.
    fn brute_force_knn(points: &[Point<f64>], query: &Point<f64>, k: usize) -> Vec<(usize, f64)> {
        let mut dists: Vec<(usize, f64)> = points
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let dx = p.x() - query.x();
                let dy = p.y() - query.y();
                (i, (dx * dx + dy * dy).sqrt())
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        dists.truncate(k);
        dists
    }

    /// Helper: brute-force radius search for verification.
    fn brute_force_radius(points: &[Point<f64>], query: &Point<f64>, radius: f64) -> Vec<usize> {
        let mut indices: Vec<usize> = points
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                let dx = p.x() - query.x();
                let dy = p.y() - query.y();
                (dx * dx + dy * dy).sqrt() <= radius
            })
            .map(|(i, _)| i)
            .collect();
        indices.sort();
        indices
    }

    #[test]
    fn test_builder_custom_leaf_sizes() {
        let points: Vec<Point<f64>> = (0..50)
            .map(|i| point!(x: (i % 10) as f64, y: (i / 10) as f64))
            .collect();

        let queries = vec![
            point!(x: 3.3, y: 2.7),
            point!(x: 0.0, y: 0.0),
            point!(x: 9.0, y: 4.0),
            point!(x: 5.5, y: 2.5),
        ];

        for leaf_size in [1, 2, 8, 16, 50] {
            let tree = BallTreeBuilder::new()
                .leaf_size(leaf_size)
                .build_pointset(points.clone());

            for query in &queries {
                // NN -- compare distances, not indices, because ties are
                // broken arbitrarily depending on tree structure.
                let nn = tree.nearest_neighbour(query).unwrap();
                let (_, bf_dist) = brute_force_nn(&points, query);
                assert!(
                    (nn.distance - bf_dist).abs() < 1e-10,
                    "NN distance mismatch with leaf_size={leaf_size} for query {query:?}"
                );

                // k-NN
                let k = 5;
                let knn = tree.nearest_neighbours(query, k);
                let bf_knn = brute_force_knn(&points, query, k);
                assert_eq!(knn.len(), bf_knn.len());
                for (tree_r, (_, bf_d)) in knn.iter().zip(bf_knn.iter()) {
                    assert!(
                        (tree_r.distance - bf_d).abs() < 1e-10,
                        "k-NN distance mismatch with leaf_size={leaf_size}"
                    );
                }

                // Radius
                let radius = 2.0;
                let within = tree.within_radius(query, radius);
                let mut tree_indices: Vec<usize> = within.iter().map(|r| r.index).collect();
                tree_indices.sort();
                let bf_indices = brute_force_radius(&points, query, radius);
                assert_eq!(
                    tree_indices, bf_indices,
                    "Radius search mismatch with leaf_size={leaf_size} for query {query:?}"
                );
            }
        }
    }

    #[test]
    fn test_builder_pointset_matches_trait() {
        let points = make_points();

        let tree_trait = points.build_ball_tree();
        let tree_builder = BallTreeBuilder::new().build_pointset(points.clone());

        let queries = vec![
            point!(x: 0.5, y: 0.5),
            point!(x: 1.5, y: 1.5),
            point!(x: -1.0, y: -1.0),
        ];
        for query in &queries {
            let nn_trait = tree_trait.nearest_neighbour(query).unwrap();
            let nn_builder = tree_builder.nearest_neighbour(query).unwrap();
            assert_eq!(nn_trait.index, nn_builder.index);
            assert!((nn_trait.distance - nn_builder.distance).abs() < 1e-10);
        }
    }

    #[test]
    fn test_builder_leaf_size_zero_clamped_to_one() {
        // leaf_size(0) should be clamped to 1 and still produce correct results
        let points = make_points();
        let tree = BallTreeBuilder::new()
            .leaf_size(0)
            .build_pointset(points.clone());

        let nn = tree.nearest_neighbour(&point!(x: 0.1, y: 0.1)).unwrap();
        let (bf_idx, _) = brute_force_nn(&points, &point!(x: 0.1, y: 0.1));
        assert_eq!(nn.index, bf_idx);
    }
}
