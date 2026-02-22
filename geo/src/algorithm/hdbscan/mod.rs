//! Hierarchical DBSCAN (HDBSCAN) clustering algorithm.
//!
//! HDBSCAN improves on DBSCAN by eliminating the epsilon parameter and
//! producing a cluster hierarchy, making it more robust to clusters of varying
//! density.
//!
//! # References
//!
//! Campello, R. J. G. B., Moulavi, D., & Sander, J. (2013).
//! *Density-Based Clustering Based on Hierarchical Density Estimates.*
//! In Advances in Knowledge Discovery and Data Mining (PAKDD 2013).

use crate::Coord;
use crate::GeoFloat;
use crate::MultiPoint;
use crate::Point;
use crate::algorithm::ball_tree::BallTree;
use crate::algorithm::ball_tree::NodeKind;
use std::collections::HashMap;
use std::collections::HashSet;

/// Squared Euclidean distance between two coords.
#[inline]
fn dist_sq<T: GeoFloat>(a: Coord<T>, b: Coord<T>) -> T {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

/// Result of an HDBSCAN clustering run.
///
/// Contains both cluster labels and GLOSH outlier scores for every input point.
#[derive(Debug, Clone, PartialEq)]
pub struct HdbscanResult<T> {
    /// Cluster assignment for each input point, in input order.
    /// `Some(id)` for clustered points, `None` for noise.
    pub labels: Vec<Option<usize>>,

    /// GLOSH (Global-Local Outlier Scores from Hierarchies) outlier score for
    /// each input point, in the range [0, 1]. Higher values indicate stronger
    /// outliers.
    pub outlier_scores: Vec<T>,
}

/// Errors that can occur during [`hdbscan`](Hdbscan::hdbscan) clustering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HdbscanError {
    /// `min_cluster_size` is out of range.
    ///
    /// `min_cluster_size` must be at least 2 and no greater than the number
    /// of input points.
    InvalidMinClusterSize {
        /// The supplied `min_cluster_size`.
        min_cluster_size: usize,
        /// The number of input points.
        n: usize,
    },
    /// `min_samples` must be at least 1.
    InvalidMinSamples {
        /// The supplied `min_samples`.
        min_samples: usize,
    },
}

impl std::fmt::Display for HdbscanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HdbscanError::InvalidMinClusterSize {
                min_cluster_size,
                n,
            } => write!(
                f,
                "invalid min_cluster_size {min_cluster_size}: must be in 2..={n} (number of input points)"
            ),
            HdbscanError::InvalidMinSamples { min_samples } => {
                write!(f, "invalid min_samples {min_samples}: must be at least 1")
            }
        }
    }
}

impl std::error::Error for HdbscanError {}

/// Perform [HDBSCAN](https://en.wikipedia.org/wiki/HDBSCAN) (Hierarchical Density-Based Spatial Clustering of Applications with Noise) clustering on a set of points.
///
/// HDBSCAN extends DBSCAN by converting it into a hierarchical clustering
/// algorithm, then extracting a flat clustering based on cluster stability.
/// Unlike DBSCAN, it does not require an epsilon parameter and can find
/// clusters of varying densities.
///
/// The central concept is the *mutual reachability distance* between two
/// points *a* and *b*: `max(core_dist(a), core_dist(b), dist(a, b))`, where
/// `core_dist(p)` is the distance from *p* to its *k*-th nearest neighbour
/// (with *k* = `min_samples`). This inflates distances in sparse regions
/// while leaving dense regions unchanged, so the minimum spanning tree of
/// the resulting complete graph -- the mutual reachability graph -- captures
/// the density-based cluster structure of the data.
///
/// The minimum spanning tree of this graph encodes a single-linkage
/// hierarchy over the points: sorting its edges by weight and removing them
/// in decreasing order produces a sequence of merges (the dendrogram) that
/// describes how clusters form as the density threshold varies. HDBSCAN
/// condenses this hierarchy and selects the most stable clusters from it.
///
/// # Parameters
///
/// - `min_cluster_size`: The minimum number of points required to form a
///   cluster. Larger values produce fewer, more conservative clusters.
///   Also used as the number of neighbours for core distance computation
///   when calling [`hdbscan`](Hdbscan::hdbscan).
/// - `min_samples` (only on [`hdbscan_with_min_samples`](Hdbscan::hdbscan_with_min_samples)):
///   The number of neighbours (including the point itself) used to compute
///   core distances. Controls how conservative the clustering is.
///
/// # Returns
///
/// A [`Result`] containing an [`HdbscanResult`] with cluster labels and
/// GLOSH outlier scores, or an [`HdbscanError`] if parameters are invalid.
/// Empty input is not an error; it yields an `HdbscanResult` with empty
/// vectors.
///
/// # Errors
///
/// Returns [`HdbscanError::InvalidMinClusterSize`] when `min_cluster_size`
/// is less than 2 or greater than the number of input points, and
/// [`HdbscanError::InvalidMinSamples`] when `min_samples` is less than 1.
///
/// # Algorithm
///
/// 1. **Transform the space.** Compute each point's *core distance* (distance
///    to its k-th nearest neighbour) using a
///    [ball tree](https://en.wikipedia.org/wiki/Ball_tree). The mutual
///    reachability distance then spreads sparse points apart while leaving
///    dense regions unchanged, making the algorithm robust to noise.
/// 2. **Build the minimum spanning tree** of the mutual reachability graph
///    using [Borůvka's algorithm](https://en.wikipedia.org/wiki/Bor%C5%AFvka%27s_algorithm)
///    with ball tree acceleration. This tree encodes the full single-linkage
///    hierarchy without needing to materialise all pairwise distances.
/// 3. **Construct the cluster hierarchy.** Sort the MST edges by weight and
///    merge components via union-find to produce a dendrogram showing how
///    clusters form as the density threshold varies.
/// 4. **Condense the cluster tree.** Splits that produce a component smaller
///    than `min_cluster_size` are not treated as true splits; instead those
///    points "fall out" of their parent cluster. This yields a simpler tree
///    whose cluster widths reflect how points progressively leave.
/// 5. **Extract the clusters.** Select the most persistent clusters by
///    measuring stability (accumulated excess of mass). A parent is kept
///    only when its own stability exceeds the combined stability of its
///    children; otherwise the children are kept.
/// 6. Compute [GLOSH](https://doi.org/10.1007/978-3-319-18123-3_2) outlier
///    scores for each point.
///
/// For an accessible walk-through with visualisations, see
/// [How HDBSCAN Works](https://hdbscan.readthedocs.io/en/latest/how_hdbscan_works.html).
///
/// # Examples
///
/// ```
/// use geo::{Hdbscan, point};
///
/// let points = vec![
///     // Cluster 1
///     point!(x: 0.0, y: 0.0),
///     point!(x: 0.5, y: 0.0),
///     point!(x: 0.0, y: 0.5),
///     point!(x: 0.5, y: 0.5),
///     point!(x: 0.25, y: 0.25),
///     // Cluster 2
///     point!(x: 10.0, y: 10.0),
///     point!(x: 10.5, y: 10.0),
///     point!(x: 10.0, y: 10.5),
///     point!(x: 10.5, y: 10.5),
///     point!(x: 10.25, y: 10.25),
/// ];
///
/// let result = points.hdbscan(5).unwrap();
///
/// // Points in cluster 1 share the same label
/// assert_eq!(result.labels[0], result.labels[1]);
/// // Points in cluster 2 share a different label
/// assert_eq!(result.labels[5], result.labels[6]);
/// // The two clusters have different labels
/// assert_ne!(result.labels[0], result.labels[5]);
/// ```
pub trait Hdbscan<T>
where
    T: GeoFloat + Send + Sync,
{
    /// Perform HDBSCAN clustering using `min_cluster_size` for both the
    /// minimum cluster size and the number of neighbours used to compute
    /// core distances.
    ///
    /// See the [trait-level documentation](Hdbscan) for details and error
    /// conditions.
    fn hdbscan(&self, min_cluster_size: usize) -> Result<HdbscanResult<T>, HdbscanError> {
        self.hdbscan_with_min_samples(min_cluster_size, min_cluster_size)
    }

    /// Perform HDBSCAN clustering with separate `min_cluster_size` and
    /// `min_samples` parameters.
    ///
    /// `min_samples` controls the number of neighbours (including the point
    /// itself) used to compute core distances. See the
    /// [trait-level documentation](Hdbscan) for details and error conditions.
    fn hdbscan_with_min_samples(
        &self,
        min_cluster_size: usize,
        min_samples: usize,
    ) -> Result<HdbscanResult<T>, HdbscanError>;
}

