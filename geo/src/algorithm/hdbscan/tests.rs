use super::*;
use crate::wkt;
use approx::assert_relative_eq;

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

// -- Internal pipeline unit tests -----------------------------------------

/// Test boruvka_mst on the 7-point dataset from petal-clustering.
///
/// Points and core distances are chosen so the MST is unambiguous.
/// We verify the total weight rather than edge ordering (which varies
/// between Boruvka implementations).
#[test]
fn test_boruvka_mst_total_weight() {
    // Points 0..7, indexed to the core_distances below.
    let points: Vec<Point<f64>> = wkt!(MULTIPOINT(
        0.0 0.0, 7.0 0.0, 15.0 0.0, 0.0 -5.0, 15.0 -5.0, 7.0 -7.0, 15.0 -14.0
    ))
    .0;
    let core_distances = vec![5.0, 7.0, 5.0, 5.0, 5.0, 6.0, 9.0];
    let tree = BallTree::new(points.iter().copied());
    let core_data = CoreData {
        core_distances,
        // A self-only k-NN row per point (knn_k = 1) means seed_best_edges
        // proposes no edges, so the MST is built entirely by the dual-tree
        // traversal from the explicit core distances above.
        knn_indices: (0..points.len()).collect(),
        knn_k: 1,
    };

    let mst = boruvka_mst(&tree, &points, &core_data);

    // MST should have n-1 = 6 edges
    assert_eq!(mst.len(), 6, "MST should have n-1 edges");

    // Expected MRD edges (canonicalised as (min, max, weight) and sorted):
    //   (0,3): max(5,5,5) = 5
    //   (2,4): max(5,5,5) = 5
    //   (0,1): max(5,7,7) = 7
    //   (1,5): max(7,6,7) = 7
    //   (1,2): max(7,5,8) = 8
    //   (4,6): max(5,9,9) = 9
    let mut actual: Vec<(usize, usize, f64)> = mst
        .iter()
        .map(|e| (e.u.min(e.v), e.u.max(e.v), e.weight))
        .collect();
    actual.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.total_cmp(&b.2))
    });
    let expected: Vec<(usize, usize, f64)> = vec![
        (0, 1, 7.0),
        (0, 3, 5.0),
        (1, 2, 8.0),
        (1, 5, 7.0),
        (2, 4, 5.0),
        (4, 6, 9.0),
    ];
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert_eq!((a.0, a.1), (e.0, e.1), "edge mismatch");
        assert_relative_eq!(a.2, e.2, epsilon = 1e-10);
    }

    let total_weight: f64 = mst.iter().map(|e| e.weight).sum();
    assert_relative_eq!(total_weight, 41.0, epsilon = 1e-10);
}

