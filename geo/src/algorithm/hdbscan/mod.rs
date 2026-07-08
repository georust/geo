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

// Temporary allowance while the algorithm is built up over several
// commits; removed in the commit that adds the public API.
#![allow(dead_code)]

use crate::Distance;
use crate::Euclidean;
use crate::GeoFloat;
use crate::Point;
use crate::algorithm::ball_tree::BallTree;
use crate::algorithm::ball_tree::NodeKind;
use std::collections::HashMap;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Step 1: Core distances via ball tree k-NN
// ---------------------------------------------------------------------------

/// Compute the core distance and k-NN indices for each point.
///
/// The *core distance* of a point is the distance to its `min_samples`-th
/// nearest neighbour, **excluding the point itself**. This matches the
/// reference [`hdbscan`](https://github.com/scikit-learn-contrib/hdbscan)
/// library: `min_samples` counts other points, not the query point. We query
/// `min_samples + 1` neighbours so that, after the self-match at distance 0,
/// exactly `min_samples` other neighbours remain; the farthest of them
/// (index `min_samples` in the distance-sorted result) is the core distance.
///
/// Runs a single pass (parallel with the `multithreading` feature; sequential
/// otherwise): for each point `p` it records `core_distances[p]` and the
/// neighbour indices. The k-NN index table is retained on `CoreData` for use
/// in per-round seeding via `seed_best_edges`.
///
/// `hdbscan_impl` validates `min_samples <= n - 1`, so the query always
/// returns exactly `min_samples + 1` neighbours and the index is in range.
pub(crate) fn compute_core_data<T: GeoFloat + Send + Sync>(
    tree: &BallTree<T>,
    points: &[Point<T>],
    min_samples: usize,
) -> CoreData<T> {
    let n = points.len();
    let k = min_samples + 1;
    let mut core_distances = vec![T::zero(); n];
    // Flat row-major k-NN table: row `p` is `knn_indices[p * k..(p + 1) * k]`.
    // One allocation instead of `n` per-point Vecs, and contiguous rows for
    // the per-round seeding scan.
    let mut knn_indices: Vec<usize> = vec![0; n * k];

    // The per-point body is identical across the multithreading and
    // sequential builds, so it lives in one closure both call. Only the spawn
    // scaffolding is feature-gated.
    let compute = |p: &Point<T>, dist: &mut T, row: &mut [usize]| {
        let knn = tree.nearest_neighbours(p, k);
        debug_assert_eq!(knn.len(), k, "min_samples must be <= n - 1");
        *dist = knn[min_samples].distance;
        for (slot, nn) in row.iter_mut().zip(knn.iter()) {
            *slot = nn.index;
        }
    };

    #[cfg(feature = "multithreading")]
    {
        let num_threads = std::thread::available_parallelism()
            .map(|t| t.get())
            .unwrap_or(1);
        let chunk_size = n.div_ceil(num_threads).max(1);

        std::thread::scope(|s| {
            for ((dist_chunk, knn_chunk), points_chunk) in core_distances
                .chunks_mut(chunk_size)
                .zip(knn_indices.chunks_mut(chunk_size * k))
                .zip(points.chunks(chunk_size))
            {
                let compute = &compute;
                s.spawn(move || {
                    for ((d, row), p) in dist_chunk
                        .iter_mut()
                        .zip(knn_chunk.chunks_mut(k))
                        .zip(points_chunk)
                    {
                        compute(p, d, row);
                    }
                });
            }
        });
    }

    #[cfg(not(feature = "multithreading"))]
    {
        for ((d, row), p) in core_distances
            .iter_mut()
            .zip(knn_indices.chunks_mut(k))
            .zip(points.iter())
        {
            compute(p, d, row);
        }
    }

    CoreData {
        core_distances,
        knn_indices,
        knn_k: k,
    }
}

// ---------------------------------------------------------------------------
// Step 2: Boruvka's MST on the mutual reachability graph
// ---------------------------------------------------------------------------

/// Data returned by `compute_core_data`: one entry per input point.
pub(crate) struct CoreData<T> {
    pub(crate) core_distances: Vec<T>,
    /// Flat row-major k-NN indices: point `p`'s neighbours occupy
    /// `knn_indices[p * knn_k..(p + 1) * knn_k]` (self at distance 0 plus the
    /// `min_samples` nearest other points). Retained for per-round seeding;
    /// freed when `CoreData` drops at the end of the HDBSCAN run.
    pub(crate) knn_indices: Vec<usize>,
    /// Row length of `knn_indices` (`min_samples + 1`).
    pub(crate) knn_k: usize,
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

