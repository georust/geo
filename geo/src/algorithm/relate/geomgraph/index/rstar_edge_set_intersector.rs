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
        let (tree, edges) = graph.tree_and_edges_mut();
        for (segment_0, segment_1) in tree.intersection_candidates_with_other_tree(&tree) {
            if check_for_self_intersecting_edges || segment_0.edge_idx != segment_1.edge_idx {
                if segment_1.edge_idx == segment_0.edge_idx {
                    let edge_0 = &mut edges[segment_0.edge_idx];
                    segment_intersector.add_intersections_against_self(
                        edge_0,
                        segment_0.segment_idx,
                        segment_1.segment_idx,
                    );
                } else {
                    // XXX: use get_many_mut when available.
                    let mi = segment_0.edge_idx.min(segment_1.edge_idx);
                    let mx = segment_0.edge_idx.max(segment_1.edge_idx);

                    assert!(mx > mi);

                    let (e0, e1) = edges.split_at_mut(mi + 1);

                    let edge_0 = &mut e0[mi];
                    let edge_1 = &mut e1[mx - (mi + 1)];

                    if segment_0.edge_idx > segment_1.edge_idx {
                        segment_intersector.add_intersections(
                            edge_1,
                            segment_0.segment_idx,
                            edge_0,
                            segment_1.segment_idx,
                        );
                    } else {
                        segment_intersector.add_intersections(
                            edge_0,
                            segment_0.segment_idx,
                            edge_1,
                            segment_1.segment_idx,
                        );
                    }
                }
            }
        }
    }

    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &mut GeometryGraph<'a, F>,
        graph_1: &mut GeometryGraph<'a, F>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let (tree_0, edges_0) = graph_0.tree_and_edges_mut();
        let (tree_1, edges_1) = graph_1.tree_and_edges_mut();

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