impl<T> Hdbscan<T> for MultiPoint<T>
where
    T: GeoFloat + Send + Sync,
{
    fn hdbscan_with_min_samples(
        &self,
        min_cluster_size: usize,
        min_samples: usize,
    ) -> Result<HdbscanResult<T>, HdbscanError> {
        hdbscan_impl(&self.0, min_cluster_size, min_samples)
    }
}

impl<T> Hdbscan<T> for [Point<T>]
where
    T: GeoFloat + Send + Sync,
{
    fn hdbscan_with_min_samples(
        &self,
        min_cluster_size: usize,
        min_samples: usize,
    ) -> Result<HdbscanResult<T>, HdbscanError> {
        hdbscan_impl(self, min_cluster_size, min_samples)
    }
}

// ---------------------------------------------------------------------------
// Core implementation
// ---------------------------------------------------------------------------

fn hdbscan_impl<T: GeoFloat + Send + Sync>(
    points: &[Point<T>],
    min_cluster_size: usize,
    min_samples: usize,
) -> Result<HdbscanResult<T>, HdbscanError> {
    let n = points.len();

    // Check `min_cluster_size` before `min_samples` so that the default
    // `hdbscan(k)` method -- which forwards `k` to both parameters -- surfaces
    // the more user-facing `InvalidMinClusterSize` on `k < 2`.
    if min_cluster_size < 2 {
        return Err(HdbscanError::InvalidMinClusterSize {
            min_cluster_size,
            n,
        });
    }

    if min_samples < 1 {
        return Err(HdbscanError::InvalidMinSamples { min_samples });
    }

    if n == 0 {
        return Ok(HdbscanResult {
            labels: Vec::new(),
            outlier_scores: Vec::new(),
        });
    }

    if min_cluster_size > n {
        return Err(HdbscanError::InvalidMinClusterSize {
            min_cluster_size,
            n,
        });
    }

    // Build ball tree once for both core distances and MST construction
    let tree = BallTree::new(points.iter().copied());

    // 1. Compute core distances using ball tree k-NN
    let core_data = compute_core_data(&tree, points, min_samples);

    // 2. Build MST over the mutual reachability graph (dual-tree Boruvka)
    let mst = boruvka_mst(&tree, points, &core_data);

    // 3. Build dendrogram (label step): sort MST edges, merge via union-find
    let dendrogram = label(&mst, n);

    // 4. Condense the dendrogram
    let condensed = condense_tree(&dendrogram, min_cluster_size);

    // Steps 5-7 use HashMaps keyed by cluster ID. Cluster IDs are dense
    // integers starting at n_points and incrementing, so these could be
    // replaced with flat Vecs using offset indexing (id - n_points) for
    // lower overhead. Left as-is because these stages are <1% of total
    // runtime at scale.

    // 5. Compute cluster stabilities and select clusters
    let stability = get_stability(&condensed);
    let (cluster_labels, is_cluster) = find_clusters(&condensed, &stability);

    // 6. Extract flat labels from selected clusters
    let labels = extract_labels(&condensed, &cluster_labels, &is_cluster, n);

    // 7. Compute GLOSH outlier scores
    let outlier_scores = glosh(&condensed, &labels, n);

    Ok(HdbscanResult {
        labels,
        outlier_scores,
    })
}

// ---------------------------------------------------------------------------
// Step 1: Core distances via ball tree k-NN
// ---------------------------------------------------------------------------

/// Compute the core distance and k-NN indices for each point.
///
/// Runs a single pass (parallel with the `multithreading` feature; sequential
/// otherwise): for each point p, run the ball-tree k-NN query with k =
/// `min_samples`, extract `core_distances[p]` (the distance to the k-th
/// neighbour, including self), and record the neighbour indices.
///
/// The k-NN index table is retained on `CoreData` for use in per-round seeding
/// via `seed_best_edges`.
///
/// If the tree holds fewer than `min_samples` points (which can only happen
/// when the caller passes `min_samples > n`, a borderline-valid combination
/// that `hdbscan_impl` currently accepts), the core distance is reported as
/// zero. In that fallback the mutual-reachability distance collapses to the
/// raw Euclidean distance, so the algorithm degenerates to plain
/// single-linkage clustering rather than marking every point as noise.
pub(crate) fn compute_core_data<T: GeoFloat + Send + Sync>(
    tree: &BallTree<T>,
    points: &[Point<T>],
    min_samples: usize,
) -> CoreData<T> {
    let n = points.len();
    let mut core_distances = vec![T::zero(); n];
    let mut knn_indices: Vec<Vec<usize>> = vec![Vec::new(); n];

    #[cfg(feature = "multithreading")]
    {
        let num_threads = std::thread::available_parallelism()
            .map(|t| t.get())
            .unwrap_or(1);
        let chunk_size = n.div_ceil(num_threads).max(1);

        std::thread::scope(|s| {
            for ((dist_chunk, knn_chunk), points_chunk) in core_distances
                .chunks_mut(chunk_size)
                .zip(knn_indices.chunks_mut(chunk_size))
                .zip(points.chunks(chunk_size))
            {
                s.spawn(move || {
                    for ((d, k), p) in dist_chunk
                        .iter_mut()
                        .zip(knn_chunk.iter_mut())
                        .zip(points_chunk)
                    {
                        let knn = tree.nearest_neighbours(p, min_samples);
                        *d = if knn.len() >= min_samples {
                            knn[min_samples - 1].distance
                        } else {
                            T::zero()
                        };
                        *k = knn.iter().map(|nn| nn.index).collect();
                    }
                });
            }
        });
    }

    #[cfg(not(feature = "multithreading"))]
    {
        for ((d, k), p) in core_distances
            .iter_mut()
            .zip(knn_indices.iter_mut())
            .zip(points.iter())
        {
            let knn = tree.nearest_neighbours(p, min_samples);
            *d = if knn.len() >= min_samples {
                knn[min_samples - 1].distance
            } else {
                T::zero()
            };
            *k = knn.iter().map(|nn| nn.index).collect();
        }
    }

    CoreData {
        core_distances,
        knn_indices,
    }
}