    // Parallel tree-position arrays: position `i` corresponds to
    // `tree.node(0).points()[i]` and its original index is
    // `tree.node(0).indices()[i]`. Keeping core distances and component labels
    // in tree-position order turns the inner loops (process_leaves and
    // update_component_of_node) into sequential cache-friendly walks instead
    // of gathers through `core_distances[indices[j]]` and
    // `point_components[indices[j]]`. Built before the initial
    // update_component_of_node call, which now consumes the pos array too.
    let tree_order: Vec<usize> = tree.node(0).indices().to_vec();
    debug_assert_eq!(tree_order.len(), n);
    let core_distances_pos: Vec<T> = tree_order
        .iter()
        .map(|&i| core_data.core_distances[i])
        .collect();
    let mut point_components_pos: Vec<usize> =
        tree_order.iter().map(|&i| point_components[i]).collect();

    update_component_of_node(tree, &point_components_pos, &mut component_of_node);

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
            core_data.knn_k,
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
        let root_dist = state.min_node_dist(0, 0);
        state.traverse(0, 0, root_dist);

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
        update_component_of_node(tree, &point_components_pos, &mut component_of_node);
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
    ///
    /// `node_dist` is the minimum Euclidean distance between the two balls,
    /// already computed by the caller (which needs it to order recursion).
    /// Passing it in avoids recomputing the same sqrt-bearing bound at the
    /// top of every call.
    fn traverse(&mut self, query_idx: usize, ref_idx: usize, node_dist: T) {
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
                let ((first, df), (second, ds)) = if dl <= dr {
                    ((rl, dl), (rr, dr))
                } else {
                    ((rr, dr), (rl, dl))
                };
                self.traverse(query_idx, first, df);
                self.traverse(query_idx, second, ds);
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
                let ((first, df), (second, ds)) = if dl <= dr {
                    ((ql, dl), (qr, dr))
                } else {
                    ((qr, dr), (ql, dl))
                };
                self.traverse(first, ref_idx, df);
                self.traverse(second, ref_idx, ds);
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
                for &(q, r, d) in &pairs {
                    self.traverse(q, r, d);
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

                // `Euclidean.distance` computes `hypot`, whereas the ball
                // tree's k-NN distances and node bounds use
                // `sqrt(dx * dx + dy * dy)`. The two can differ by ~1 ULP, but
                // the pruning bound has geometric slack (centre distance minus
                // radii), so the mismatch cannot cause an incorrect prune.
                let d = Euclidean.distance(q_coord, r_points[j].0);
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
/// `knn_indices` is the flat row-major table (row `p` is
/// `knn_indices[p * k..(p + 1) * k]`); `point_components` is indexed by
/// original point index (NOT tree position). `core_distances` is likewise
/// original-index order.
fn seed_best_edges<T: GeoFloat>(
    knn_indices: &[usize],
    k: usize,
    core_distances: &[T],
    point_components: &[usize],
    best_edges: &mut [Option<BestEdge<T>>],
) {
    debug_assert_eq!(knn_indices.len(), core_distances.len() * k);
    debug_assert_eq!(point_components.len(), core_distances.len());
    debug_assert_eq!(best_edges.len(), core_distances.len());

    for (p, neighbours) in knn_indices.chunks(k).enumerate() {
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
///
/// `point_components_pos` is the component label per point in tree-position
/// order (parallel to `BallTreeNode::points()`), so a leaf's points are read
/// sequentially over `position_range()` rather than gathered through the
/// `indices()` permutation.
fn update_component_of_node<T: GeoFloat>(
    tree: &BallTree<T>,
    point_components_pos: &[usize],
    component_of_node: &mut [Option<usize>],
) {
    for node_idx in (0..tree.node_count()).rev() {
        let node = tree.node(node_idx);
        match node.kind() {
            NodeKind::Leaf => {
                let mut comp = None;
                let mut all_same = true;
                for pos in node.position_range() {
                    let c = point_components_pos[pos];
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
/// synthetic label – see [`TreeUnionFind`]) can share the same
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

    /// Union by size. Returns `Some(big_root)` (the surviving root) when a
    /// merge happened, or `None` when `x` and `y` were already in the same
    /// component. The caller can use the surviving root to update any
    /// side-channel bookkeeping keyed on roots.
    fn union(&mut self, x: usize, y: usize) -> Option<usize> {
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
        Some(big)
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
        if let Some(big) = self.inner.union(x, y) {
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
// Step 3: Label – build dendrogram from sorted MST edges
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
///
/// `n_points` is the true number of input points, passed in rather than
/// derived from `dendrogram.len()`: a complete dendrogram has exactly
/// `n_points - 1` entries, but deriving the count would silently mis-classify
/// point vs virtual ids if the dendrogram were ever short. (`hdbscan_impl`
/// rejects incomplete spanning trees before reaching here.)
fn condense_tree<T: GeoFloat>(
    dendrogram: &[DendrogramEntry<T>],
    min_cluster_size: usize,
    n_points: usize,
) -> Vec<CondensedEntry<T>> {
    // An empty dendrogram (fewer than two points) has no virtual root to
    // expand; `Virtual(n_points - 2)` would also underflow. Nothing condenses.
    if dendrogram.is_empty() {
        return Vec::new();
    }
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

    // A merge distance (mutual-reachability distance) of zero – produced when
    // enough coincident points drive a core distance to zero – would give
    // lambda = 1/0 = infinity and poison the stability and GLOSH stages
    // downstream (they would see non-finite or NaN values). Cap such merges at
    // the largest finite lambda in the tree (the reciprocal of the smallest
    // merge distance whose reciprocal is still finite – a subnormal distance
    // overflows to infinity and is excluded) so every lambda stays finite.
    // Points that fall out at this capped lambda sit at the cluster core and
    // score zero in GLOSH. If no merge yields a finite positive lambda (fully
    // coincident input) any positive fallback works: every merge is then at
    // the same capped lambda, so all stabilities are zero.
    let lambda_cap = {
        let max_finite = dendrogram
            .iter()
            .filter(|e| e.distance > T::zero())
            .map(|e| T::one() / e.distance)
            .filter(|l| l.is_finite())
            .fold(T::zero(), T::max);
        if max_finite > T::zero() {
            max_finite
        } else {
            T::one()
        }
    };

    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        let virt_idx = match node {
            // Leaves (original points) do not expand.
            DendrogramNode::Point(_) => continue,
            DendrogramNode::Virtual(i) => i,
        };

        let entry = &dendrogram[virt_idx];
        // A non-finite reciprocal (distance zero, or subnormal enough to
        // overflow) is replaced by the cap; a NaN distance also lands here.
        let lambda = {
            let l = T::one() / entry.distance;
            if l.is_finite() { l } else { lambda_cap }
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
            (true, false) | (false, true) => {
                // Exactly one child is big enough: it inherits the parent
                // label and keeps expanding; the small child's points fall out.
                let (big, small) = if left_big {
                    (left, right)
                } else {
                    (right, left)
                };
                relabel.insert(big, parent_label);
                stack.push(big);
                emit_fallout(small, parent_label, lambda, dendrogram, &mut result);
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
/// Following the reference HDBSCAN, a cluster's stability is
///
/// ```text
/// sum over every condensed-tree row with this cluster as parent of
///     (lambda_row - lambda_birth) * child_size
/// ```
///
/// where `lambda_birth` is the lambda at which the cluster was created (the
/// lambda of the entry where this cluster appears as a child; zero for the
/// root, which never appears as a child). Both point fall-out rows
/// (`child_size == 1`) and cluster-split rows (`child_size > 1`) contribute:
/// the split rows credit the parent for the mass still present when it
/// splits. Omitting the split term biases excess-of-mass selection towards
/// children and over-fragments the output.
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
        // Ensure every cluster appears as a key: parents always, and
        // cluster-children so that leaf clusters seed a zero entry.
        stability.entry(entry.parent).or_insert(T::zero());
        if entry.child_size > 1 {
            stability.entry(entry.child).or_insert(T::zero());
        }

        let birth = lambda_birth
            .get(&entry.parent)
            .copied()
            .unwrap_or(T::zero());
        let child_size = T::from(entry.child_size).expect("child_size fits in T");
        let contribution = (entry.lambda - birth) * child_size;
        let s = stability.entry(entry.parent).or_insert(T::zero());
        *s = *s + contribution;
    }

    stability
}

/// Map each cluster-child to its parent cluster in the condensed tree.
///
/// Both `find_clusters` and `extract_labels` need this reverse mapping; it is
/// built once in `hdbscan_impl` and shared by reference.
fn build_parent_of<T>(condensed: &[CondensedEntry<T>]) -> HashMap<usize, usize> {
    let mut parent_of: HashMap<usize, usize> = HashMap::new();
    for entry in condensed {
        if entry.child_size > 1 {
            parent_of.insert(entry.child, entry.parent);
        }
    }
    parent_of
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
    parent_of: &HashMap<usize, usize>,
) -> (HashMap<usize, usize>, HashSet<usize>) {
    // Identify all cluster nodes (those that appear as parents, or as
    // children with size > 1), tracking which appear as a cluster-child.
    let mut all_clusters: HashSet<usize> = HashSet::new();
    let mut cluster_children: HashSet<usize> = HashSet::new();
    for entry in condensed {
        all_clusters.insert(entry.parent);
        if entry.child_size > 1 {
            all_clusters.insert(entry.child);
            cluster_children.insert(entry.child);
        }
    }

    // The root(s): parent clusters that never appear as a cluster-child.
    // Reference HDBSCAN (with the default `allow_single_cluster = false`)
    // excludes the root from selection, so points that reach it without
    // passing through a selected cluster are labelled noise. A complete MST
    // yields exactly one root; a disconnected forest – which finding 4
    // rejects – would yield one root per component, and excluding all of
    // them is the correct generalisation.
    let roots: HashSet<usize> = all_clusters
        .iter()
        .filter(|c| !cluster_children.contains(c))
        .copied()
        .collect();

    // Build parent->children mapping (clusters only).
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    for entry in condensed {
        if entry.child_size > 1 {
            children.entry(entry.parent).or_default().push(entry.child);
        }
    }

    // Selectable (non-root) leaves: clusters with no cluster-children.
    let leaves: Vec<usize> = all_clusters
        .iter()
        .filter(|c| !roots.contains(c) && !children.contains_key(c))
        .copied()
        .collect();

    // Bottom-up pass: propagate total stability in topological order (each
    // cluster after all its children, tracked by unprocessed-child in-degree)
    // and record which clusters "win" – are at least as stable as their
    // combined children. Winners are *not* deselected here; a separate
    // top-down pass resolves the final selection in one O(c) sweep, avoiding
    // the O(c^2) cost of re-walking each winning ancestor's whole subtree.
    let mut total_stability: HashMap<usize, T> = stability.clone();
    let mut won: HashSet<usize> = HashSet::new();

    // In-degree: number of cluster children not yet processed.
    let mut in_degree: HashMap<usize, usize> = HashMap::new();
    for (cluster, child_list) in &children {
        in_degree.insert(*cluster, child_list.len());
    }

    // Seed the queue with leaves (clusters with no children / in-degree 0).
    let mut queue: Vec<usize> = leaves;

    while let Some(cluster) = queue.pop() {
        // The root is excluded from selection and, having no parent,
        // propagates nothing.
        if roots.contains(&cluster) {
            continue;
        }
        match children.get(&cluster) {
            Some(child_list) => {
                let children_stability: T = child_list
                    .iter()
                    .map(|c| *total_stability.get(c).unwrap_or(&T::zero()))
                    .fold(T::zero(), |acc, x| acc + x);
                let self_stability = *stability.get(&cluster).unwrap_or(&T::zero());
                if self_stability >= children_stability {
                    won.insert(cluster);
                    total_stability.insert(cluster, self_stability);
                } else {
                    total_stability.insert(cluster, children_stability);
                }
            }
            // A leaf cluster has no children to be more stable than it, so it
            // always wins.
            None => {
                won.insert(cluster);
            }
        }

        // Decrement parent's in-degree; enqueue when all children are done.
        if let Some(&parent) = parent_of.get(&cluster) {
            let deg = in_degree.get_mut(&parent).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push(parent);
            }
        }
    }

    // Top-down pass: on each root-to-leaf path the topmost winner is selected
    // and everything below it is deselected. Descend from the roots' children;
    // the first winner on a path is added and its subtree is not explored.
    let mut is_cluster: HashSet<usize> = HashSet::new();
    let mut stack: Vec<usize> = roots
        .iter()
        .filter_map(|r| children.get(r))
        .flatten()
        .copied()
        .collect();
    while let Some(cluster) = stack.pop() {
        if won.contains(&cluster) {
            is_cluster.insert(cluster);
        } else if let Some(child_list) = children.get(&cluster) {
            stack.extend(child_list.iter().copied());
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

// ---------------------------------------------------------------------------
// Step 6: Extract flat cluster labels
// ---------------------------------------------------------------------------

/// Assign each point to its nearest selected ancestor cluster.
fn extract_labels<T: GeoFloat>(
    condensed: &[CondensedEntry<T>],
    cluster_labels: &HashMap<usize, usize>,
    is_cluster: &HashSet<usize>,
    parent_of: &HashMap<usize, usize>,
    n: usize,
) -> Vec<Option<usize>> {
    // For each point, find the deepest selected cluster it belongs to.
    // A point belongs to a cluster if it appears (directly or transitively)
    // as a child_size==1 entry under that cluster.

    // Build a map: for each cluster, find its nearest selected ancestor
    let mut cluster_to_selected: HashMap<usize, Option<usize>> = HashMap::new();

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
            let label = find_selected_ancestor(entry.parent, parent_of, &mut cluster_to_selected);
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

/// Walk up the condensed-tree parent chain from `start` until a cached entry
/// or a root is reached, then back-fill the cache for every cluster visited.
/// Iterative to avoid unbounded recursion on deep hierarchies, matching the
/// stack-based pattern in [`emit_fallout`].
///
/// `extract_labels` pre-seeds the cache with every selected cluster's label,
/// so a selected ancestor is always found via the cache; a chain that reaches
/// a root (never selected, never cached) yields `None` – the point is noise.
fn find_selected_ancestor(
    start: usize,
    parent_of: &HashMap<usize, usize>,
    cluster_to_selected: &mut HashMap<usize, Option<usize>>,
) -> Option<usize> {
    let mut chain = Vec::new();
    let mut current = start;
    let result = loop {
        if let Some(&cached) = cluster_to_selected.get(&current) {
            break cached;
        }
        chain.push(current);
        match parent_of.get(&current) {
            Some(&parent) => current = parent,
            // Root: not selected, so points routed here are noise.
            None => break None,
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

/// Compute GLOSH outlier scores for every point.
///
/// Following the reference HDBSCAN, the score is computed purely from the
/// condensed tree and is independent of the flat cluster selection, so noise
/// points are scored too (rather than hard-coded to 1.0). For a point that
/// falls out of condensed-tree cluster `C` at lambda `lambda_p`,
///
/// ```text
/// score = (lambda_max(C) - lambda_p) / lambda_max(C)
/// ```
///
/// where `lambda_max(C)` is the deepest lambda anywhere in the subtree rooted
/// at `C` – the highest density any descendant reaches. A point sitting at
/// the densest core scores 0; a point that drops out while the rest of its
/// neighbourhood persists to much higher density scores near 1.
fn glosh<T: GeoFloat>(condensed: &[CondensedEntry<T>], n: usize) -> Vec<T> {
    // Pass 1: each cluster's direct "death" lambda – the maximum lambda over
    // the condensed-tree rows that have it as parent.
    let mut deaths: HashMap<usize, T> = HashMap::new();
    for entry in condensed {
        let d = deaths.entry(entry.parent).or_insert(T::zero());
        *d = (*d).max(entry.lambda);
    }

    // Pass 2: propagate deaths up so `deaths[c]` becomes the maximum lambda
    // across the whole subtree rooted at `c`. `condense_tree` emits rows in
    // DFS pre-order (a cluster's rows precede all of its descendants' rows),
    // so iterating in reverse visits descendants before ancestors and one
    // pass suffices.
    for entry in condensed.iter().rev() {
        if entry.child_size > 1 {
            let child_death = deaths.get(&entry.child).copied().unwrap_or(T::zero());
            let d = deaths.entry(entry.parent).or_insert(T::zero());
            *d = (*d).max(child_death);
        }
    }

    // Score each point from its direct parent cluster's subtree-max lambda.
    // `child_size == 1` uniquely identifies point-fallout entries (see the
    // note in `extract_labels`); every point appears in exactly one.
    let mut scores = vec![T::zero(); n];
    for entry in condensed {
        if entry.child_size == 1 {
            let lambda_max = deaths.get(&entry.parent).copied().unwrap_or(T::zero());
            // `lambda_max` is finite (condense_tree caps zero-distance merges);
            // a zero denominator only arises for fully degenerate input, where
            // the score is defined as 0.
            scores[entry.child] = if lambda_max > T::zero() {
                (lambda_max - entry.lambda) / lambda_max
            } else {
                T::zero()
            };
        }
    }
    scores
}

#[cfg(test)]
mod tests;