/// Test that the MST connects all points (every point reachable).
#[test]
fn test_boruvka_mst_connectivity() {
    let points: Vec<Point<f64>> = wkt!(MULTIPOINT(
        0.0 0.0, 7.0 0.0, 15.0 0.0, 0.0 -5.0, 15.0 -5.0, 7.0 -7.0, 15.0 -14.0
    ))
    .0;
    let core_distances = vec![5.0, 7.0, 5.0, 5.0, 5.0, 6.0, 9.0];
    let tree = BallTree::new(points.iter().copied());
    let core_data = CoreData {
        core_distances,
        // A self-only k-NN row per point (knn_k = 1) means seed_best_edges
        // proposes no edges, so the MST is built entirely by the dual-tree
        // traversal from the explicit core distances above.
        knn_indices: (0..points.len()).collect(),
        knn_k: 1,
    };

    let mst = boruvka_mst(&tree, &points, &core_data);

    // Check connectivity via union-find
    let mut uf = UnionForest::new(points.len());
    for edge in &mst {
        uf.union(edge.u, edge.v);
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
    assert_relative_eq!(distances[0], 1.0, epsilon = 1e-12);
    assert_relative_eq!(distances[1], 1.0, epsilon = 1e-12);

    // Final merge creates the root with all 4 points
    assert_eq!(sizes[2], 4);
    assert_relative_eq!(distances[2], 10.0, epsilon = 1e-12);
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
    use DendrogramNode::{Point as P, Virtual as V};
    let dendrogram: Vec<DendrogramEntry<f64>> = vec![
        // entry 0 -> node 7: merge points 0,1
        DendrogramEntry {
            left: P(0),
            right: P(1),
            distance: 2.0,
            size: 2,
        },
        // entry 1 -> node 8: merge points 2,3
        DendrogramEntry {
            left: P(2),
            right: P(3),
            distance: 2.0,
            size: 2,
        },
        // entry 2 -> node 9: merge points 4,5
        DendrogramEntry {
            left: P(4),
            right: P(5),
            distance: 2.0,
            size: 2,
        },
        // entry 3 -> node 10: merge virtual entries 0,1 (size 4)
        DendrogramEntry {
            left: V(0),
            right: V(1),
            distance: 4.0,
            size: 4,
        },
        // entry 4 -> node 11: merge virtual entry 2 + point 6 (size 3)
        DendrogramEntry {
            left: V(2),
            right: P(6),
            distance: 4.0,
            size: 3,
        },
        // entry 5 -> node 12: merge virtual entries 3,4 (size 7)
        DendrogramEntry {
            left: V(3),
            right: V(4),
            distance: 8.0,
            size: 7,
        },
    ];

    let condensed = condense_tree(&dendrogram, 3, 7);

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
        assert_relative_eq!(entry.lambda, 0.125, epsilon = 1e-10);
    }

    // All point fall-out entries should have lambda = 1/4 = 0.25
    // (they fall out when the size-2 sub-clusters are too small)
    for entry in &point_entries {
        assert_relative_eq!(entry.lambda, 0.25, epsilon = 1e-10);
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
    use DendrogramNode::{Point as P, Virtual as V};
    // 5 points, dendrogram merges them pairwise then together
    let dendrogram: Vec<DendrogramEntry<f64>> = vec![
        DendrogramEntry {
            left: P(0),
            right: P(1),
            distance: 1.0,
            size: 2,
        },
        DendrogramEntry {
            left: P(2),
            right: P(3),
            distance: 1.0,
            size: 2,
        },
        // virtual entry 2: merge V(0) + point 4 (size 3)
        DendrogramEntry {
            left: V(0),
            right: P(4),
            distance: 2.0,
            size: 3,
        },
        // root: merge V(1) + V(2) (size 5)
        DendrogramEntry {
            left: V(1),
            right: V(2),
            distance: 5.0,
            size: 5,
        },
    ];

    let condensed = condense_tree(&dendrogram, 3, 5);

    // No split produces two children >= min_cluster_size (3), so
    // the root keeps absorbing. All points should fall out of the root.
    let cluster_entries: Vec<_> = condensed.iter().filter(|e| e.child_size > 1).collect();

    // The root (node 8) splits into node 7 (size 3) and node 6 (size 2).
    // Node 7 is >= 3 but node 6 is not, so node 6's points fall out and
    // node 7 inherits the root label. Then node 7 splits into node 5
    // (size 2, < 3) and point 4. Neither is >= 3, so their points fall out
    // of the root too.
    //
    // This means there are no true splits – no cluster children.
    assert_eq!(cluster_entries.len(), 0, "no true split should occur");
}

#[test]
fn compute_core_data_retains_knn_indices() {
    use crate::Point;
    use crate::algorithm::ball_tree::BallTree;

    let pts: Vec<Point<f64>> = (0..6).map(|i| Point::new(i as f64, 0.0)).collect();
    let tree = BallTree::new(pts.iter().copied());
    let core_data = super::compute_core_data(&tree, &pts, 3);

    // min_samples + 1 = 4 neighbours are stored per point (self plus the
    // 3 nearest other points), in a flat row-major table.
    assert_eq!(core_data.knn_k, 4);
    assert_eq!(core_data.knn_indices.len(), 6 * 4);
    for (p, row) in core_data.knn_indices.chunks(core_data.knn_k).enumerate() {
        assert_eq!(row.len(), 4);
        assert!(row.contains(&p), "knn row {p} should contain self");
    }
}

#[test]
fn seed_best_edges_seeds_tightest_cross_component_mrd() {
    use crate::point;
    let pts: Vec<Point<f64>> = vec![
        point!(x: 0.0, y: 0.0),
        point!(x: 1.0, y: 0.0),
        point!(x: 10.0, y: 0.0),
        point!(x: 11.0, y: 0.0),
    ];
    let tree = BallTree::new(pts.iter().copied());

    // min_samples = 1 stores 2 neighbours per point (self + nearest other).
    // For points 0,1 the nearest other is each other (same component), so
    // component A gets no cross-component seed; likewise B.
    let core_data = super::compute_core_data(&tree, &pts, 1);
    let point_components: Vec<usize> = vec![0, 0, 2, 2];
    let mut best_edges: Vec<Option<super::BestEdge<f64>>> = vec![None; 4];
    super::seed_best_edges(
        &core_data.knn_indices,
        core_data.knn_k,
        &core_data.core_distances,
        &point_components,
        &mut best_edges,
    );
    assert!(
        best_edges[0].is_none(),
        "comp 0 has no cross-component neighbour at min_samples=1"
    );
    assert!(
        best_edges[2].is_none(),
        "comp 2 has no cross-component neighbour at min_samples=1"
    );

    // min_samples = 2 stores 3 neighbours per point, so point 1's kNN
    // reaches across to point 2.
    let core_data_k2 = super::compute_core_data(&tree, &pts, 2);
    let mut best_edges_k2: Vec<Option<super::BestEdge<f64>>> = vec![None; 4];
    super::seed_best_edges(
        &core_data_k2.knn_indices,
        core_data_k2.knn_k,
        &core_data_k2.core_distances,
        &point_components,
        &mut best_edges_k2,
    );
    let e0 = best_edges_k2[0].expect("comp 0 should have a seed via point 1 -> point 2");
    let expected = core_data_k2.core_distances[1].max(core_data_k2.core_distances[2]);
    assert_eq!(e0.mrd, expected, "seed MRD should be max(core(p), core(q))");
    assert_eq!(e0.from, 1);
    assert_eq!(e0.to, 2);
}

#[test]
fn seed_best_edges_scans_all_k_not_first_hit() {
    // Construct kNN lists manually to force a case where the FIRST
    // cross-component neighbour has LARGER core(q) than a LATER one.
    // If the helper breaks on first hit, it will pick the wrong neighbour.
    //
    // In round 2+, component IDs are still original point indices (the
    // UnionForest root). Here we use a merged component: points 1..=4
    // share component 1 (point 1 is their root). Point 0 is in comp 0.
    // Flat row-major k-NN table (knn_k = 3): one row of 3 neighbours per point.
    #[rustfmt::skip]
    let knn_indices: Vec<usize> = vec![
        0, 1, 2,
        1, 0, 2,
        2, 1, 0,
        3, 4, 0,
        4, 3, 0,
    ];
    let core_distances: Vec<f64> = vec![5.0, 10.0, 3.0, 5.0, 5.0];
    // Point 0 alone in comp 0; points 1..=4 in comp 1 (root = point 1).
    let point_components: Vec<usize> = vec![0, 1, 1, 1, 1];
    let mut best_edges: Vec<Option<super::BestEdge<f64>>> = vec![None; 5];
    super::seed_best_edges(
        &knn_indices,
        3,
        &core_distances,
        &point_components,
        &mut best_edges,
    );

    let e = best_edges[0].expect("comp 0 should have a seed");
    // First cross-component hit is point 1 with MRD = max(5, 10) = 10.
    // Later neighbour point 2 has MRD = max(5, 3) = 5 which is tighter.
    assert_eq!(
        e.to, 2,
        "should pick tightest-MRD neighbour, not first cross-component hit"
    );
    assert_relative_eq!(e.mrd, 5.0, epsilon = 1e-10);
}

#[test]
fn seed_best_edges_skips_self() {
    // Point 0's kNN starts with self (distance 0). If self-skip is
    // missing, the helper would write a (0 -> 0) edge, or pick self's
    // own core as a candidate MRD. Both points are present so all slices
    // are length 2 as required by the function contract.
    // Flat row-major table (knn_k = 2): row 0 = [0, 1], row 1 = [1, 0].
    let knn_indices: Vec<usize> = vec![0, 1, 1, 0];
    let core_distances: Vec<f64> = vec![1.0, 2.0];
    let point_components: Vec<usize> = vec![0, 1];
    let mut best_edges: Vec<Option<super::BestEdge<f64>>> = vec![None; 2];
    super::seed_best_edges(
        &knn_indices,
        2,
        &core_distances,
        &point_components,
        &mut best_edges,
    );
    let e = best_edges[0].expect("should find point 1 as cross-component");
    assert_eq!(e.to, 1);
    assert_ne!(e.from, e.to);
}

#[test]
fn seed_bounds_propagates_bottom_up_from_leaf_best_edges() {
    use crate::Point;
    use crate::algorithm::ball_tree::BallTree;

    // Six points. The ball tree's node structure is opaque; we rely only
    // on the invariant that bounds[internal] = max(bounds[left],
    // bounds[right]) and bounds[leaf] = max over points in the leaf of
    // best_edges[comp(p)].mrd.
    let points: Vec<Point<f64>> = wkt!(MULTIPOINT(
        0.0 0.0, 1.0 0.0, 2.0 0.0, 3.0 0.0, 4.0 0.0, 5.0 0.0
    ))
    .0;
    let tree = BallTree::new(points.iter().copied());
    let num_nodes = tree.node_count();

    // Seed every point's best edge to mrd = 7.0 except point 3 which is
    // unseeded (None -> contributes infinity). Round-1 components: comp(p)=p.
    let mut best_edges: Vec<Option<super::BestEdge<f64>>> = (0..6)
        .map(|p| {
            if p == 3 {
                None
            } else {
                Some(super::BestEdge {
                    from: p,
                    to: (p + 1) % 6,
                    mrd: 7.0,
                })
            }
        })
        .collect();
    let point_components_pos: Vec<usize> = tree.node(0).indices().to_vec();
    // In round 1, comp(p) = p, so point_components_pos[i] = tree.node(0).indices()[i].

    let mut bounds = vec![f64::NEG_INFINITY; num_nodes];
    super::seed_bounds(&tree, &best_edges, &point_components_pos, &mut bounds);

    // The root's bound must be the max over all points in the tree of
    // their best_edges[comp(p)].mrd. One point (3) is unseeded, which
    // contributes infinity, so bounds[0] must be infinity.
    assert_eq!(bounds[0], f64::INFINITY);

    // Re-seed p3 and rerun: now every point contributes 7.0 and every
    // bound must equal 7.0 exactly (no node is empty; every leaf has at
    // least one point and every internal's children are non-empty).
    best_edges[3] = Some(super::BestEdge {
        from: 3,
        to: 4,
        mrd: 7.0,
    });
    bounds.fill(f64::NEG_INFINITY);
    super::seed_bounds(&tree, &best_edges, &point_components_pos, &mut bounds);

    for (idx, b) in bounds.iter().enumerate() {
        assert_relative_eq!(*b, 7.0, epsilon = 1e-12);
        // Sanity: no node left at neg_infinity, i.e. every node was visited.
        assert_ne!(*b, f64::NEG_INFINITY, "node {idx} was not visited");
    }
}
