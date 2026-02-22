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

use crate::algorithm::ball_tree::{BallTree, BuildBallTree, NodeKind, coord_distance_sq};
use crate::{GeoFloat, MultiPoint, Point};
use std::collections::{HashMap, HashSet};

/// Result of an HDBSCAN clustering run.
///
/// Contains both cluster labels and GLOSH outlier scores for every input point.
#[derive(Debug, Clone)]
pub struct HdbscanResult<T> {
    /// Cluster assignment for each input point, in input order.
    /// `Some(id)` for clustered points, `None` for noise.
    pub labels: Vec<Option<usize>>,

    /// GLOSH (Global-Local Outlier Scores from Hierarchies) outlier score for
    /// each input point, in the range [0, 1]. Higher values indicate stronger
    /// outliers.
    pub outlier_scores: Vec<T>,
}

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
/// An [`HdbscanResult`] containing cluster labels and GLOSH outlier scores.
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
/// let result = points.hdbscan(5);
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
    /// See the [trait-level documentation](Hdbscan) for details.
    fn hdbscan(&self, min_cluster_size: usize) -> HdbscanResult<T> {
        self.hdbscan_with_min_samples(min_cluster_size, min_cluster_size)
    }

    /// Perform HDBSCAN clustering with separate `min_cluster_size` and
    /// `min_samples` parameters.
    ///
    /// `min_samples` controls the number of neighbours (including the point
    /// itself) used to compute core distances. See the
    /// [trait-level documentation](Hdbscan) for details.
    fn hdbscan_with_min_samples(
        &self,
        min_cluster_size: usize,
        min_samples: usize,
    ) -> HdbscanResult<T>;
}

