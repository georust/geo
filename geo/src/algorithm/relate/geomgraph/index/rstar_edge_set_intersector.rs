use super::super::{Edge, GeometryGraph};
use super::{EdgeSetIntersector, Segment, SegmentIntersector};
use crate::GeoFloat;

use std::cell::RefCell;
use std::rc::Rc;

use rstar::{RTree, RTreeNum};

pub(crate) struct RStarEdgeSetIntersector;

impl<F> EdgeSetIntersector<F> for RStarEdgeSetIntersector
where
    F: GeoFloat + RTreeNum,
{
    fn compute_intersections_within_set(
        &self,
        graph: &mut GeometryGraph<F>,
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let tree = graph.get_or_build_tree();
        let edges = graph.edges_mut();
        for (segment_0, segment_1) in tree.intersection_candidates_with_other_tree(&tree) {
            if check_for_self_intersecting_edges || segment_0.edge_idx != segment_1.edge_idx {
                // use get_many_mut when available.
                assert!(segment_1.edge_idx > segment_0.edge_idx);
                let (e0, e1) = edges.split_at_mut(segment_0.edge_idx+1);
                let edge_0 = &mut e0[segment_0.edge_idx];
                let edge_1 = &mut e1[segment_1.edge_idx-segment_0.edge_idx+1];
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
        graph_0: &mut GeometryGraph<'a, F>,
        graph_1: &mut GeometryGraph<'a, F>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let tree_0 = graph_0.get_or_build_tree();
        let tree_1 = graph_1.get_or_build_tree();

        let edges_0 = graph_0.edges_mut();
        let edges_1 = graph_1.edges_mut();

        for (segment_0, segment_1) in tree_0.intersection_candidates_with_other_tree(&tree_1) {
            let edge_0 = &mut edges_0[segment_0.edge_idx];
            let edge_1 = &mut edges_1[segment_1.edge_idx];
            segment_intersector.add_intersections(
                edge_0,
                segment_0.segment_idx,
                edge_1,
                segment_1.segment_idx,
            );
        }
    }
}
