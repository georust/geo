use super::super::Edge;
use super::{EdgeSetIntersector, SegmentIntersector};
use crate::GeoFloat;

use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct SimpleEdgeSetIntersector;

impl SimpleEdgeSetIntersector {
    pub fn new() -> Self {
        SimpleEdgeSetIntersector
    }

    fn compute_intersects<F: GeoFloat>(
        &mut self,
        edge0: &Rc<RefCell<Edge<F>>>,
        edge1: &Rc<RefCell<Edge<F>>>,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        let edge0_coords_len = edge0.borrow().coords().len() - 1;
        let edge1_coords_len = edge1.borrow().coords().len() - 1;
        for i0 in 0..edge0_coords_len {
            for i1 in 0..edge1_coords_len {
                segment_intersector.add_intersections(edge0, i0, edge1, i1);
            }
        }
    }
}

impl<F: GeoFloat> EdgeSetIntersector<F> for SimpleEdgeSetIntersector {
    fn compute_intersections_within_set(
        &mut self,
        edges: &[Rc<RefCell<Edge<F>>>],
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        for edge0 in edges.iter() {
            for edge1 in edges.iter() {
                if check_for_self_intersecting_edges || edge0.as_ptr() != edge1.as_ptr() {
                    self.compute_intersects(edge0, edge1, segment_intersector);
                }
            }
        }
    }

    fn compute_intersections_between_sets(
        &mut self,
        edges0: &[Rc<RefCell<Edge<F>>>],
        edges1: &[Rc<RefCell<Edge<F>>>],
        segment_intersector: &mut SegmentIntersector<F>,
    ) {
        for edge0 in edges0 {
            for edge1 in edges1 {
                self.compute_intersects(edge0, edge1, segment_intersector);
            }
        }
    }
}