impl<T> Hdbscan<T> for MultiPoint<T>
where
    T: GeoFloat + Send + Sync,
{
    fn hdbscan_with_min_samples(
        &self,
        min_cluster_size: usize,
        min_samples: usize,
    ) -> HdbscanResult<T> {
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
    ) -> HdbscanResult<T> {
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
) -> HdbscanResult<T> {
    let n = points.len();

    if n == 0 || min_cluster_size < 2 || min_cluster_size > n {
        return HdbscanResult {
            labels: vec![None; n],
            outlier_scores: vec![T::one(); n],
        };
    }

    // Build ball tree once for both core distances and MST construction
    let tree = points.build_ball_tree();

    // 1. Compute core distances using ball tree k-NN
    let core_distances = compute_core_distances(&tree, points, min_samples);

    // 2. Build MST over the mutual reachability graph (dual-tree Boruvka)
    let mst = boruvka_mst(&tree, points, &core_distances);

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

    HdbscanResult {
        labels,
        outlier_scores,
    }
}

// ---------------------------------------------------------------------------
// Step 1: Core distances via ball tree k-NN
// ---------------------------------------------------------------------------

/// Compute the core distance for each point: the distance to its k-th nearest
/// neighbour (where k = min_samples, including the point itself).
///
/// Uses the ball tree for efficient spatial queries. The same tree is reused
/// for MST construction, avoiding building a second spatial index.
fn core_dist_for_point<T: GeoFloat>(tree: &BallTree<T>, p: &Point<T>, min_samples: usize) -> T {
    let knn = tree.nearest_neighbours(p, min_samples);
    if knn.len() >= min_samples {
        knn[min_samples - 1].distance
    } else {
        T::zero()
    }
}

fn compute_core_distances<T: GeoFloat + Send + Sync>(
    tree: &BallTree<T>,
    points: &[Point<T>],
    min_samples: usize,
) -> Vec<T> {
    #[cfg(feature = "multithreading")]
    {
        let num_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        let mut distances = vec![T::zero(); points.len()];
        let chunk_size = points.len().div_ceil(num_threads);

        std::thread::scope(|s| {
            for (chunk_distances, chunk_points) in distances
                .chunks_mut(chunk_size)
                .zip(points.chunks(chunk_size))
            {
                s.spawn(move || {
                    for (dist, point) in chunk_distances.iter_mut().zip(chunk_points) {
                        *dist = core_dist_for_point(tree, point, min_samples);
                    }
                });
            }
        });

        distances
    }

    #[cfg(not(feature = "multithreading"))]
    {
        points
            .iter()
            .map(|p| core_dist_for_point(tree, p, min_samples))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Step 2: Boruvka's MST on the mutual reachability graph
// ---------------------------------------------------------------------------

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
    core_distances: &[T],
) -> Vec<MstEdge<T>> {
    let n = points.len();
    if n <= 1 {
        return Vec::new();
    }

    let num_nodes = tree.nodes.len();
    let mut uf = BoruvkaUnionFind::new(n);
    let mut edges = Vec::with_capacity(n - 1);
    let mut point_components: Vec<usize> = (0..n).collect();
    let mut component_of_node: Vec<Option<usize>> = vec![None; num_nodes];
    update_component_of_node(tree, &point_components, &mut component_of_node);

    let mut bounds = vec![T::infinity(); num_nodes];
    let mut best_edges: Vec<Option<BestEdge<T>>> = vec![None; n];

    loop {
        bounds.fill(T::infinity());
        best_edges.fill(None);

        let mut state = DualTreeBoruvkaState {
            tree,
            points,
            core_distances,
            point_components: &point_components,
            component_of_node: &component_of_node,
            bounds: &mut bounds,
            best_edges: &mut best_edges,
        };
        state.traverse(0, 0);

        let mut merged_any = false;
        for edge in best_edges.iter().flatten() {
            let ru = uf.find(edge.from);
            let rv = uf.find(edge.to);
            if ru != rv {
                uf.union(ru, rv);
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
    points: &'a [Point<T>],
    core_distances: &'a [T],
    point_components: &'a [usize],
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
        ) {
            if qc == rc {
                return;
            }
        }

        // Distance-bound pruning: if the minimum Euclidean distance between
        // the two balls exceeds the best candidate for the query subtree,
        // no point in the reference subtree can improve any candidate.
        let node_dist = self.min_node_dist(query_idx, ref_idx);
        if node_dist >= self.bounds[query_idx] {
            return;
        }

        let q_kind = self.tree.nodes[query_idx].kind;
        let r_kind = self.tree.nodes[ref_idx].kind;

        match (&q_kind, &r_kind) {
            (NodeKind::Leaf, NodeKind::Leaf) => {
                self.process_leaves(query_idx, ref_idx);
            }
            (NodeKind::Leaf, NodeKind::Branch { left, right }) => {
                let (rl, rr) = (*left, *right);
                let dl = self.min_node_dist(query_idx, rl);
                let dr = self.min_node_dist(query_idx, rr);
                let (first, second) = if dl <= dr { (rl, rr) } else { (rr, rl) };
                self.traverse(query_idx, first);
                self.traverse(query_idx, second);
            }
            (NodeKind::Branch { left, right }, NodeKind::Leaf) => {
                let (ql, qr) = (*left, *right);
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
                let (ql, qr, rl, rr) = (*ql, *qr, *rl, *rr);
                let mut pairs = [
                    (ql, rl, self.min_node_dist(ql, rl)),
                    (ql, rr, self.min_node_dist(ql, rr)),
                    (qr, rl, self.min_node_dist(qr, rl)),
                    (qr, rr, self.min_node_dist(qr, rr)),
                ];
                pairs.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
                for &(q, r, _) in &pairs {
                    self.traverse(q, r);
                }
                self.bounds[query_idx] = self.bounds[ql].max(self.bounds[qr]);
            }
        }
    }

    /// Base case: process all point pairs from two leaf nodes.
    fn process_leaves(&mut self, query_idx: usize, ref_idx: usize) {
        let q_start = self.tree.nodes[query_idx].start;
        let q_end = self.tree.nodes[query_idx].end;
        let r_start = self.tree.nodes[ref_idx].start;
        let r_end = self.tree.nodes[ref_idx].end;

        for qi in q_start..q_end {
            let q_orig = self.tree.indices[qi];
            let q_comp = self.point_components[q_orig];
            let core_q = self.core_distances[q_orig];
            let q_coord = self.points[q_orig].0;

            // Per-point pruning: if core_q already exceeds the best candidate
            // for this component, no MRD involving q can improve it (since
            // mrd >= core_q).
            let q_best = self.best_edges[q_comp].map_or(T::infinity(), |e| e.mrd);
            if core_q > q_best {
                continue;
            }

            for ri in r_start..r_end {
                let r_orig = self.tree.indices[ri];
                if q_orig == r_orig {
                    continue;
                }
                let r_comp = self.point_components[r_orig];
                if q_comp == r_comp {
                    continue;
                }

                // Per-point pruning on the reference side: if core_r already
                // exceeds the best candidate for q's component, skip.
                if self.core_distances[r_orig] > q_best {
                    continue;
                }

                let d = coord_distance_sq(q_coord, self.points[r_orig].0).sqrt();
                let mrd = d.max(core_q).max(self.core_distances[r_orig]);

                // The guard on q_comp is an optimisation: q_best is hoisted
                // outside the inner loop, so we can skip the function call
                // and array access entirely. An equivalent guard for r_comp
                // is not hoisted because r_comp changes every inner iteration.
                // update_best_edge performs its own internal comparison, so
                // the unguarded call for r_comp is correct, just slightly
                // less efficient.
                if mrd < q_best {
                    update_best_edge(self.best_edges, q_comp, q_orig, r_orig, mrd);
                }
                update_best_edge(self.best_edges, r_comp, r_orig, q_orig, mrd);
            }
        }

        // Update the bound for the query node: the maximum candidate MRD
        // across all points in this leaf.
        let max_mrd = (q_start..q_end).fold(T::neg_infinity(), |acc, qi| {
            let q_orig = self.tree.indices[qi];
            let q_comp = self.point_components[q_orig];
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
        let na = &self.tree.nodes[a];
        let nb = &self.tree.nodes[b];
        let center_dist = coord_distance_sq(na.center, nb.center).sqrt();
        let d = center_dist - na.radius - nb.radius;
        d.max(T::zero())
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
    match best[comp] {
        Some(existing) if mrd >= existing.mrd => {
            // Current best is still better or equal; no update needed.
        }
        Some(_) | None => {
            best[comp] = Some(BestEdge { from, to, mrd });
        }
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
    for node_idx in (0..tree.nodes.len()).rev() {
        let node = &tree.nodes[node_idx];
        match &node.kind {
            NodeKind::Leaf => {
                let mut comp = None;
                let mut all_same = true;
                for i in node.start..node.end {
                    let c = point_components[tree.indices[i]];
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
                let lc = component_of_node[*left];
                let rc = component_of_node[*right];
                component_of_node[node_idx] = match (lc, rc) {
                    (Some(l), Some(r)) if l == r => Some(l),
                    (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) | (None, None) => None,
                };
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Step 3: Label -- build dendrogram from sorted MST edges
// ---------------------------------------------------------------------------

/// A row in the dendrogram (single-linkage style).
///
/// When two components merge at distance `distance`, a new virtual node is
/// created. `left` and `right` are the IDs of the merged components (either
/// original point indices or previous virtual node IDs). `size` is the
/// combined number of points.
#[derive(Debug, Clone)]
struct DendrogramEntry<T> {
    left: usize,
    right: usize,
    distance: T,
    size: usize,
}

/// Build a single-linkage dendrogram by sorting MST edges and merging
/// components using union-find.
fn label<T: GeoFloat>(mst: &[MstEdge<T>], n: usize) -> Vec<DendrogramEntry<T>> {
    // Sort edges by weight
    let mut sorted_indices: Vec<usize> = (0..mst.len()).collect();
    sorted_indices.sort_by(|&a, &b| {
        mst[a]
            .weight
            .partial_cmp(&mst[b].weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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

        // The dendrogram refers to components by their "label":
        // - original points are labelled 0..n
        // - merged components are labelled n, n+1, ...
        let label_u = uf.component_label(root_u);
        let label_v = uf.component_label(root_v);

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
    let root = 2 * n_points - 2; // the last virtual node

    // Build children lookup from dendrogram
    // Virtual node k corresponds to dendrogram[k - n_points]
    let mut result = Vec::new();

    // BFS/DFS through the dendrogram
    // Stack entries: (node_id, is_virtual)
    // A virtual node i has children dendrogram[i - n_points].left and .right
    // We need to "relabel" clusters as we skip small splits.
    let mut relabel = HashMap::new();
    let mut next_cluster_id = n_points;

    // Use a stack for iterative DFS
    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        if node < n_points {
            // This is a leaf (original point) -- nothing to expand
            continue;
        }

        let entry = &dendrogram[node - n_points];
        let lambda = if entry.distance > T::zero() {
            T::one() / entry.distance
        } else {
            T::infinity()
        };

        let left = entry.left;
        let right = entry.right;
        let left_size = node_size(left, dendrogram, n_points);
        let right_size = node_size(right, dendrogram, n_points);

        // Determine the parent label (may have been relabelled)
        let parent_label = *relabel.get(&node).unwrap_or(&node);

        let left_big = left_size >= min_cluster_size;
        let right_big = right_size >= min_cluster_size;

        match (left_big, right_big) {
            (true, true) => {
                // True split: both children become new clusters
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

                // Map the original dendrogram node IDs to new cluster labels
                relabel.insert(left, left_label);
                relabel.insert(right, right_label);

                stack.push(left);
                stack.push(right);
            }
            (true, false) => {
                // Only left is big enough: it inherits the parent label
                relabel.insert(left, parent_label);
                stack.push(left);

                // Right is too small: its points fall out
                emit_fallout(
                    right,
                    parent_label,
                    lambda,
                    dendrogram,
                    n_points,
                    &mut result,
                );
            }
            (false, true) => {
                // Only right is big enough: it inherits the parent label
                relabel.insert(right, parent_label);
                stack.push(right);

                // Left is too small: its points fall out
                emit_fallout(
                    left,
                    parent_label,
                    lambda,
                    dendrogram,
                    n_points,
                    &mut result,
                );
            }
            (false, false) => {
                // Neither child is big enough: all points fall out
                emit_fallout(
                    left,
                    parent_label,
                    lambda,
                    dendrogram,
                    n_points,
                    &mut result,
                );
                emit_fallout(
                    right,
                    parent_label,
                    lambda,
                    dendrogram,
                    n_points,
                    &mut result,
                );
            }
        }
    }

    result
}

/// Compute the number of points under a node.
fn node_size<T>(node: usize, dendrogram: &[DendrogramEntry<T>], n_points: usize) -> usize {
    if node < n_points {
        1
    } else {
        dendrogram[node - n_points].size
    }
}

/// Emit individual point fall-out entries for all points under `node`.
///
/// Uses an explicit stack instead of recursion for stack safety on large
/// dendrograms.
fn emit_fallout<T: GeoFloat>(
    node: usize,
    parent: usize,
    lambda: T,
    dendrogram: &[DendrogramEntry<T>],
    n_points: usize,
    result: &mut Vec<CondensedEntry<T>>,
) {
    let mut stack = vec![node];
    while let Some(current) = stack.pop() {
        if current < n_points {
            result.push(CondensedEntry {
                parent,
                child: current,
                lambda,
                child_size: 1,
            });
        } else {
            let entry = &dendrogram[current - n_points];
            stack.push(entry.left);
            stack.push(entry.right);
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

    // For each cluster, walk up to find the nearest selected ancestor
    for &cluster in is_cluster {
        cluster_to_selected.insert(cluster, cluster_labels.get(&cluster).copied());
    }

    // For non-selected clusters, find their nearest selected ancestor
    fn find_selected_ancestor(
        cluster: usize,
        parent_of: &HashMap<usize, usize>,
        cluster_to_selected: &mut HashMap<usize, Option<usize>>,
        is_cluster: &HashSet<usize>,
        cluster_labels: &HashMap<usize, usize>,
    ) -> Option<usize> {
        if let Some(&cached) = cluster_to_selected.get(&cluster) {
            return cached;
        }

        if is_cluster.contains(&cluster) {
            let label = cluster_labels.get(&cluster).copied();
            cluster_to_selected.insert(cluster, label);
            return label;
        }

        if let Some(&parent) = parent_of.get(&cluster) {
            let result = find_selected_ancestor(
                parent,
                parent_of,
                cluster_to_selected,
                is_cluster,
                cluster_labels,
            );
            cluster_to_selected.insert(cluster, result);
            result
        } else {
            // This is the root; if not selected, points are noise
            let label = cluster_labels.get(&cluster).copied();
            cluster_to_selected.insert(cluster, label);
            label
        }
    }

    // Now assign labels to points
    let mut labels = vec![None; n];
    for entry in condensed {
        if entry.child_size == 1 && entry.child < n {
            // This point falls out of entry.parent at lambda
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

    labels
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
    // Record the lambda at which each point fell out of the condensed tree
    let mut point_lambda = vec![T::zero(); n];
    for entry in condensed {
        if entry.child_size == 1 && entry.child < n {
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
                if lambda_max > T::zero() {
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

// ---------------------------------------------------------------------------
// Union-Find
// ---------------------------------------------------------------------------

/// Simple union-find for Boruvka's MST construction.
struct BoruvkaUnionFind {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl BoruvkaUnionFind {
    fn new(n: usize) -> Self {
        BoruvkaUnionFind {
            parent: (0..n).collect(),
            size: vec![1; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }
        let (big, small) = if self.size[rx] >= self.size[ry] {
            (rx, ry)
        } else {
            (ry, rx)
        };
        self.parent[small] = big;
        self.size[big] += self.size[small];
    }
}

/// Union-find data structure with path compression, union-by-size, and
/// component labelling for the dendrogram construction.
struct TreeUnionFind {
    parent: Vec<usize>,
    size: Vec<usize>,
    /// Label for each component (initially the point index, updated on merge).
    component_label: Vec<usize>,
}

impl TreeUnionFind {
    fn new(n: usize) -> Self {
        TreeUnionFind {
            parent: (0..n).collect(),
            size: vec![1; n],
            component_label: (0..n).collect(),
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // path halving
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, x: usize, y: usize, new_label: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }

        // Union by size
        let (big, small) = if self.size[rx] >= self.size[ry] {
            (rx, ry)
        } else {
            (ry, rx)
        };

        self.parent[small] = big;
        self.size[big] += self.size[small];
        self.component_label[big] = new_label;
    }

    fn size(&self, x: usize) -> usize {
        self.size[x]
    }

    fn component_label(&self, x: usize) -> usize {
        self.component_label[x]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point;

    #[test]
    fn test_empty_input() {
        let points: Vec<Point<f64>> = vec![];
        let result = points.hdbscan(5);
        assert!(result.labels.is_empty());
        assert!(result.outlier_scores.is_empty());
    }

    #[test]
    fn test_single_point() {
        let points = vec![point!(x: 0.0, y: 0.0)];
        let result = points.hdbscan(2);
        assert_eq!(result.labels, vec![None]);
    }

    #[test]
    fn test_min_cluster_size_too_large() {
        let points = vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 1.0, y: 0.0),
            point!(x: 0.0, y: 1.0),
        ];
        // min_cluster_size larger than point count -- everything is noise
        let result = points.hdbscan(100);
        assert!(result.labels.iter().all(|l| l.is_none()));
    }

    /// Verify that the MultiPoint trait impl delegates correctly and
    /// produces the same result as calling on the underlying slice.
    #[test]
    fn test_multipoint_trait_impl() {
        let mp = MultiPoint::new(vec![
            point!(x: 0.0, y: 0.0),
            point!(x: 0.1, y: 0.0),
            point!(x: 0.0, y: 0.1),
            point!(x: 0.1, y: 0.1),
            point!(x: 0.05, y: 0.05),
        ]);

        let result_mp = mp.hdbscan(3);
        let result_slice = mp.0.hdbscan(3);

        assert_eq!(result_mp.labels.len(), 5);
        assert_eq!(result_mp.labels, result_slice.labels);
    }

    #[test]
    fn test_identical_points() {
        let points: Vec<Point<f64>> = (0..10).map(|_| point!(x: 5.0, y: 5.0)).collect();
        let result = points.hdbscan(3);
        assert_eq!(result.labels.len(), 10);
        // Should not panic; either all clustered or all noise is acceptable
    }

    #[test]
    fn test_min_cluster_size_one() {
        // min_cluster_size < 2 is treated as invalid: all points are noise
        let points = vec![point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
        let result = points.hdbscan(1);
        assert!(result.labels.iter().all(|l| l.is_none()));
    }

    #[test]
    fn test_two_points() {
        let points = vec![point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
        let result = points.hdbscan(2);
        assert_eq!(result.labels.len(), 2);
        // With only 2 points and min_cluster_size=2, both should be in the
        // same cluster or both noise
        if result.labels[0].is_some() {
            assert_eq!(result.labels[0], result.labels[1]);
        }
    }

    #[test]
    fn test_three_clusters() {
        let mut points = Vec::new();

        // Cluster A: 10 points near origin
        for i in 0..10 {
            let x = (i % 5) as f64 * 0.1;
            let y = (i / 5) as f64 * 0.1;
            points.push(Point::new(x, y));
        }

        // Cluster B: 10 points far away
        for i in 0..10 {
            let x = 10.0 + (i % 5) as f64 * 0.1;
            let y = 10.0 + (i / 5) as f64 * 0.1;
            points.push(Point::new(x, y));
        }

        // Cluster C: 10 points far away in another direction
        for i in 0..10 {
            let x = 20.0 + (i % 5) as f64 * 0.1;
            let y = 0.0 + (i / 5) as f64 * 0.1;
            points.push(Point::new(x, y));
        }

        // min_cluster_size=6 ensures no sub-split within a 10-point cluster
        // can produce two children both >= 6, preventing internal over-splitting.
        let result = points.hdbscan(6);

        // Each group should be a distinct cluster
        let label_a = result.labels[0];
        let label_b = result.labels[10];
        let label_c = result.labels[20];
        assert!(label_a.is_some(), "cluster A should be found");
        assert!(label_b.is_some(), "cluster B should be found");
        assert!(label_c.is_some(), "cluster C should be found");

        // All three labels should be distinct
        assert_ne!(label_a, label_b, "A and B should differ");
        assert_ne!(label_a, label_c, "A and C should differ");
        assert_ne!(label_b, label_c, "B and C should differ");

        // All points within each group should share their cluster label
        for i in 0..10 {
            assert_eq!(
                result.labels[i], label_a,
                "point {i} should be in cluster A"
            );
        }
        for i in 10..20 {
            assert_eq!(
                result.labels[i], label_b,
                "point {i} should be in cluster B"
            );
        }
        for i in 20..30 {
            assert_eq!(
                result.labels[i], label_c,
                "point {i} should be in cluster C"
            );
        }
    }

    // -- Union-Find unit tests ------------------------------------------------

    #[test]
    fn test_union_find_basic() {
        let mut uf = TreeUnionFind::new(5);
        assert_eq!(uf.find(0), 0);
        assert_eq!(uf.find(4), 4);
        assert_eq!(uf.size(0), 1);

        uf.union(0, 1, 5);
        assert_eq!(uf.find(0), uf.find(1));
        let root = uf.find(0);
        assert_eq!(uf.size(root), 2);
        assert_eq!(uf.component_label(root), 5);
    }

    #[test]
    fn test_union_find_chain() {
        let mut uf = TreeUnionFind::new(4);
        uf.union(0, 1, 4);
        uf.union(2, 3, 5);
        uf.union(0, 2, 6);
        let root = uf.find(0);
        assert_eq!(uf.find(1), root);
        assert_eq!(uf.find(2), root);
        assert_eq!(uf.find(3), root);
        assert_eq!(uf.size(root), 4);
        assert_eq!(uf.component_label(root), 6);
    }

    // -- Known-answer tests (ported from petal-clustering) --------------------

    /// Dataset from petal-clustering's `partial_labels` test (unsupervised).
    ///
    /// Three spatial groups plus one isolated noise point:
    ///   Group 1 (5 pts): indices 0..5, near (1-3, 7-9)
    ///   Group 2 (4 pts): indices 5..9, near (5-6, 3-4)
    ///   Group 3 (6 pts): indices 9..15, near (8-9, 1-3)
    ///   Outlier:         index 15, at (7, 8)
    ///
    /// petal-clustering produces 2 clusters by merging groups 2+3 (because
    /// its label function groups tied edges, creating a flatter hierarchy).
    /// Our implementation produces 3 clusters because the binary dendrogram
    /// gives each group enough stability to survive on its own. Both are
    /// valid HDBSCAN results; the difference is in tied-edge handling.
    #[test]
    fn test_known_answer_petal_clustering() {
        let points = vec![
            // Group 1
            Point::new(1.0, 9.0),
            Point::new(2.0, 9.0),
            Point::new(1.0, 8.0),
            Point::new(2.0, 8.0),
            Point::new(3.0, 7.0),
            // Group 2
            Point::new(5.0, 4.0),
            Point::new(6.0, 4.0),
            Point::new(5.0, 3.0),
            Point::new(6.0, 3.0),
            // Group 3
            Point::new(8.0, 3.0),
            Point::new(9.0, 3.0),
            Point::new(8.0, 2.0),
            Point::new(9.0, 2.0),
            Point::new(8.0, 1.0),
            Point::new(9.0, 1.0),
            // Outlier
            Point::new(7.0, 8.0),
        ];
        let result = points.hdbscan_with_min_samples(4, 4);

        // Group 1 (indices 0..5) should share a label
        let label_g1 = result.labels[0];
        assert!(label_g1.is_some(), "group 1 should be clustered");
        for i in 0..5 {
            assert_eq!(result.labels[i], label_g1, "point {i} should be in group 1");
        }

        // Group 2 (indices 5..9) should share a label
        let label_g2 = result.labels[5];
        assert!(label_g2.is_some(), "group 2 should be clustered");
        for i in 5..9 {
            assert_eq!(result.labels[i], label_g2, "point {i} should be in group 2");
        }

        // Group 3 (indices 9..15) should share a label
        let label_g3 = result.labels[9];
        assert!(label_g3.is_some(), "group 3 should be clustered");
        for i in 9..15 {
            assert_eq!(result.labels[i], label_g3, "point {i} should be in group 3");
        }

        // All three groups should have distinct labels
        assert_ne!(label_g1, label_g2, "groups 1 and 2 should differ");
        assert_ne!(label_g1, label_g3, "groups 1 and 3 should differ");
        assert_ne!(label_g2, label_g3, "groups 2 and 3 should differ");

        // Point 15 is an outlier: it should have a higher GLOSH score
        // than the average cluster member
        let cluster_avg: f64 = result.outlier_scores[0..15].iter().sum::<f64>() / 15.0;
        assert!(
            result.outlier_scores[15] > cluster_avg,
            "outlier score ({}) should exceed cluster average ({cluster_avg})",
            result.outlier_scores[15]
        );
    }

    /// Dataset from petal-clustering's `outlier_scores` test.
    ///
    /// Three clusters plus two outlier points. The outliers should have
    /// higher GLOSH scores than core cluster members, regardless of whether
    /// they are labelled as noise or absorbed into the nearest cluster.
    #[test]
    fn test_known_answer_glosh_scores() {
        let points = vec![
            // Cluster A (8 points)
            Point::new(2.0, 9.0),
            Point::new(3.0, 9.0),
            Point::new(2.0, 8.0),
            Point::new(3.0, 8.0),
            Point::new(2.0, 7.0),
            Point::new(3.0, 7.0),
            Point::new(1.0, 8.0),
            Point::new(4.0, 8.0),
            // Cluster B (5 points)
            Point::new(7.0, 9.0),
            Point::new(7.0, 8.0),
            Point::new(8.0, 8.0),
            Point::new(8.0, 7.0),
            Point::new(9.0, 7.0),
            // Cluster C (5 points)
            Point::new(6.0, 3.0),
            Point::new(5.0, 2.0),
            Point::new(6.0, 2.0),
            Point::new(7.0, 2.0),
            Point::new(6.0, 1.0),
            // Outliers
            Point::new(8.0, 4.0), // index 18
            Point::new(3.0, 3.0), // index 19
        ];
        let result = points.hdbscan_with_min_samples(5, 5);

        // All scores should be in [0, 1]
        for (i, &score) in result.outlier_scores.iter().enumerate() {
            assert!(
                score >= 0.0 && score <= 1.0,
                "outlier score for point {i} is {score}, expected [0, 1]"
            );
        }

        // Outlier points should have higher scores than cluster core members.
        // Compute average score for cluster A (indices 0..8) as a reference
        // for "core" cluster behaviour.
        let outlier1_score = result.outlier_scores[18];
        let outlier2_score = result.outlier_scores[19];
        let cluster_a_avg: f64 = result.outlier_scores[0..8].iter().sum::<f64>() / 8.0;

        assert!(
            outlier1_score > cluster_a_avg,
            "outlier1 score ({outlier1_score}) should exceed cluster A average ({cluster_a_avg})"
        );
        assert!(
            outlier2_score > cluster_a_avg,
            "outlier2 score ({outlier2_score}) should exceed cluster A average ({cluster_a_avg})"
        );

        // The three clusters should be found
        let label_a = result.labels[0];
        assert!(label_a.is_some(), "cluster A should be found");
        let a_consistent = (0..8).all(|i| result.labels[i] == label_a);
        assert!(a_consistent, "cluster A should be internally consistent");
    }

    // -- Clusters with noise --------------------------------------------------

    /// Two clusters with distant outlier points.
    ///
    /// Points far from all clusters merge at the top of the hierarchy
    /// and get classified as noise when the root is deselected in favour
    /// of its child clusters.
    #[test]
    fn test_clusters_with_distant_noise() {
        let mut points = Vec::new();

        // Dense cluster A near origin (10 points in a 1x1 area)
        for i in 0..10 {
            let x = (i % 5) as f64 * 0.2;
            let y = (i / 5) as f64 * 0.2;
            points.push(Point::new(x, y));
        }

        // Dense cluster B far away (10 points in a 1x1 area)
        for i in 0..10 {
            let x = 20.0 + (i % 5) as f64 * 0.2;
            let y = 20.0 + (i / 5) as f64 * 0.2;
            points.push(Point::new(x, y));
        }

        // Distant outlier -- far from both clusters, merges at the
        // very top of the hierarchy
        points.push(Point::new(100.0, 100.0)); // idx 20

        let result = points.hdbscan(5);

        // Both clusters should be found
        let label_a = result.labels[0];
        let label_b = result.labels[10];
        assert!(label_a.is_some(), "cluster A should be found");
        assert!(label_b.is_some(), "cluster B should be found");
        assert_ne!(label_a, label_b);

        // Cluster membership should be consistent
        for i in 0..10 {
            assert_eq!(
                result.labels[i], label_a,
                "point {i} should be in cluster A"
            );
        }
        for i in 10..20 {
            assert_eq!(
                result.labels[i], label_b,
                "point {i} should be in cluster B"
            );
        }

        // Distant outlier should be noise
        assert_eq!(result.labels[20], None, "distant outlier should be noise");
        // And should have a high outlier score
        assert!(
            result.outlier_scores[20] > 0.5,
            "distant outlier score ({}) should be high",
            result.outlier_scores[20]
        );
    }

    /// Points between two clusters get absorbed into the nearest cluster
    /// (correct HDBSCAN behaviour) but should have higher outlier scores.
    #[test]
    fn test_absorbed_bridge_points_have_high_scores() {
        let mut points = Vec::new();

        // Dense cluster A (10 points)
        for i in 0..10 {
            let x = (i % 5) as f64 * 0.2;
            let y = (i / 5) as f64 * 0.2;
            points.push(Point::new(x, y));
        }

        // Dense cluster B (10 points)
        for i in 0..10 {
            let x = 20.0 + (i % 5) as f64 * 0.2;
            let y = 20.0 + (i / 5) as f64 * 0.2;
            points.push(Point::new(x, y));
        }

        // Bridge points between the clusters
        points.push(Point::new(5.0, 5.0)); // idx 20
        points.push(Point::new(10.0, 10.0)); // idx 21
        points.push(Point::new(15.0, 15.0)); // idx 22

        let result = points.hdbscan(5);

        // Both clusters should be found
        let label_a = result.labels[0];
        let label_b = result.labels[10];
        assert!(label_a.is_some(), "cluster A should be found");
        assert!(label_b.is_some(), "cluster B should be found");
        assert_ne!(label_a, label_b);

        // Bridge points are absorbed into the nearest cluster; their
        // outlier scores should be higher than the average cluster member.
        let cluster_avg: f64 = result.outlier_scores[0..20].iter().sum::<f64>() / 20.0;
        for i in 20..23 {
            assert!(
                result.outlier_scores[i] >= cluster_avg,
                "bridge point {i} score ({}) should be >= cluster avg ({cluster_avg})",
                result.outlier_scores[i]
            );
        }
    }

    // -- Internal pipeline unit tests -----------------------------------------

    /// Test boruvka_mst on the 7-point dataset from petal-clustering.
    ///
    /// Points and core distances are chosen so the MST is unambiguous.
    /// We verify the total weight rather than edge ordering (which varies
    /// between Boruvka implementations).
    #[test]
    fn test_boruvka_mst_total_weight() {
        let points = vec![
            Point::new(0.0, 0.0),    // 0
            Point::new(7.0, 0.0),    // 1
            Point::new(15.0, 0.0),   // 2
            Point::new(0.0, -5.0),   // 3
            Point::new(15.0, -5.0),  // 4
            Point::new(7.0, -7.0),   // 5
            Point::new(15.0, -14.0), // 6
        ];
        let core_distances = vec![5.0, 7.0, 5.0, 5.0, 5.0, 6.0, 9.0];
        let tree = points.build_ball_tree();

        let mst = boruvka_mst(&tree, &points, &core_distances);

        // MST should have n-1 = 6 edges
        assert_eq!(mst.len(), 6, "MST should have n-1 edges");

        // Expected MRD edges:
        //   (0,3): max(5,5,5) = 5
        //   (2,4): max(5,5,5) = 5
        //   (0,1): max(5,7,7) = 7
        //   (1,5): max(7,6,7) = 7
        //   (1,2): max(7,5,8) = 8
        //   (4,6): max(5,9,9) = 9
        // Total = 41
        let total_weight: f64 = mst.iter().map(|e| e.weight).sum();
        assert!(
            (total_weight - 41.0).abs() < 1e-10,
            "MST total weight should be 41.0, got {total_weight}"
        );
    }

    /// Test that the MST connects all points (every point reachable).
    #[test]
    fn test_boruvka_mst_connectivity() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(7.0, 0.0),
            Point::new(15.0, 0.0),
            Point::new(0.0, -5.0),
            Point::new(15.0, -5.0),
            Point::new(7.0, -7.0),
            Point::new(15.0, -14.0),
        ];
        let core_distances = vec![5.0, 7.0, 5.0, 5.0, 5.0, 6.0, 9.0];
        let tree = points.build_ball_tree();

        let mst = boruvka_mst(&tree, &points, &core_distances);

        // Check connectivity via union-find
        let mut uf = BoruvkaUnionFind::new(points.len());
        for edge in &mst {
            let ru = uf.find(edge.u);
            let rv = uf.find(edge.v);
            uf.union(ru, rv);
        }
        let root = uf.find(0);
        for i in 1..points.len() {
            assert_eq!(uf.find(i), root, "point {i} should be connected to point 0");
        }
    }

    /// Test the dendrogram (label step) with a simple 4-point example.
    ///
    /// Points form two pairs: {0,1} close together, {2,3} close together,
    /// with the pairs far apart. The MST should merge each pair first,
    /// then merge the two pairs.
    #[test]
    fn test_label_dendrogram() {
        // Manually constructed MST:
        //   (0,1) at weight 1.0
        //   (2,3) at weight 1.0
        //   (1,2) at weight 10.0
        let mst = vec![
            MstEdge {
                u: 0,
                v: 1,
                weight: 1.0,
            },
            MstEdge {
                u: 2,
                v: 3,
                weight: 1.0,
            },
            MstEdge {
                u: 1,
                v: 2,
                weight: 10.0,
            },
        ];

        let dendrogram = label(&mst, 4);

        // Should have 3 entries (n-1 merges)
        assert_eq!(dendrogram.len(), 3);

        // First two merges at distance 1.0 (pairs), final merge at 10.0
        // The order of the first two is implementation-defined (both at
        // distance 1.0), but both should have size 2. The final merge
        // should have size 4.
        let sizes: Vec<usize> = dendrogram.iter().map(|e| e.size).collect();
        let distances: Vec<f64> = dendrogram.iter().map(|e| e.distance).collect();

        // First two merges create size-2 clusters
        assert_eq!(sizes[0], 2);
        assert_eq!(sizes[1], 2);
        assert_eq!(distances[0], 1.0);
        assert_eq!(distances[1], 1.0);

        // Final merge creates the root with all 4 points
        assert_eq!(sizes[2], 4);
        assert_eq!(distances[2], 10.0);
    }

    /// Test condense_tree with a hand-crafted dendrogram.
    ///
    /// Dendrogram for 7 points (adapted from petal-clustering's condense_mst
    /// test):
    ///
    ///             node 12
    ///           /          \           distance = 8.0
    ///       node 10      node 11
    ///       /    \        /    \       distance = 4.0
    ///    node 7  node 8  node 9  pt 6
    ///    /  \    /  \    /  \          distance = 2.0
    ///   0    1  2    3  4    5
    ///
    /// With min_cluster_size=3, nodes 7, 8, 9 (size 2 each) are too small
    /// to be clusters. The condensed tree should have two child clusters
    /// under the root, with their points falling out individually.
    #[test]
    fn test_condense_tree_structure() {
        let dendrogram: Vec<DendrogramEntry<f64>> = vec![
            // entry 0 -> node 7: merge points 0,1
            DendrogramEntry {
                left: 0,
                right: 1,
                distance: 2.0,
                size: 2,
            },
            // entry 1 -> node 8: merge points 2,3
            DendrogramEntry {
                left: 2,
                right: 3,
                distance: 2.0,
                size: 2,
            },
            // entry 2 -> node 9: merge points 4,5
            DendrogramEntry {
                left: 4,
                right: 5,
                distance: 2.0,
                size: 2,
            },
            // entry 3 -> node 10: merge nodes 7,8 (size 4)
            DendrogramEntry {
                left: 7,
                right: 8,
                distance: 4.0,
                size: 4,
            },
            // entry 4 -> node 11: merge node 9 + point 6 (size 3)
            DendrogramEntry {
                left: 9,
                right: 6,
                distance: 4.0,
                size: 3,
            },
            // entry 5 -> node 12: merge nodes 10,11 (size 7)
            DendrogramEntry {
                left: 10,
                right: 11,
                distance: 8.0,
                size: 7,
            },
        ];

        let condensed = condense_tree(&dendrogram, 3);

        // Should have exactly 2 cluster entries + 7 point fall-out entries = 9
        let cluster_entries: Vec<_> = condensed.iter().filter(|e| e.child_size > 1).collect();
        let point_entries: Vec<_> = condensed.iter().filter(|e| e.child_size == 1).collect();

        assert_eq!(
            cluster_entries.len(),
            2,
            "should have 2 child cluster entries"
        );
        assert_eq!(point_entries.len(), 7, "all 7 points should fall out");

        // The two cluster children should have sizes 4 and 3
        let mut child_sizes: Vec<usize> = cluster_entries.iter().map(|e| e.child_size).collect();
        child_sizes.sort();
        assert_eq!(child_sizes, vec![3, 4]);

        // Both cluster entries should have lambda = 1/8 = 0.125
        for entry in &cluster_entries {
            assert!(
                (entry.lambda - 0.125).abs() < 1e-10,
                "cluster lambda should be 1/8"
            );
        }

        // All point fall-out entries should have lambda = 1/4 = 0.25
        // (they fall out when the size-2 sub-clusters are too small)
        for entry in &point_entries {
            assert!(
                (entry.lambda - 0.25).abs() < 1e-10,
                "point lambda should be 1/4, got {}",
                entry.lambda
            );
        }

        // Every point 0..7 should appear exactly once
        let mut point_children: Vec<usize> = point_entries.iter().map(|e| e.child).collect();
        point_children.sort();
        assert_eq!(point_children, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    /// Test that condense_tree correctly handles a single-cluster case
    /// (no split large enough to produce two child clusters).
    #[test]
    fn test_condense_tree_single_cluster() {
        // 5 points, dendrogram merges them pairwise then together
        let dendrogram: Vec<DendrogramEntry<f64>> = vec![
            DendrogramEntry {
                left: 0,
                right: 1,
                distance: 1.0,
                size: 2,
            },
            DendrogramEntry {
                left: 2,
                right: 3,
                distance: 1.0,
                size: 2,
            },
            DendrogramEntry {
                left: 5,
                right: 4,
                distance: 2.0,
                size: 3,
            },
            DendrogramEntry {
                left: 6,
                right: 7,
                distance: 5.0,
                size: 5,
            },
        ];

        let condensed = condense_tree(&dendrogram, 3);

        // No split produces two children >= min_cluster_size (3), so
        // the root keeps absorbing. All points should fall out of the root.
        let cluster_entries: Vec<_> = condensed.iter().filter(|e| e.child_size > 1).collect();

        // The root (node 8) splits into node 7 (size 3) and node 6 (size 2).
        // Node 7 is >= 3 but node 6 is not, so node 6's points fall out and
        // node 7 inherits the root label. Then node 7 splits into node 5
        // (size 2, < 3) and point 4. Neither is >= 3, so their points fall out
        // of the root too.
        //
        // This means there are no true splits -- no cluster children.
        assert_eq!(cluster_entries.len(), 0, "no true split should occur");
    }

    // -- GLOSH score ordering -------------------------------------------------

    /// Points near the centre of a cluster should have lower outlier scores
    /// than points at the periphery.
    #[test]
    fn test_glosh_score_ordering() {
        let mut points = Vec::new();

        // Core of the cluster: dense 4x4 grid (16 points, spacing 0.1)
        for i in 0..16 {
            let x = (i % 4) as f64 * 0.1;
            let y = (i / 4) as f64 * 0.1;
            points.push(Point::new(x, y));
        }

        // Peripheral points: further from the centre
        points.push(Point::new(-0.5, 0.15)); // idx 16
        points.push(Point::new(0.8, 0.15)); // idx 17
        points.push(Point::new(0.15, -0.5)); // idx 18
        points.push(Point::new(0.15, 0.8)); // idx 19

        let result = points.hdbscan_with_min_samples(5, 3);

        // All should be in one cluster
        let label = result.labels[0];
        assert!(label.is_some(), "should find a cluster");

        // Average score of core points (first 16)
        let core_avg: f64 = result.outlier_scores[0..16].iter().sum::<f64>() / 16.0;

        // Average score of peripheral points (last 4)
        let peripheral_avg: f64 = result.outlier_scores[16..20].iter().sum::<f64>() / 4.0;

        assert!(
            peripheral_avg > core_avg,
            "peripheral avg ({peripheral_avg}) should exceed core avg ({core_avg})"
        );
    }

    // -- Large-n correctness --------------------------------------------------

    /// Test with ~500 deterministic points in 3 well-separated blobs.
    ///
    /// Uses a simple LCG with CLT-based Gaussian approximation for
    /// reproducibility. Gaussian-like density (high at centre, tapering
    /// outward) gives HDBSCAN a clear density gradient to work with,
    /// avoiding the over-splitting that occurs with uniform distributions.
    #[test]
    fn test_large_n_correctness() {
        // Simple LCG for deterministic pseudo-random numbers in [0, 1)
        let mut seed: u64 = 42;
        let mut next_f64 = || -> f64 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            (seed >> 33) as f64 / (1u64 << 31) as f64
        };

        // CLT Gaussian approximation: sum of 6 uniform [0,1) values
        // produces approximately N(3, sqrt(0.5)) -- shifted to N(0, ~0.7)
        let mut next_gaussian = || -> f64 {
            let sum = next_f64() + next_f64() + next_f64() + next_f64() + next_f64() + next_f64();
            sum - 3.0
        };

        let mut points = Vec::new();

        // Blob A: 200 points centred at (0, 0)
        for _ in 0..200 {
            points.push(Point::new(next_gaussian(), next_gaussian()));
        }

        // Blob B: 150 points centred at (30, 30)
        for _ in 0..150 {
            points.push(Point::new(30.0 + next_gaussian(), 30.0 + next_gaussian()));
        }

        // Blob C: 150 points centred at (60, 0)
        for _ in 0..150 {
            points.push(Point::new(60.0 + next_gaussian(), next_gaussian()));
        }

        let result = points.hdbscan_with_min_samples(15, 5);

        // Should find exactly 3 clusters
        let mut distinct: Vec<usize> = result.labels.iter().filter_map(|l| *l).collect();
        distinct.sort();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            3,
            "expected 3 clusters, found {}",
            distinct.len()
        );

        // Blob A (indices 0..200) should share a label
        let label_a = result.labels[0];
        assert!(label_a.is_some(), "blob A should be clustered");
        let a_consistent = (0..200).all(|i| result.labels[i] == label_a);
        assert!(a_consistent, "all blob A points should share a label");

        // Blob B (indices 200..350) should share a different label
        let label_b = result.labels[200];
        assert!(label_b.is_some(), "blob B should be clustered");
        let b_consistent = (200..350).all(|i| result.labels[i] == label_b);
        assert!(b_consistent, "all blob B points should share a label");

        // Blob C (indices 350..500) should share a third label
        let label_c = result.labels[350];
        assert!(label_c.is_some(), "blob C should be clustered");
        let c_consistent = (350..500).all(|i| result.labels[i] == label_c);
        assert!(c_consistent, "all blob C points should share a label");

        // All three labels should differ
        assert_ne!(label_a, label_b);
        assert_ne!(label_a, label_c);
        assert_ne!(label_b, label_c);
    }

    // -- f32 test -------------------------------------------------------------

    /// Verify that the algorithm works with f32 coordinates.
    #[test]
    fn test_f32_two_clusters() {
        let points: Vec<Point<f32>> = vec![
            Point::new(0.0f32, 0.0),
            Point::new(0.1, 0.0),
            Point::new(0.0, 0.1),
            Point::new(0.1, 0.1),
            Point::new(0.05, 0.05),
            Point::new(10.0, 10.0),
            Point::new(10.1, 10.0),
            Point::new(10.0, 10.1),
            Point::new(10.1, 10.1),
            Point::new(10.05, 10.05),
        ];
        let result = points.hdbscan(3);

        let label_a = result.labels[0];
        let label_b = result.labels[5];
        assert!(label_a.is_some(), "cluster A should be found");
        assert!(label_b.is_some(), "cluster B should be found");
        assert_ne!(label_a, label_b);

        for i in 0..5 {
            assert_eq!(result.labels[i], label_a);
        }
        for i in 5..10 {
            assert_eq!(result.labels[i], label_b);
        }
    }
}