// ---------------------------------------------------------------------------
// Step 2: Boruvka's MST on the mutual reachability graph
// ---------------------------------------------------------------------------

/// Data returned by `compute_core_data`: one entry per input point.
pub(crate) struct CoreData<T> {
    pub(crate) core_distances: Vec<T>,
    /// k-NN indices per point (length `min_samples`, includes self at
    /// distance 0). Retained for per-round seeding; freed when
    /// `CoreData` drops at the end of the HDBSCAN run.
    pub(crate) knn_indices: Vec<Vec<usize>>,
}

/// An edge in the minimum spanning tree.
#[derive(Debug, Clone)]
struct MstEdge<T> {
    u: usize,
    v: usize,
    weight: T,
}

/// The lightest outgoing edge found so far for a given component during a
/// Boruvka round.
#[derive(Debug, Clone, Copy)]
struct BestEdge<T> {
    from: usize,
    to: usize,
    mrd: T,
}

/// Build MST using dual-tree Boruvka's algorithm with ball tree acceleration.
///
/// The mutual reachability distance is:
///   mreach(i, j) = max(core_dist[i], core_dist[j], dist(i, j))
///
/// Each Boruvka round uses a dual-tree traversal over the ball tree to find,
/// for every component, the lightest outgoing edge under MRD. Two tree nodes
/// are pruned when: (a) all their points share the same component, or (b) the
/// node-to-node Euclidean lower bound exceeds the current best candidate for
/// the query subtree. Components halve each round, giving O(log n) rounds and
/// O(n log n) amortised total work.
fn boruvka_mst<T: GeoFloat>(
    tree: &BallTree<T>,
    points: &[Point<T>],
    core_data: &CoreData<T>,
) -> Vec<MstEdge<T>> {
    let n = points.len();
    if n <= 1 {
        return Vec::new();
    }

    let num_nodes = tree.node_count();
    let mut uf = UnionForest::new(n);
    let mut edges = Vec::with_capacity(n - 1);
    let mut point_components: Vec<usize> = (0..n).collect();
    let mut component_of_node: Vec<Option<usize>> = vec![None; num_nodes];
    update_component_of_node(tree, &point_components, &mut component_of_node);

    // Parallel tree-position arrays: position `i` corresponds to
    // `tree.node(0).points()[i]` and its original index is
    // `tree.node(0).indices()[i]`. Keeping core distances and component labels
    // in tree-position order turns process_leaves' inner loop into a
    // sequential cache-friendly walk instead of gathers through
    // `core_distances[indices[j]]` and `point_components[indices[j]]`.
    let tree_order: Vec<usize> = tree.node(0).indices().to_vec();
    debug_assert_eq!(tree_order.len(), n);
    let core_distances_pos: Vec<T> = tree_order
        .iter()
        .map(|&i| core_data.core_distances[i])
        .collect();
    let mut point_components_pos: Vec<usize> =
        tree_order.iter().map(|&i| point_components[i]).collect();

    let mut bounds = vec![T::infinity(); num_nodes];
    let mut best_edges: Vec<Option<BestEdge<T>>> = vec![None; n];

    // Seed best_edges + bounds at the top of every Boruvka round using the
    // stored k-NN data. In round 1, comp(p) = p so every cross-component
    // top-k neighbour yields a real edge seed. In rounds 2+, seeds degrade
    // gracefully as components grow: points whose top-k is entirely
    // intra-component contribute nothing, leaving the round to run with
    // the baseline infinity bound for those components. See
    // `docs/superpowers/specs/2026-04-16-hdbscan-bound-seeding-design.md`.
    loop {
        // No-op on round 1 (best_edges was freshly allocated with Nones);
        // clears accumulated candidates at the start of rounds 2+.
        best_edges.fill(None);
        seed_best_edges(
            &core_data.knn_indices,
            &core_data.core_distances,
            &point_components,
            &mut best_edges,
        );
        bounds.fill(T::infinity());
        seed_bounds(tree, &best_edges, &point_components_pos, &mut bounds);

        let mut state = DualTreeBoruvkaState {
            tree,
            core_distances_pos: &core_distances_pos,
            point_components_pos: &point_components_pos,
            component_of_node: &component_of_node,
            bounds: &mut bounds,
            best_edges: &mut best_edges,
        };
        state.traverse(0, 0);

        let mut merged_any = false;
        for edge in best_edges.iter().flatten() {
            if uf.union(edge.from, edge.to).is_some() {
                edges.push(MstEdge {
                    u: edge.from,
                    v: edge.to,
                    weight: edge.mrd,
                });
                merged_any = true;
            }
        }

        if !merged_any || edges.len() >= n - 1 {
            break;
        }

        for (i, comp) in point_components.iter_mut().enumerate() {
            *comp = uf.find(i);
        }
        for (pos, comp_pos) in point_components_pos.iter_mut().enumerate() {
            *comp_pos = point_components[tree_order[pos]];
        }
        update_component_of_node(tree, &point_components, &mut component_of_node);
    }

    edges
}

/// State for dual-tree Boruvka traversal.
///
/// Bundles the shared tree data (read-only during traversal) with the mutable
/// per-round state: `bounds` tracks the tightest MRD candidate for each tree
/// node's subtree, and `best_edges` tracks the lightest outgoing edge for each
/// component.
struct DualTreeBoruvkaState<'a, T: GeoFloat> {
    tree: &'a BallTree<T>,
    /// Core distances in tree-position order (parallel to
    /// `BallTreeNode::points()` within each leaf). Lets the inner loop read
    /// core_dist sequentially instead of gathering through
    /// `core_distances[indices[j]]`.
    core_distances_pos: &'a [T],
    /// Point-to-component mapping in tree-position order. Same rationale as
    /// `core_distances_pos`.
    point_components_pos: &'a [usize],
    component_of_node: &'a [Option<usize>],
    /// Per-node bound: the maximum candidate MRD across all points in the
    /// subtree. Used for distance-bound pruning. Allocated once, reset each
    /// Boruvka round.
    bounds: &'a mut [T],
    /// Per-component best outgoing edge. Indexed by component root (a point
    /// index in 0..n). Allocated once, reset each Boruvka round.
    best_edges: &'a mut [Option<BestEdge<T>>],
}

