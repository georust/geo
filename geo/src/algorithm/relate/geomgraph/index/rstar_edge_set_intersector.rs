use super::super::GeometryGraph;
use super::{EdgeSetIntersector, SegmentIntersector};
use crate::GeoFloat;

use rstar::RTreeNum;

pub(crate) struct RStarEdgeSetIntersector;

impl<F> EdgeSetIntersector<F> for RStarEdgeSetIntersector
where
    F: GeoFloat + RTreeNum,
{
    fn compute_intersections_within_set(
        &self,
        graph: &GeometryGraph<F>,
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edges = graph.edges();

        for (segment_0, segment_1) in graph
            .tree()
            .intersection_candidates_with_other_tree(graph.tree())
        {
            let (edge_idx_0, segment_idx_0) = segment_0.payload;
            let (edge_idx_1, segment_idx_1) = segment_1.payload;
            if check_for_self_intersecting_edges || edge_idx_0 != edge_idx_1 {
                let edge_0 = &edges[edge_idx_0];
                let edge_1 = &edges[edge_idx_1];
                segment_intersector.add_intersections(edge_0, segment_idx_0, edge_1, segment_idx_1);
            }
        }
    }

    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &GeometryGraph<'a, F>,
        graph_1: &GeometryGraph<'a, F>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edges_0 = graph_0.edges();
        let edges_1 = graph_1.edges();

        let tree_0 = graph_0.tree();
        let tree_1 = graph_1.tree();

        for (segment_0, segment_1) in tree_0.intersection_candidates_with_other_tree(tree_1) {
            let (edge_idx_0, segment_idx_0) = segment_0.payload;
            let (edge_idx_1, segment_idx_1) = segment_1.payload;
            let edge_0 = &edges_0[edge_idx_0];
            let edge_1 = &edges_1[edge_idx_1];
            segment_intersector.add_intersections(edge_0, segment_idx_0, edge_1, segment_idx_1);
        }
    }
}
