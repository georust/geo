use super::super::{Edge, GeometryGraph};
use super::SegmentIntersector;
use crate::{Coord, GeoFloat};

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
        &self,
        graph: &GeometryGraph<F>,
        check_for_self_intersecting_edges: bool,
        segment_intersector: &mut SegmentIntersector<F>,
    );

    /// Compute all intersections between two sets of edges, recording those intersections on
    /// the intersecting edges.
    fn compute_intersections_between_sets<'a>(
        &self,
        graph_0: &GeometryGraph<'a, F>,
        graph_1: &GeometryGraph<'a, F>,
        segment_intersector: &mut SegmentIntersector<F>,
    );
}