impl<T: GeoFloat> DualTreeBoruvkaState<'_, T> {
    /// Recursively traverse query and reference subtrees to find the best
    /// inter-component edge under MRD for each component.
    fn traverse(&mut self, query_idx: usize, ref_idx: usize) {
        // Same-component pruning: if every point in both subtrees belongs to
        // the same component, no inter-component edge exists.
        if let (Some(qc), Some(rc)) = (
            self.component_of_node[query_idx],
            self.component_of_node[ref_idx],
        ) && qc == rc
        {
            return;
        }

        // Distance-bound pruning: if the minimum Euclidean distance between
        // the two balls exceeds the best candidate for the query subtree,
        // no point in the reference subtree can improve any candidate.
        let node_dist = self.min_node_dist(query_idx, ref_idx);
        if node_dist >= self.bounds[query_idx] {
            return;
        }

        let q_kind = self.tree.node(query_idx).kind();
        let r_kind = self.tree.node(ref_idx).kind();

        match (q_kind, r_kind) {
            (NodeKind::Leaf, NodeKind::Leaf) => {
                self.process_leaves(query_idx, ref_idx);
            }
            (
                NodeKind::Leaf,
                NodeKind::Branch {
                    left: rl,
                    right: rr,
                },
            ) => {
                let dl = self.min_node_dist(query_idx, rl);
                let dr = self.min_node_dist(query_idx, rr);
                let (first, second) = if dl <= dr { (rl, rr) } else { (rr, rl) };
                self.traverse(query_idx, first);
                self.traverse(query_idx, second);
            }
            (
                NodeKind::Branch {
                    left: ql,
                    right: qr,
                },
                NodeKind::Leaf,
            ) => {
                let dl = self.min_node_dist(ql, ref_idx);
                let dr = self.min_node_dist(qr, ref_idx);
                let (first, second) = if dl <= dr { (ql, qr) } else { (qr, ql) };
                self.traverse(first, ref_idx);
                self.traverse(second, ref_idx);
                self.bounds[query_idx] = self.bounds[ql].max(self.bounds[qr]);
            }
            (
                NodeKind::Branch {
                    left: ql,
                    right: qr,
                },
                NodeKind::Branch {
                    left: rl,
                    right: rr,
                },
            ) => {
                let mut pairs = [
                    (ql, rl, self.min_node_dist(ql, rl)),
                    (ql, rr, self.min_node_dist(ql, rr)),
                    (qr, rl, self.min_node_dist(qr, rl)),
                    (qr, rr, self.min_node_dist(qr, rr)),
                ];
                pairs.sort_by(|a, b| a.2.total_cmp(&b.2));
                for &(q, r, _) in &pairs {
                    self.traverse(q, r);
                }
                self.bounds[query_idx] = self.bounds[ql].max(self.bounds[qr]);
            }
        }
    }

    /// Base case: process all point pairs from two leaf nodes.
    ///
    /// All per-point data is read from tree-position-ordered slices
    /// (`points()`, `core_distances_pos`, `point_components_pos`) so the
    /// inner loop walks contiguous memory instead of gathering through the
    /// `indices` permutation. Only the rare best-edge write and the
    /// pair-skip check use the original-index arrays.
    fn process_leaves(&mut self, query_idx: usize, ref_idx: usize) {
        let q_node = self.tree.node(query_idx);
        let r_node = self.tree.node(ref_idx);
        let q_range = q_node.position_range();
        let r_range = r_node.position_range();
        let q_points = q_node.points();
        let r_points = r_node.points();
        let q_orig = q_node.indices();
        let r_orig = r_node.indices();

        let q_core = &self.core_distances_pos[q_range.clone()];
        let r_core = &self.core_distances_pos[r_range.clone()];
        let q_comp_slice = &self.point_components_pos[q_range];
        let r_comp_slice = &self.point_components_pos[r_range];

        for i in 0..q_points.len() {
            let q_comp = q_comp_slice[i];
            let core_q = q_core[i];
            let q_coord = q_points[i].0;
            let q_oi = q_orig[i];

            // Per-point pruning: if core_q already exceeds the best candidate
            // for this component, no MRD involving q can improve it (since
            // mrd >= core_q).
            let q_best = self.best_edges[q_comp].map_or(T::infinity(), |e| e.mrd);
            if core_q > q_best {
                continue;
            }

            for j in 0..r_points.len() {
                let r_oi = r_orig[j];
                if q_oi == r_oi {
                    continue;
                }
                let r_comp = r_comp_slice[j];
                if q_comp == r_comp {
                    continue;
                }

                let core_r = r_core[j];
                // Per-point pruning on the reference side: if core_r already
                // exceeds the best candidate for q's component, skip.
                if core_r > q_best {
                    continue;
                }

                let d = dist_sq(q_coord, r_points[j].0).sqrt();
                let mrd = d.max(core_q).max(core_r);

                // The guard on q_comp is an optimisation: q_best is hoisted
                // outside the inner loop, so we can skip the function call
                // and array access entirely. An equivalent guard for r_comp
                // is not hoisted because r_comp changes every inner iteration.
                // update_best_edge performs its own internal comparison, so
                // the unguarded call for r_comp is correct, just slightly
                // less efficient.
                if mrd < q_best {
                    update_best_edge(self.best_edges, q_comp, q_oi, r_oi, mrd);
                }
                update_best_edge(self.best_edges, r_comp, r_oi, q_oi, mrd);
            }
        }

        // Update the bound for the query node: the maximum candidate MRD
        // across all points in this leaf.
        let max_mrd = q_comp_slice.iter().fold(T::neg_infinity(), |acc, &q_comp| {
            let candidate_mrd = self.best_edges[q_comp].map_or(T::infinity(), |e| e.mrd);
            acc.max(candidate_mrd)
        });
        if max_mrd < self.bounds[query_idx] {
            self.bounds[query_idx] = max_mrd;
        }
    }

    /// Lower bound on the Euclidean distance between any two points in two
    /// ball tree nodes.
    #[inline]
    fn min_node_dist(&self, a: usize, b: usize) -> T {
        self.tree.node(a).min_distance_to(&self.tree.node(b))
    }
}

