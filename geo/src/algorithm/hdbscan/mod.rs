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

use crate::GeoFloat;
use crate::Point;
use crate::algorithm::ball_tree::BallTree;

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

#[cfg(test)]
mod tests;
