use super::super::{CoordNode, Edge, LineIntersection, LineIntersector};
use crate::{Coordinate, GeoFloat, Line};

use std::cell::{Ref, RefCell};

/// Computes the intersection of line segments and adds the intersection to the [`Edge`s] containing
/// the segments.
pub(crate) struct SegmentIntersector<F>
where
    F: GeoFloat,
{
    // Though JTS leaves this abstract - we might consider hard coding it to a RobustLineIntersector
    line_intersector: Box<dyn LineIntersector<F>>,
    edges_are_from_same_geometry: bool,
    proper_intersection_point: Option<Coordinate<F>>,
    has_proper_interior_intersection: bool,
    boundary_nodes: Option<[Vec<CoordNode<F>>; 2]>,
}

impl<F> SegmentIntersector<F>
where
    F: GeoFloat,
{
    fn is_adjacent_segments(i1: usize, i2: usize) -> bool {
        let difference = if i1 > i2 { i1 - i2 } else { i2 - i1 };
        difference == 1
    }

    pub fn new(
        line_intersector: Box<dyn LineIntersector<F>>,
        edges_are_from_same_geometry: bool,
    ) -> SegmentIntersector<F> {
        SegmentIntersector {
            line_intersector,
            edges_are_from_same_geometry,
            has_proper_interior_intersection: false,
            proper_intersection_point: None,
            boundary_nodes: None,
        }
    }
    pub fn set_boundary_nodes(
        &mut self,
        boundary_nodes_0: Vec<CoordNode<F>>,
        boundary_nodes_1: Vec<CoordNode<F>>,
    ) {
        debug_assert!(
            self.boundary_nodes.is_none(),
            "Should only set boundaries between geometries once"
        );
        self.boundary_nodes = Some([boundary_nodes_0, boundary_nodes_1]);
    }

    pub fn has_proper_intersection(&self) -> bool {
        self.proper_intersection_point.is_some()
    }

    pub fn has_proper_interior_intersection(&self) -> bool {
        self.has_proper_interior_intersection
    }

    /// A trivial intersection is an apparent self-intersection which in fact is simply the point
    /// shared by adjacent line segments.  Note that closed edges require a special check for the
    /// point shared by the beginning and end segments.
    fn is_trivial_intersection(
        &self,
        intersection: LineIntersection<F>,
        edge0: &RefCell<Edge<F>>,
        segment_index_0: usize,
        edge1: &RefCell<Edge<F>>,
        segment_index_1: usize,
    ) -> bool {
        if edge0.as_ptr() != edge1.as_ptr() {
            return false;
        }

        if matches!(intersection, LineIntersection::Collinear { .. }) {
            return false;
        }

        if Self::is_adjacent_segments(segment_index_0, segment_index_1) {
            return true;
        }

        let edge0 = edge0.borrow();
        if edge0.is_closed() {
            // first and last coords in a ring are adjacent
            let max_segment_index = edge0.coords().len() - 1;
            if (segment_index_0 == 0 && segment_index_1 == max_segment_index)
                || (segment_index_1 == 0 && segment_index_0 == max_segment_index)
            {
                return true;
            }
        }

        false
    }

    pub fn add_intersections(
        &mut self,
        edge0: &RefCell<Edge<F>>,
        segment_index_0: usize,
        edge1: &RefCell<Edge<F>>,
        segment_index_1: usize,
    ) {
        // avoid a segment spuriously "intersecting" with itself
        if edge0.as_ptr() == edge1.as_ptr() && segment_index_0 == segment_index_1 {
            return;
        }

        let line_0 = Line::new(
            edge0.borrow().coords()[segment_index_0],
            edge0.borrow().coords()[segment_index_0 + 1],
        );
        let line_1 = Line::new(
            edge1.borrow().coords()[segment_index_1],
            edge1.borrow().coords()[segment_index_1 + 1],
        );

        let intersection = self.line_intersector.compute_intersection(line_0, line_1);

        if intersection.is_none() {
            return;
        }
        let intersection = intersection.unwrap();

        if !self.edges_are_from_same_geometry {
            edge0.borrow_mut().mark_as_unisolated();
            edge1.borrow_mut().mark_as_unisolated();
        }
        if !self.is_trivial_intersection(
            intersection,
            edge0,
            segment_index_0,
            edge1,
            segment_index_1,
        ) {
            if self.edges_are_from_same_geometry || !intersection.is_proper() {
                // In the case of self-noding, `edge0` might alias `edge1`, so it's imperative that
                // the mutable borrow's are short lived and do not overlap.
                edge0
                    .borrow_mut()
                    .add_intersections(intersection, line_0, segment_index_0);

                edge1
                    .borrow_mut()
                    .add_intersections(intersection, line_1, segment_index_1);
            }
            if let LineIntersection::SinglePoint {
                is_proper: true,
                intersection: intersection_coord,
            } = intersection
            {
                self.proper_intersection_point = Some(intersection_coord);

                if !self.is_boundary_point(&intersection_coord, &self.boundary_nodes) {
                    self.has_proper_interior_intersection = true
                }
            }
        }
    }

    fn is_boundary_point(
        &self,
        intersection: &Coordinate<F>,
        boundary_nodes: &Option<[Vec<CoordNode<F>>; 2]>,
    ) -> bool {
        match &boundary_nodes {
            Some(boundary_nodes) => boundary_nodes
                .iter()
                .flatten()
                .any(|node| intersection == node.coordinate()),
            None => false,
        }
    }
}
