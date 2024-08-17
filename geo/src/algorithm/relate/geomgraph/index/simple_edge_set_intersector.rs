use super::super::{Edge, GeometryGraph};
use super::{EdgeSetIntersector, SegmentIntersector};
use crate::GeoFloat;

use std::cell::RefCell;
// use std::rc::Rc;
use std::sync::Arc;

pub(crate) struct SimpleEdgeSetIntersector;

impl SimpleEdgeSetIntersector {
    pub fn new() -> Self {
        SimpleEdgeSetIntersector
    }

    fn compute_intersects<F: GeoFloat>(
        &self,
        edge0: &mut Edge<F>,
        edge1: &mut Edge<F>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edge0_coords_len = edge0.coords().len() - 1;
        let edge1_coords_len = edge1.coords().len() - 1;
        for i0 in 0..edge0_coords_len {
            for i1 in 0..edge1_coords_len {
                segment_intersector.add_intersections(edge0, i0, edge1, i1);
            }
        }
    }

    fn compute_intersects_against_self<F: GeoFloat>(
        &self,
        edge0: &mut Edge<F>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edge0_coords_len = edge0.coords().len() - 1;
        for i0 in 0..edge0_coords_len {
            for i1 in 0..edge0_coords_len {
                segment_intersector.add_intersections_against_self(edge0, i0, i1);
            }
        }
    }
}

impl<F: GeoFloat> EdgeSetIntersector<F> for SimpleEdgeSetIntersector {
    fn compute_intersections_within_set(
        &self,
        graph: &mut GeometryGraph<F>,
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edges = graph.edges_mut();
        for i in 0..edges.len() {
            let (e0, e1) = edges.split_at_mut(i + 1);
            let (e0, edge0) = e0.split_at_mut(i);
            debug_assert_eq!(edge0.len(), 1);
            let edge0 = &mut edge0[0];

            if check_for_self_intersecting_edges {
                self.compute_intersects_against_self(edge0, segment_intersector);
            }

            for edge1 in e0.iter_mut().chain(e1) {
                self.compute_intersects(edge0, edge1, segment_intersector);
            }
        }
    }

    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &mut GeometryGraph<'a, F>,
        graph_1: &mut GeometryGraph<'a, F>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edges_0 = graph_0.edges_mut();
        let edges_1 = graph_1.edges_mut();

        for edge0 in edges_0 {
            for edge1 in &mut *edges_1 {
                self.compute_intersects(edge0, edge1, segment_intersector);
            }
        }
    }
}
