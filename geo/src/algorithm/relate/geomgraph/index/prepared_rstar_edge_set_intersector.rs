use super::super::{Edge, GeometryGraph, PlanarGraph};
use super::{EdgeSetIntersector, Segment, SegmentIntersector};
use crate::{Coordinate, GeoFloat};

use std::cell::RefCell;
use std::rc::Rc;

use rstar::RTree;

pub(crate) struct PreparedRStarEdgeSetIntersector<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    tree: RTree<Segment<F>>,
}

impl<F> PreparedRStarEdgeSetIntersector<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    pub(crate) fn new(tree: RTree<Segment<F>>) -> Self {
        PreparedRStarEdgeSetIntersector { tree }
    }
}

impl<F> EdgeSetIntersector<F> for PreparedRStarEdgeSetIntersector<F>
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

        for (segment_0, segment_1) in self
            .tree
            .intersection_candidates_with_other_tree(&self.tree)
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
