use super::*;

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
