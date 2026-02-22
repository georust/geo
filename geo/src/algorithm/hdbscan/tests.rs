use super::*;
use crate::point;
use crate::wkt;
use approx::assert_relative_eq;

#[test]
fn test_empty_input() {
    let points: Vec<Point<f64>> = vec![];
    let result = points.hdbscan(5).unwrap();
    assert!(result.labels.is_empty());
    assert!(result.outlier_scores.is_empty());
}

#[test]
fn test_single_point() {
    // A single point cannot satisfy min_cluster_size=2, so this is an
    // out-of-range parameter error, not a silent all-noise result.
    let points = [point!(x: 0.0, y: 0.0)];
    assert_eq!(
        points.hdbscan(2),
        Err(HdbscanError::InvalidMinClusterSize {
            min_cluster_size: 2,
            n: 1,
        }),
    );
}

#[test]
fn test_min_cluster_size_too_large() {
    let points = [
        point!(x: 0.0, y: 0.0),
        point!(x: 1.0, y: 0.0),
        point!(x: 0.0, y: 1.0),
    ];
    assert_eq!(
        points.hdbscan(100),
        Err(HdbscanError::InvalidMinClusterSize {
            min_cluster_size: 100,
            n: 3,
        }),
    );
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

    let result_mp = mp.hdbscan(3).unwrap();
    let result_slice = mp.0.hdbscan(3).unwrap();

    assert_eq!(result_mp.labels, result_slice.labels);
    assert_eq!(result_mp.outlier_scores, result_slice.outlier_scores);
}

#[test]
fn test_identical_points() {
    let points: Vec<Point<f64>> = (0..10).map(|_| point!(x: 5.0, y: 5.0)).collect();
    let result = points.hdbscan(3).unwrap();
    // Identical points must end up with a single verdict: either all
    // clustered with the same label, or all noise.
    let first = result.labels[0];
    assert_eq!(result.labels, vec![first; 10]);
    // Coincident points produce zero-distance MST edges, which become
    // `lambda = infinity` in the condensed tree. Guard against that
    // propagating into GLOSH as NaN.
    for score in &result.outlier_scores {
        assert!(
            score.is_finite(),
            "GLOSH outlier score must be finite, got {score}"
        );
        assert!(
            (0.0..=1.0).contains(score),
            "GLOSH outlier score must be in [0, 1], got {score}"
        );
    }
}