/// Seed `best_edges` for a Boruvka round using the stored k-NN data.
///
/// For each point `p`, scans `p`'s top-k neighbours and keeps the tightest
/// cross-component MRD. The seed is the true MRD of the chosen edge
/// `(p, q)`, since by construction `dist(p, q) <= core(p)` for every
/// `q` in `p`'s top-k, so `max(core(p), core(q), dist(p, q))` collapses
/// to `max(core(p), core(q))`. `update_best_edge` arbitrates across
/// multiple points sharing a component. Points whose entire k-NN lies
/// inside `comp(p)` contribute no seed for this round, and the
/// corresponding component falls back to `best_edges[comp] = None`
/// (equivalent to `q_best = infinity` during traversal).
///
/// `knn_indices` and `point_components` are both indexed by original
/// point index (NOT tree position). `core_distances` is likewise
/// original-index order.
fn seed_best_edges<T: GeoFloat>(
    knn_indices: &[Vec<usize>],
    core_distances: &[T],
    point_components: &[usize],
    best_edges: &mut [Option<BestEdge<T>>],
) {
    debug_assert_eq!(knn_indices.len(), core_distances.len());
    debug_assert_eq!(point_components.len(), core_distances.len());
    debug_assert_eq!(best_edges.len(), core_distances.len());

    for (p, neighbours) in knn_indices.iter().enumerate() {
        let comp_p = point_components[p];
        let core_p = core_distances[p];
        let mut best: Option<(usize, T)> = None;
        for &q in neighbours {
            if q == p {
                continue;
            }
            if point_components[q] == comp_p {
                continue;
            }
            let mrd = core_p.max(core_distances[q]);
            // Ties go to the first-seen neighbour. This matches
            // `update_best_edge`'s strict-`<` convention, so every candidate
            // evaluated by the traversal will tie-break the same way.
            match best {
                Some((_, m)) if m <= mrd => {}
                _ => best = Some((q, mrd)),
            }
        }
        if let Some((q, mrd)) = best {
            update_best_edge(best_edges, comp_p, p, q, mrd);
        }
    }
}

/// Compute initial `bounds[node_idx]` for every tree node, bottom-up.
///
/// For a leaf node, the bound is the max over its points of
/// `best_edges[comp(p)].mrd`, or infinity if some point's component has no
/// seeded best edge. For an internal node, the bound is the max of its
/// children's bounds. See the design spec for the correctness argument.
///
/// Uses recursion. The ball tree's depth is O(log n) for balanced inputs, so
/// stack usage is bounded. If future balls produce pathologically deep trees,
/// convert to an explicit post-order traversal.
fn seed_bounds<T: GeoFloat>(
    tree: &BallTree<T>,
    best_edges: &[Option<BestEdge<T>>],
    point_components_pos: &[usize],
    bounds: &mut [T],
) {
    fn recurse<T: GeoFloat>(
        tree: &BallTree<T>,
        node_idx: usize,
        best_edges: &[Option<BestEdge<T>>],
        point_components_pos: &[usize],
        bounds: &mut [T],
    ) -> T {
        let node = tree.node(node_idx);
        let bound = match node.kind() {
            NodeKind::Leaf => {
                let range = node.position_range();
                let mut max_mrd = T::neg_infinity();
                for pos in range {
                    let comp = point_components_pos[pos];
                    let mrd = best_edges[comp].map_or(T::infinity(), |e| e.mrd);
                    if mrd > max_mrd {
                        max_mrd = mrd;
                    }
                }
                max_mrd
            }
            NodeKind::Branch { left, right } => {
                let lb = recurse(tree, left, best_edges, point_components_pos, bounds);
                let rb = recurse(tree, right, best_edges, point_components_pos, bounds);
                lb.max(rb)
            }
        };
        bounds[node_idx] = bound;
        bound
    }

    if !bounds.is_empty() {
        recurse(tree, 0, best_edges, point_components_pos, bounds);
    }
}

/// Update the best outgoing edge for a component if the given MRD is lower
/// than the current best.
#[inline]
fn update_best_edge<T: GeoFloat>(
    best: &mut [Option<BestEdge<T>>],
    comp: usize,
    from: usize,
    to: usize,
    mrd: T,
) {
    if best[comp].is_none_or(|existing| mrd < existing.mrd) {
        best[comp] = Some(BestEdge { from, to, mrd });
    }
}

