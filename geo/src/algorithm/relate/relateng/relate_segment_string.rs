//! A linear edge of a [`super::relate_geometry::RelateGeometry`]: a run of
//! coordinates from a line or a polygon ring, tagged with its source.
//!
//! Port of JTS `RelateSegmentString`. The JTS back-pointer to the parent
//! `RelateGeometry` is never read and is dropped; the parent polygonal
//! element is identified by its document-order id, as in
//! [`super::node_section::NodeSection`].

use crate::dimensions::Dimensions;
use crate::{Coord, GeoFloat};

use super::node_section::NodeSection;
use super::topology_predicate::InputIndex;

#[derive(Debug)]
pub(crate) struct RelateSegmentString<F: GeoFloat> {
    coords: Vec<Coord<F>>,
    input: InputIndex,
    dim: Dimensions,
    id: i32,
    ring_id: i32,
    polygonal_id: Option<usize>,
}

impl<F: GeoFloat> RelateSegmentString<F> {
    pub fn create_line(pts: Vec<Coord<F>>, input: InputIndex, element_id: i32) -> Self {
        Self::create(pts, input, Dimensions::OneDimensional, element_id, -1, None)
    }

    pub fn create_ring(
        pts: Vec<Coord<F>>,
        input: InputIndex,
        element_id: i32,
        ring_id: i32,
        polygonal_id: usize,
    ) -> Self {
        Self::create(
            pts,
            input,
            Dimensions::TwoDimensional,
            element_id,
            ring_id,
            Some(polygonal_id),
        )
    }

    fn create(
        mut pts: Vec<Coord<F>>,
        input: InputIndex,
        dim: Dimensions,
        id: i32,
        ring_id: i32,
        polygonal_id: Option<usize>,
    ) -> Self {
        // Remove repeated (consecutive equal) points.
        pts.dedup();
        Self {
            coords: pts,
            input,
            dim,
            id,
            ring_id,
            polygonal_id,
        }
    }

    pub fn input(&self) -> InputIndex {
        self.input
    }

    pub fn is_a(&self) -> bool {
        self.input == InputIndex::A
    }

    pub fn coords(&self) -> &[Coord<F>] {
        &self.coords
    }

    pub fn coord(&self, i: usize) -> Coord<F> {
        self.coords[i]
    }

    pub fn size(&self) -> usize {
        self.coords.len()
    }

    fn is_closed(&self) -> bool {
        self.coords.len() >= 2 && self.coords[0] == self.coords[self.coords.len() - 1]
    }

    /// Builds the node section for an intersection of this segment string
    /// at the given point on the given segment.
    pub fn create_node_section(&self, seg_index: usize, int_pt: Coord<F>) -> NodeSection<F> {
        let is_node_at_vertex =
            int_pt == self.coord(seg_index) || int_pt == self.coord(seg_index + 1);
        let prev = self.prev_vertex(seg_index, int_pt);
        let next = self.next_vertex(seg_index, int_pt);
        NodeSection::new(
            self.input,
            self.dim,
            self.id,
            self.ring_id,
            self.polygonal_id,
            is_node_at_vertex,
            prev,
            int_pt,
            next,
        )
    }

    /// The previous vertex before the intersection point, or `None` if none
    /// exists (at the start of an open line).
    fn prev_vertex(&self, seg_index: usize, pt: Coord<F>) -> Option<Coord<F>> {
        let seg_start = self.coord(seg_index);
        if seg_start != pt {
            return Some(seg_start);
        }
        // The point is at the segment start, so get the previous vertex.
        if seg_index > 0 {
            return Some(self.coord(seg_index - 1));
        }
        if self.is_closed() {
            // The vertex before the first point of a ring is the one before
            // the closing point.
            return Some(self.coord(self.coords.len() - 2));
        }
        None
    }

    /// The next vertex after the intersection point, or `None` if none
    /// exists (at the end of an open line).
    fn next_vertex(&self, seg_index: usize, pt: Coord<F>) -> Option<Coord<F>> {
        let seg_end = self.coord(seg_index + 1);
        if seg_end != pt {
            return Some(seg_end);
        }
        // The point is at the segment end, so get the next vertex.
        if seg_index + 2 < self.coords.len() {
            return Some(self.coord(seg_index + 2));
        }
        if self.is_closed() {
            // The vertex after the closing point of a ring is its second
            // point.
            return Some(self.coord(1));
        }
        // The segment string is not closed, so there is no next segment.
        None
    }

    /// Tests if a segment intersection point has this segment as its
    /// canonical containing segment.
    ///
    /// Segments are half-closed: they contain their start point but not
    /// their endpoint, except for the final segment of an open segment
    /// string, which contains its endpoint as well. This assigns vertices
    /// to a unique segment, so intersections lying exactly at segment
    /// endpoints are not double-counted.
    pub fn is_containing_segment(&self, seg_index: usize, pt: Coord<F>) -> bool {
        // An intersection at the segment start vertex is processed.
        if pt == self.coord(seg_index) {
            return true;
        }
        if pt == self.coord(seg_index + 1) {
            let is_final_segment = seg_index == self.coords.len() - 2;
            if self.is_closed() || !is_final_segment {
                return false;
            }
            // For the final segment, intersections with the final endpoint
            // are processed.
            return true;
        }
        // An interior intersection is processed.
        true
    }
}
