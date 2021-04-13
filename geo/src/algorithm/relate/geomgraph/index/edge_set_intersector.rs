use super::super::Edge;
use super::SegmentIntersector;
use crate::{Coordinate, GeoFloat};

use std::cell::RefCell;
use std::rc::Rc;

pub(crate) trait EdgeSetIntersector<F: GeoFloat> {
    /// Compute all intersections between the edges within a set, recording those intersections on
    /// the intersecting edges.
    ///
    /// `edges`: the set of edges to check. Mutated to record any intersections.
    /// `check_for_self_intersecting_edges`: if false, an edge is not checked for intersections with itself.
    /// `segment_intersector`: the SegmentIntersector to use
    fn compute_intersections_within_set(
        &mut self,
        edges: &[Rc<RefCell<Edge<F>>>],
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    );

    /// Compute all intersections between two sets of edges, recording those intersections on
    /// the intersecting edges.
    fn compute_intersections_between_sets(
        &mut self,
        edges0: &[Rc<RefCell<Edge<F>>>],
        edges1: &[Rc<RefCell<Edge<F>>>],
        segment_intersector: &mut SegmentIntersector<F>,
    );
}