#[test]
fn test_min_cluster_size_one() {
    // min_cluster_size < 2 is a parameter error.
    let points = [point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
    assert_eq!(
        points.hdbscan(1),
        Err(HdbscanError::InvalidMinClusterSize {
            min_cluster_size: 1,
            n: 2,
        }),
    );
}

#[test]
fn test_invalid_min_samples() {
    let points = [point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
    assert_eq!(
        points.hdbscan_with_min_samples(2, 0),
        Err(HdbscanError::InvalidMinSamples { min_samples: 0 }),
    );
}

#[test]
fn test_two_points() {
    let points = [point!(x: 0.0, y: 0.0), point!(x: 1.0, y: 1.0)];
    let result = points.hdbscan(2).unwrap();
    // With only 2 points and min_cluster_size=2, both must share the
    // same verdict: either the same cluster label, or both noise.
    assert_eq!(result.labels[0], result.labels[1]);
    assert_eq!(result.labels.len(), 2);
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
    let result = points.hdbscan(6).unwrap();

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
    // Point layout:
    //   indices  0..5:  group 1 near (1-3, 7-9)
    //   indices  5..9:  group 2 near (5-6, 3-4)
    //   indices 9..15:  group 3 near (8-9, 1-3)
    //   index   15:     outlier at (7, 8)
    let points: MultiPoint<f64> = wkt!(MULTIPOINT(
        1.0 9.0, 2.0 9.0, 1.0 8.0, 2.0 8.0, 3.0 7.0,
        5.0 4.0, 6.0 4.0, 5.0 3.0, 6.0 3.0,
        8.0 3.0, 9.0 3.0, 8.0 2.0, 9.0 2.0, 8.0 1.0, 9.0 1.0,
        7.0 8.0
    ));
    let result = points.hdbscan_with_min_samples(4, 4).unwrap();

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
    // Point layout:
    //   indices  0..8:  cluster A, dense 3x3-ish block
    //   indices  8..13: cluster B, 5 points near (7-9, 7-9)
    //   indices 13..18: cluster C, 5 points near (5-7, 1-3)
    //   index   18:     outlier at (8, 4)
    //   index   19:     outlier at (3, 3)
    let points: MultiPoint<f64> = wkt!(MULTIPOINT(
        2.0 9.0, 3.0 9.0, 2.0 8.0, 3.0 8.0, 2.0 7.0, 3.0 7.0, 1.0 8.0, 4.0 8.0,
        7.0 9.0, 7.0 8.0, 8.0 8.0, 8.0 7.0, 9.0 7.0,
        6.0 3.0, 5.0 2.0, 6.0 2.0, 7.0 2.0, 6.0 1.0,
        8.0 4.0,
        3.0 3.0
    ));
    let result = points.hdbscan_with_min_samples(5, 5).unwrap();

    // All scores should be in [0, 1]
    for (i, &score) in result.outlier_scores.iter().enumerate() {
        assert!(
            (0.0..=1.0).contains(&score),
            "outlier score for point {i} is {score}, expected [0, 1]"
        );
    }

    // Outlier points should have higher scores than cluster core members.
    // Compute average score for cluster A (indices 0..8) as a reference
    // for "core" cluster behaviour.
    let outlier1_score = result.outlier_scores[18];
    let outlier2_score = result.outlier_scores[19];
    let cluster_a_avg: f64 = result.outlier_scores[0..8].iter().sum::<f64>() / 8.0;

    // Pinned numeric values guard against scoring regressions.
    assert_relative_eq!(outlier1_score, 0.333_333_333_333_333_37, epsilon = 1e-10);
    assert_relative_eq!(outlier2_score, 1.0, epsilon = 1e-10);
    assert_relative_eq!(cluster_a_avg, 0.073_223_304_703_363_1, epsilon = 1e-10);

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

    let result = points.hdbscan(5).unwrap();

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

    let result = points.hdbscan(5).unwrap();

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
    let tree = BallTree::new(points.iter().copied());
    let core_data = CoreData {
        core_distances,
        knn_indices: vec![Vec::new(); points.len()],
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
    let tree = BallTree::new(points.iter().copied());
    let core_data = CoreData {
        core_distances,
        knn_indices: vec![Vec::new(); points.len()],
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

    let result = points.hdbscan_with_min_samples(5, 3).unwrap();

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

    let result = points.hdbscan_with_min_samples(15, 5).unwrap();

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
    let result = points.hdbscan(3).unwrap();

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

#[test]
fn compute_core_data_retains_knn_indices() {
    use crate::Point;
    use crate::algorithm::ball_tree::BallTree;

    let pts: Vec<Point<f64>> = (0..6).map(|i| Point::new(i as f64, 0.0)).collect();
    let tree = BallTree::new(pts.iter().copied());
    let core_data = super::compute_core_data(&tree, &pts, 3);

    assert_eq!(core_data.knn_indices.len(), 6);
    for (p, neighbours) in core_data.knn_indices.iter().enumerate() {
        assert_eq!(neighbours.len(), 3);
        assert!(
            neighbours.contains(&p),
            "knn_indices[{p}] should contain self"
        );
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
    let core_data = super::compute_core_data(&tree, &pts, 2);

    // All in one cross-component configuration: 0,1 in A; 2,3 in B.
    // k=2 means each point's kNN is {self, closest other}. For points 0,1
    // the closest other is each other (same component), so no seed for A.
    let point_components: Vec<usize> = vec![0, 0, 2, 2];
    let mut best_edges: Vec<Option<super::BestEdge<f64>>> = vec![None; 4];
    super::seed_best_edges(
        &core_data.knn_indices,
        &core_data.core_distances,
        &point_components,
        &mut best_edges,
    );
    assert!(
        best_edges[0].is_none(),
        "comp 0 has no cross-component kNN at k=2"
    );
    assert!(
        best_edges[2].is_none(),
        "comp 2 has no cross-component kNN at k=2"
    );

    // k=3 reaches across: kNN(1) = {1, 0, 2}, so point 1 has a
    // cross-component neighbour 2.
    let core_data_k3 = super::compute_core_data(&tree, &pts, 3);
    let mut best_edges_k3: Vec<Option<super::BestEdge<f64>>> = vec![None; 4];
    super::seed_best_edges(
        &core_data_k3.knn_indices,
        &core_data_k3.core_distances,
        &point_components,
        &mut best_edges_k3,
    );
    let e0 = best_edges_k3[0].expect("comp 0 should have a seed via point 1 -> point 2");
    let expected = core_data_k3.core_distances[1].max(core_data_k3.core_distances[2]);
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
    let knn_indices: Vec<Vec<usize>> = vec![
        vec![0, 1, 2],
        vec![1, 0, 2],
        vec![2, 1, 0],
        vec![3, 4, 0],
        vec![4, 3, 0],
    ];
    let core_distances: Vec<f64> = vec![5.0, 10.0, 3.0, 5.0, 5.0];
    // Point 0 alone in comp 0; points 1..=4 in comp 1 (root = point 1).
    let point_components: Vec<usize> = vec![0, 1, 1, 1, 1];
    let mut best_edges: Vec<Option<super::BestEdge<f64>>> = vec![None; 5];
    super::seed_best_edges(
        &knn_indices,
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
    let knn_indices: Vec<Vec<usize>> = vec![vec![0, 1], vec![1, 0]];
    let core_distances: Vec<f64> = vec![1.0, 2.0];
    let point_components: Vec<usize> = vec![0, 1];
    let mut best_edges: Vec<Option<super::BestEdge<f64>>> = vec![None; 2];
    super::seed_best_edges(
        &knn_indices,
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
    let points: Vec<Point<f64>> = vec![
        Point::new(0.0, 0.0),
        Point::new(1.0, 0.0),
        Point::new(2.0, 0.0),
        Point::new(3.0, 0.0),
        Point::new(4.0, 0.0),
        Point::new(5.0, 0.0),
    ];
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
