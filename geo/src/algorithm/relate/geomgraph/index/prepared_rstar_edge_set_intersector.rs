use super::super::{Edge, GeometryGraph, PlanarGraph};
use super::{EdgeSetIntersector, Segment, SegmentIntersector};
use crate::GeoFloat;

use std::cell::RefCell;
use std::rc::Rc;

use rstar::RTree;

pub(crate) struct PreparedRStarEdgeSetIntersector;

impl<F> EdgeSetIntersector<F> for PreparedRStarEdgeSetIntersector
where
    F: GeoFloat + rstar::RTreeNum,
{
    fn compute_intersections_within_set<'a>(
        &self,
        graph: &GeometryGraph<'a, F>,
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edges = graph.edges();

        for (segment_0, segment_1) in graph
            .tree()
            .unwrap()
            .intersection_candidates_with_other_tree(&graph.tree().unwrap())
        {
            if check_for_self_intersecting_edges || segment_0.edge_idx != segment_1.edge_idx {
                let edge_0 = &edges[segment_0.edge_idx];
                let edge_1 = &edges[segment_1.edge_idx];
                segment_intersector.add_intersections(
                    edge_0,
                    segment_0.segment_idx,
                    edge_1,
                    segment_1.segment_idx,
                );
            }
        }
    }

    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &GeometryGraph<'a, F>,
        graph_1: &GeometryGraph<'a, F>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let tree_0 = graph_0.tree().unwrap();
        let tree_1 = graph_1.tree().unwrap();

        let edges_0 = graph_0.edges();
        let edges_1 = graph_1.edges();

        for (segment_0, segment_1) in tree_0.intersection_candidates_with_other_tree(&tree_1) {
            let edge_0 = &edges_0[segment_0.edge_idx];
            let edge_1 = &edges_1[segment_1.edge_idx];
            segment_intersector.add_intersections(
                edge_0,
                segment_0.segment_idx,
                edge_1,
                segment_1.segment_idx,
            );
        }
    }
}