/// Update the component-of-node array after components have been merged.
///
/// A node's component is set to a component ID when all points in its subtree
/// share the same component, or `None` when the subtree spans multiple
/// components. Nodes are processed in reverse index order (children before
/// parents) so that internal nodes can be computed from their children.
fn update_component_of_node<T: GeoFloat>(
    tree: &BallTree<T>,
    point_components: &[usize],
    component_of_node: &mut [Option<usize>],
) {
    for node_idx in (0..tree.node_count()).rev() {
        let node = tree.node(node_idx);
        match node.kind() {
            NodeKind::Leaf => {
                let mut comp = None;
                let mut all_same = true;
                for &idx in node.indices() {
                    let c = point_components[idx];
                    match comp {
                        None => comp = Some(c),
                        Some(prev) if prev != c => {
                            all_same = false;
                            break;
                        }
                        Some(_) => {} // Same component as before; continue
                    }
                }
                component_of_node[node_idx] = if all_same { comp } else { None };
            }
            NodeKind::Branch { left, right } => {
                let lc = component_of_node[left];
                let rc = component_of_node[right];
                component_of_node[node_idx] = match (lc, rc) {
                    (Some(l), Some(r)) if l == r => Some(l),
                    (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) | (None, None) => None,
                };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Union-Find (shared between Step 2 and Step 3)
// ---------------------------------------------------------------------------

/// Disjoint-set forest with path halving and union-by-size.
///
/// Tracks per-component size so that both Boruvka's MST construction
/// (which only needs size-aware find+union) and the single-linkage
/// dendrogram construction (which additionally tags each root with a
/// synthetic label -- see [`TreeUnionFind`]) can share the same
/// primitives.
struct UnionForest {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl UnionForest {
    fn new(n: usize) -> Self {
        UnionForest {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // path halving
            x = self.parent[x];
        }
        x
    }

    /// Union by size. Returns `Some((big_root, small_root))` when a merge
    /// happened, or `None` when `x` and `y` were already in the same
    /// component. The caller can use the returned pair to update any
    /// side-channel bookkeeping keyed on roots.
    fn union(&mut self, x: usize, y: usize) -> Option<(usize, usize)> {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return None;
        }
        let (big, small) = if self.size[rx] >= self.size[ry] {
            (rx, ry)
        } else {
            (ry, rx)
        };
        self.parent[small] = big;
        self.size[big] += self.size[small];
        Some((big, small))
    }

    fn size(&self, x: usize) -> usize {
        self.size[x]
    }
}

/// [`UnionForest`] with a synthetic per-component label, used by the
/// single-linkage dendrogram construction in [`label`].
struct TreeUnionFind {
    inner: UnionForest,
    /// Label for each component (initially the point index, updated on
    /// each merge to the fresh virtual-node id).
    component_label: Vec<usize>,
}

impl TreeUnionFind {
    fn new(n: usize) -> Self {
        TreeUnionFind {
            inner: UnionForest::new(n),
            component_label: (0..n).collect(),
        }
    }

    fn find(&mut self, x: usize) -> usize {
        self.inner.find(x)
    }

    fn union(&mut self, x: usize, y: usize, new_label: usize) {
        if let Some((big, _small)) = self.inner.union(x, y) {
            self.component_label[big] = new_label;
        }
    }

    fn size(&self, x: usize) -> usize {
        self.inner.size(x)
    }

    fn component_label(&self, x: usize) -> usize {
        self.component_label[x]
    }
}

// ---------------------------------------------------------------------------
// Step 3: Label -- build dendrogram from sorted MST edges
// ---------------------------------------------------------------------------

/// An identifier for a node in the single-linkage dendrogram.
///
/// The dendrogram uses two kinds of nodes: original input points and
/// virtual merged components. Internally they are encoded as a single
/// dense id where ids `0..n_points` are [`Point`](DendrogramNode::Point)
/// and ids `n_points..` are [`Virtual`](DendrogramNode::Virtual), with the
/// virtual id offset so that `Virtual(i)` corresponds to `dendrogram[i]`.
///
/// Using an explicit enum (rather than raw `usize` comparisons against
/// `n_points`) keeps the boundary arithmetic on one conversion site and
/// prevents point/virtual confusion at call sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DendrogramNode {
    /// An original input point, identified by its input index.
    Point(usize),
    /// A virtual merged component; the payload is the dendrogram entry
    /// index (i.e. the raw id minus `n_points`).
    Virtual(usize),
}

impl DendrogramNode {
    /// Convert a raw dendrogram id back into its tagged form.
    #[inline]
    fn classify(id: usize, n_points: usize) -> Self {
        if id < n_points {
            DendrogramNode::Point(id)
        } else {
            DendrogramNode::Virtual(id - n_points)
        }
    }

    /// Encode this node back into the dense raw-id space used downstream
    /// as condensed-tree cluster ids.
    #[inline]
    fn as_raw_id(self, n_points: usize) -> usize {
        match self {
            DendrogramNode::Point(p) => p,
            DendrogramNode::Virtual(i) => n_points + i,
        }
    }
}

/// A row in the dendrogram (single-linkage style).
///
/// When two components merge at distance `distance`, a new virtual node is
/// created. `left` and `right` identify the merged components (either
/// original points or previous virtual nodes). `size` is the combined
/// number of points.
#[derive(Debug, Clone)]
struct DendrogramEntry<T> {
    left: DendrogramNode,
    right: DendrogramNode,
    distance: T,
    size: usize,
}

/// Build a single-linkage dendrogram by sorting MST edges and merging
/// components using union-find.
fn label<T: GeoFloat>(mst: &[MstEdge<T>], n: usize) -> Vec<DendrogramEntry<T>> {
    // Sort edges by weight
    let mut sorted_indices: Vec<usize> = (0..mst.len()).collect();
    sorted_indices.sort_by(|&a, &b| mst[a].weight.total_cmp(&mst[b].weight));

    let mut uf = TreeUnionFind::new(n);
    let mut dendrogram = Vec::with_capacity(mst.len());

    for &idx in &sorted_indices {
        let edge = &mst[idx];
        let root_u = uf.find(edge.u);
        let root_v = uf.find(edge.v);

        if root_u == root_v {
            continue;
        }

        let size_u = uf.size(root_u);
        let size_v = uf.size(root_v);

        // The dendrogram refers to components by their raw id, which is
        // either an input point (< n) or a previously merged component
        // (>= n).
        let label_u = DendrogramNode::classify(uf.component_label(root_u), n);
        let label_v = DendrogramNode::classify(uf.component_label(root_v), n);

        let new_label = n + dendrogram.len();
        uf.union(root_u, root_v, new_label);

        dendrogram.push(DendrogramEntry {
            left: label_u,
            right: label_v,
            distance: edge.weight,
            size: size_u + size_v,
        });
    }

    dendrogram
}

// ---------------------------------------------------------------------------
// Step 4: Condense the dendrogram
// ---------------------------------------------------------------------------

/// An entry in the condensed tree.
///
/// Each entry represents a parent-child relationship in the condensed cluster
/// hierarchy. `lambda` = 1/distance at which the split/fall-out happens.
#[derive(Debug, Clone)]
struct CondensedEntry<T> {
    parent: usize,
    child: usize,
    lambda: T,
    child_size: usize,
}

/// Condense the single-linkage dendrogram by removing splits that produce
/// components smaller than `min_cluster_size`. Such small components have
/// their points "fall out" of the parent cluster individually instead.
fn condense_tree<T: GeoFloat>(
    dendrogram: &[DendrogramEntry<T>],
    min_cluster_size: usize,
) -> Vec<CondensedEntry<T>> {
    let n_points = dendrogram.len() + 1; // dendrogram has n-1 entries for n points
    let root = DendrogramNode::Virtual(n_points - 2); // the last virtual node

    let mut result = Vec::new();

    // `relabel` maps a dendrogram node to the condensed-tree cluster id
    // it now belongs to. Clusters that survive as their own node get a
    // fresh id from `next_cluster_id`; otherwise they inherit their
    // parent's id. Un-relabelled virtual nodes fall back to their raw
    // dense id (`n_points + index`), which is also what downstream cluster
    // stages (e.g. `get_stability`, `find_clusters`) use as their keys.
    let mut relabel: HashMap<DendrogramNode, usize> = HashMap::new();
    let mut next_cluster_id = n_points;

    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        let virt_idx = match node {
            // Leaves (original points) do not expand.
            DendrogramNode::Point(_) => continue,
            DendrogramNode::Virtual(i) => i,
        };

        let entry = &dendrogram[virt_idx];
        let lambda = if entry.distance > T::zero() {
            T::one() / entry.distance
        } else {
            T::infinity()
        };

        let left = entry.left;
        let right = entry.right;
        let left_size = node_size(left, dendrogram);
        let right_size = node_size(right, dendrogram);

        // Determine the parent label (may have been relabelled).
        let parent_label = *relabel.get(&node).unwrap_or(&node.as_raw_id(n_points));

        let left_big = left_size >= min_cluster_size;
        let right_big = right_size >= min_cluster_size;

        match (left_big, right_big) {
            (true, true) => {
                // True split: both children become new clusters.
                let left_label = next_cluster_id;
                next_cluster_id += 1;
                let right_label = next_cluster_id;
                next_cluster_id += 1;

                result.push(CondensedEntry {
                    parent: parent_label,
                    child: left_label,
                    lambda,
                    child_size: left_size,
                });
                result.push(CondensedEntry {
                    parent: parent_label,
                    child: right_label,
                    lambda,
                    child_size: right_size,
                });

                relabel.insert(left, left_label);
                relabel.insert(right, right_label);

                stack.push(left);
                stack.push(right);
            }
            (true, false) => {
                // Only left is big enough: it inherits the parent label.
                relabel.insert(left, parent_label);
                stack.push(left);

                // Right is too small: its points fall out.
                emit_fallout(right, parent_label, lambda, dendrogram, &mut result);
            }
            (false, true) => {
                // Only right is big enough: it inherits the parent label.
                relabel.insert(right, parent_label);
                stack.push(right);

                // Left is too small: its points fall out.
                emit_fallout(left, parent_label, lambda, dendrogram, &mut result);
            }
            (false, false) => {
                // Neither child is big enough: all points fall out.
                emit_fallout(left, parent_label, lambda, dendrogram, &mut result);
                emit_fallout(right, parent_label, lambda, dendrogram, &mut result);
            }
        }
    }

    result
}

/// Compute the number of points under a node.
fn node_size<T>(node: DendrogramNode, dendrogram: &[DendrogramEntry<T>]) -> usize {
    match node {
        DendrogramNode::Point(_) => 1,
        DendrogramNode::Virtual(i) => dendrogram[i].size,
    }
}

/// Emit individual point fall-out entries for all points under `node`.
///
/// Uses an explicit stack instead of recursion for stack safety on large
/// dendrograms.
fn emit_fallout<T: GeoFloat>(
    node: DendrogramNode,
    parent: usize,
    lambda: T,
    dendrogram: &[DendrogramEntry<T>],
    result: &mut Vec<CondensedEntry<T>>,
) {
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        match current {
            DendrogramNode::Point(p) => {
                result.push(CondensedEntry {
                    parent,
                    child: p,
                    lambda,
                    child_size: 1,
                });
            }
            DendrogramNode::Virtual(i) => {
                let entry = &dendrogram[i];
                stack.push(entry.left);
                stack.push(entry.right);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Step 5: Compute stability and select clusters
// ---------------------------------------------------------------------------

/// Compute the stability of each cluster in the condensed tree.
///
/// Stability = sum over all points that fell out of this cluster of
/// (lambda_point - lambda_birth), where lambda_birth is the lambda at which
/// the cluster was created (= the lambda of the entry where this cluster
/// appears as a child).
fn get_stability<T: GeoFloat>(condensed: &[CondensedEntry<T>]) -> HashMap<usize, T> {
    // Pass 1: Find lambda_birth for each cluster: the lambda of the entry
    // where the cluster first appears as a child. Must be fully built before
    // we can compute stability, since a parent's birth may appear anywhere
    // in the condensed list.
    let mut lambda_birth: HashMap<usize, T> = HashMap::new();
    for entry in condensed {
        if entry.child_size > 1 {
            lambda_birth.entry(entry.child).or_insert(entry.lambda);
        }
    }

    // Pass 2: Accumulate stability and ensure all clusters have entries.
    let mut stability: HashMap<usize, T> = HashMap::new();
    for entry in condensed {
        stability.entry(entry.parent).or_insert(T::zero());
        if entry.child_size > 1 {
            stability.entry(entry.child).or_insert(T::zero());
        } else {
            // Point falling out of its parent cluster
            let birth = lambda_birth
                .get(&entry.parent)
                .copied()
                .unwrap_or(T::zero());
            let s = stability.entry(entry.parent).or_insert(T::zero());
            *s = *s + (entry.lambda - birth).max(T::zero());
        }
    }

    stability
}

/// Select clusters using the excess-of-mass method.
///
/// Returns:
/// - `cluster_labels`: mapping from condensed-tree cluster IDs to flat
///   cluster IDs (0, 1, 2, ...)
/// - `is_cluster`: set of condensed-tree cluster IDs that are selected
fn find_clusters<T: GeoFloat>(
    condensed: &[CondensedEntry<T>],
    stability: &HashMap<usize, T>,
) -> (HashMap<usize, usize>, HashSet<usize>) {
    // Identify all cluster nodes (those that appear as parents, or as
    // children with size > 1). Initially all are selected.
    let mut is_cluster: HashSet<usize> = HashSet::new();
    for entry in condensed {
        is_cluster.insert(entry.parent);
        if entry.child_size > 1 {
            is_cluster.insert(entry.child);
        }
    }

    // Build parent->children mapping (clusters only)
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    for entry in condensed {
        if entry.child_size > 1 {
            children.entry(entry.parent).or_default().push(entry.child);
        }
    }

    // Find leaves (clusters with no cluster children)
    let leaves: Vec<usize> = is_cluster
        .iter()
        .filter(|&c| !children.contains_key(c))
        .copied()
        .collect();

    // Build reverse parent mapping: child_cluster -> parent_cluster
    let mut parent_of: HashMap<usize, usize> = HashMap::new();
    for entry in condensed {
        if entry.child_size > 1 {
            parent_of.insert(entry.child, entry.parent);
        }
    }

    // Bottom-up pass: propagate stability using topological order.
    // Each cluster is processed exactly once, after all its children,
    // by tracking the number of unprocessed children (in-degree).
    let mut total_stability: HashMap<usize, T> = stability.clone();

    // In-degree: number of cluster children not yet processed.
    let mut in_degree: HashMap<usize, usize> = HashMap::new();
    for (cluster, child_list) in &children {
        in_degree.insert(*cluster, child_list.len());
    }

    // Seed the queue with leaves (clusters with no children / in-degree 0).
    let mut queue: Vec<usize> = leaves;

    while let Some(cluster) = queue.pop() {
        if let Some(child_list) = children.get(&cluster) {
            // Sum of children's total stability
            let children_stability: T = child_list
                .iter()
                .map(|c| *total_stability.get(c).unwrap_or(&T::zero()))
                .fold(T::zero(), |acc, x| acc + x);

            let self_stability = *stability.get(&cluster).unwrap_or(&T::zero());

            if self_stability >= children_stability {
                // This cluster is more stable: keep it, deselect all descendants
                is_cluster.insert(cluster);
                deselect_descendants(cluster, &children, &mut is_cluster);
                total_stability.insert(cluster, self_stability);
            } else {
                // Children are more stable: deselect this cluster
                is_cluster.remove(&cluster);
                total_stability.insert(cluster, children_stability);
            }
        }
        // Leaves are always selected (already true in is_cluster)

        // Decrement parent's in-degree; enqueue when all children are done.
        if let Some(&parent) = parent_of.get(&cluster) {
            let deg = in_degree.get_mut(&parent).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push(parent);
            }
        }
    }

    // Assign sequential IDs to selected clusters
    let mut cluster_labels = HashMap::new();
    let mut selected: Vec<usize> = is_cluster.iter().copied().collect();
    selected.sort();
    for (id, c) in selected.into_iter().enumerate() {
        cluster_labels.insert(c, id);
    }

    (cluster_labels, is_cluster)
}

/// Deselect all descendant clusters using an explicit stack for stack safety,
/// matching the approach used in `emit_fallout`.
fn deselect_descendants(
    cluster: usize,
    children: &HashMap<usize, Vec<usize>>,
    is_cluster: &mut HashSet<usize>,
) {
    let mut stack = vec![cluster];
    while let Some(current) = stack.pop() {
        if let Some(child_list) = children.get(&current) {
            for &child in child_list {
                is_cluster.remove(&child);
                stack.push(child);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Step 6: Extract flat cluster labels
// ---------------------------------------------------------------------------

/// Assign each point to its nearest selected ancestor cluster.
fn extract_labels<T: GeoFloat>(
    condensed: &[CondensedEntry<T>],
    cluster_labels: &HashMap<usize, usize>,
    is_cluster: &HashSet<usize>,
    n: usize,
) -> Vec<Option<usize>> {
    // For each point, find the deepest selected cluster it belongs to.
    // A point belongs to a cluster if it appears (directly or transitively)
    // as a child_size==1 entry under that cluster.

    // Build a map: for each cluster, find its nearest selected ancestor
    let mut cluster_to_selected: HashMap<usize, Option<usize>> = HashMap::new();

    // Build parent mapping
    let mut parent_of: HashMap<usize, usize> = HashMap::new();
    for entry in condensed {
        if entry.child_size > 1 {
            parent_of.insert(entry.child, entry.parent);
        }
    }

    // Seed the cache with every selected cluster's own label.
    for &cluster in is_cluster {
        cluster_to_selected.insert(cluster, cluster_labels.get(&cluster).copied());
    }

    // Now assign labels to points. `child_size == 1` uniquely identifies a
    // point-fallout entry in the condensed tree: only `emit_fallout` creates
    // entries with size 1, and they always carry an original point index in
    // `child`. Cluster-child entries (from the `(true, true)` branch of
    // `condense_tree`) always have `child_size >= min_cluster_size >= 2`.
    let mut labels = vec![None; n];
    // Post-condition guard: every one of the `n` input points must appear
    // exactly once as a `child_size == 1` entry. Holds by construction
    // (`emit_fallout` walks every leaf of each subtree) but a debug assert
    // catches regressions where a condense-tree code path forgets to emit.
    #[cfg(debug_assertions)]
    let mut seen = vec![false; n];
    for entry in condensed {
        if entry.child_size == 1 {
            #[cfg(debug_assertions)]
            {
                debug_assert!(
                    entry.child < n,
                    "condensed-tree child index {} out of range (n = {n})",
                    entry.child
                );
                debug_assert!(
                    !seen[entry.child],
                    "point {} emitted twice in condensed tree",
                    entry.child
                );
                seen[entry.child] = true;
            }
            let label = find_selected_ancestor(
                entry.parent,
                &parent_of,
                &mut cluster_to_selected,
                is_cluster,
                cluster_labels,
            );
            labels[entry.child] = label;
        }
    }
    #[cfg(debug_assertions)]
    debug_assert!(
        seen.iter().all(|&b| b),
        "not every input point was emitted by the condensed tree"
    );

    labels
}

/// Walk up the condensed-tree parent chain from `start` until a cached entry,
/// a selected cluster, or the root is reached, then back-fill the cache for
/// every cluster visited. Iterative to avoid unbounded recursion on deep
/// hierarchies, matching the stack-based pattern in
/// [`emit_fallout`] and [`deselect_descendants`].
fn find_selected_ancestor(
    start: usize,
    parent_of: &HashMap<usize, usize>,
    cluster_to_selected: &mut HashMap<usize, Option<usize>>,
    is_cluster: &HashSet<usize>,
    cluster_labels: &HashMap<usize, usize>,
) -> Option<usize> {
    let mut chain = Vec::new();
    let mut current = start;
    let result = loop {
        if let Some(&cached) = cluster_to_selected.get(&current) {
            break cached;
        }
        chain.push(current);
        if is_cluster.contains(&current) {
            break cluster_labels.get(&current).copied();
        }
        match parent_of.get(&current) {
            Some(&parent) => current = parent,
            // Root: not selected, so points routed here are noise.
            None => break cluster_labels.get(&current).copied(),
        }
    };
    for c in chain {
        cluster_to_selected.insert(c, result);
    }
    result
}

// ---------------------------------------------------------------------------
// Step 7: GLOSH outlier scores
// ---------------------------------------------------------------------------

/// Compute GLOSH outlier scores.
///
/// For each point:
///   score = 1 - (lambda_point / lambda_max_cluster)
///
/// where lambda_point is the lambda at which the point fell out of its
/// cluster, and lambda_max_cluster is the maximum lambda of any point in
/// that cluster.
fn glosh<T: GeoFloat>(
    condensed: &[CondensedEntry<T>],
    labels: &[Option<usize>],
    n: usize,
) -> Vec<T> {
    // Record the lambda at which each point fell out of the condensed tree.
    // `child_size == 1` uniquely identifies point-fallout entries (see the
    // note in `extract_labels`).
    let mut point_lambda = vec![T::zero(); n];
    for entry in condensed {
        if entry.child_size == 1 {
            point_lambda[entry.child] = entry.lambda;
        }
    }

    // Find max lambda for each selected cluster (using the labels we already
    // computed in extract_labels, avoiding a redundant parent walk)
    let mut max_lambda: HashMap<usize, T> = HashMap::new();
    for (&lambda, &label) in point_lambda.iter().zip(labels.iter()) {
        if let Some(cluster) = label {
            let entry = max_lambda.entry(cluster).or_insert(T::zero());
            *entry = (*entry).max(lambda);
        }
    }

    // Compute scores
    labels
        .iter()
        .zip(point_lambda.iter())
        .map(|(&label, &lambda)| {
            if let Some(cluster) = label {
                let lambda_max = *max_lambda.get(&cluster).unwrap_or(&T::one());
                // When a cluster contains coincident points, zero-distance MST
                // edges produce `lambda = infinity` in `condense_tree`, which
                // would yield `1 - inf/inf = NaN` here. Treat non-finite
                // `lambda_max` as 'point at cluster core' and return zero.
                if lambda_max.is_finite() && lambda_max > T::zero() {
                    T::one() - lambda / lambda_max
                } else {
                    T::zero()
                }
            } else {
                // Noise points get score 1.0
                T::one()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
