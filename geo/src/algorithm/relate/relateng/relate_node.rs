//! A node in the topology graph of a relate evaluation, with its incident
//! edges kept sorted counter-clockwise by angle.
//!
//! Port of JTS `RelateNode`.

use std::cmp::Ordering;

use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::Direction;
use crate::{Coord, GeoFloat};

use super::node_section::{NodeSection, next_index, prev_index};
use super::relate_edge::{IS_FORWARD, IS_REVERSE, RelateEdge};
use super::topology_predicate::InputIndex;

pub(crate) struct RelateNode<F: GeoFloat> {
    node_pt: Coord<F>,
    /// The edges around the node in CCW order, ordered by their CCW angle
    /// with the positive X-axis.
    edges: Vec<RelateEdge<F>>,
}

impl<F: GeoFloat> RelateNode<F> {
    pub fn new(node_pt: Coord<F>) -> Self {
        Self {
            node_pt,
            edges: Vec::new(),
        }
    }

    pub fn edges(&self) -> &[RelateEdge<F>] {
        &self.edges
    }

    pub fn add_edges_list(&mut self, sections: &[NodeSection<F>]) {
        for ns in sections {
            self.add_edges(ns);
        }
    }

    pub fn add_edges(&mut self, ns: &NodeSection<F>) {
        match ns.dimension() {
            Dimensions::OneDimensional => {
                self.add_line_edge(ns.input(), ns.vertex(0));
                self.add_line_edge(ns.input(), ns.vertex(1));
            }
            Dimensions::TwoDimensional => {
                // Assumes node edges have CW orientation (the JTS norm):
                // the entering edge has the interior on its left, the
                // exiting edge on its right.
                let e0 = self.add_area_edge(ns.input(), ns.vertex(0), IS_REVERSE);
                let e1 = self.add_area_edge(ns.input(), ns.vertex(1), IS_FORWARD);

                // A well-formed area section has both edges; a degenerate
                // one (zero-length or absent edge) contributes nothing to
                // update. The indices are looked up only after both
                // insertions: the second insertion can shift the first
                // edge's position (JTS uses object identity here).
                if e0.is_some() && e1.is_some() {
                    let index0 = self.find_edge_index(ns.vertex(0).expect("edge was added"));
                    let index1 = self.find_edge_index(ns.vertex(1).expect("edge was added"));
                    self.update_edges_in_area(ns.input(), index0, index1);
                    self.update_if_area_prev(ns.input(), index0);
                    self.update_if_area_next(ns.input(), index1);
                }
            }
            _ => {}
        }
    }

    /// Marks every edge strictly between the entering and exiting edge of
    /// an area section as lying in the area interior.
    fn update_edges_in_area(&mut self, input: InputIndex, index_from: usize, index_to: usize) {
        let mut index = next_index(self.edges.len(), index_from);
        while index != index_to {
            self.edges[index].set_area_interior(input);
            index = next_index(self.edges.len(), index);
        }
    }

    fn update_if_area_prev(&mut self, input: InputIndex, index: usize) {
        let index_prev = prev_index(self.edges.len(), index);
        if self.edges[index_prev].is_interior(input, Direction::Left) {
            self.edges[index].set_area_interior(input);
        }
    }

    fn update_if_area_next(&mut self, input: InputIndex, index: usize) {
        let index_next = next_index(self.edges.len(), index);
        if self.edges[index_next].is_interior(input, Direction::Right) {
            self.edges[index].set_area_interior(input);
        }
    }

    /// The index of the edge with the given direction point. The edge list
    /// holds at most one edge per direction angle, so the lookup is exact.
    fn find_edge_index(&self, dir_pt: Coord<F>) -> usize {
        self.edges
            .iter()
            .position(|e| e.compare_to_edge(dir_pt) == Ordering::Equal)
            .expect("edge with this direction was added")
    }

    fn add_line_edge(&mut self, input: InputIndex, dir_pt: Option<Coord<F>>) -> Option<usize> {
        self.add_edge(input, dir_pt, Dimensions::OneDimensional, IS_REVERSE)
    }

    fn add_area_edge(
        &mut self,
        input: InputIndex,
        dir_pt: Option<Coord<F>>,
        is_forward: bool,
    ) -> Option<usize> {
        self.add_edge(input, dir_pt, Dimensions::TwoDimensional, is_forward)
    }

    /// Adds or merges an edge to the node, keeping the edge list sorted by
    /// angle. Returns the index of the created or merged edge, or `None`
    /// for a missing or zero-length direction point.
    fn add_edge(
        &mut self,
        input: InputIndex,
        dir_pt: Option<Coord<F>>,
        dim: Dimensions,
        is_forward: bool,
    ) -> Option<usize> {
        let dir_pt = dir_pt?;
        if self.node_pt == dir_pt {
            return None;
        }

        for i in 0..self.edges.len() {
            match self.edges[i].compare_to_edge(dir_pt) {
                Ordering::Equal => {
                    self.edges[i].merge(input, dim, is_forward);
                    return Some(i);
                }
                Ordering::Greater => {
                    // Found a further edge, so insert the new edge before
                    // it.
                    self.edges.insert(
                        i,
                        RelateEdge::create(self.node_pt, dir_pt, input, dim, is_forward),
                    );
                    return Some(i);
                }
                Ordering::Less => {}
            }
        }
        self.edges.push(RelateEdge::create(
            self.node_pt,
            dir_pt,
            input,
            dim,
            is_forward,
        ));
        Some(self.edges.len() - 1)
    }

    /// Computes the final topology for the edges around this node.
    ///
    /// Although nodes lie on the boundary of areas or the interior of
    /// lines, in a mixed GeometryCollection they may also lie in the
    /// interior of an area; this floods the locations of the sides and
    /// line to interior.
    pub fn finish(&mut self, is_area_interior_a: bool, is_area_interior_b: bool) {
        self.finish_node(InputIndex::A, is_area_interior_a);
        self.finish_node(InputIndex::B, is_area_interior_b);
    }

    fn finish_node(&mut self, input: InputIndex, is_area_interior: bool) {
        if is_area_interior {
            for edge in &mut self.edges {
                edge.set_area_interior(input);
            }
        } else {
            let start_index = self
                .edges
                .iter()
                .position(|e| e.is_known(input))
                // Only interacting nodes are finished, so an edge of each
                // input is present.
                .expect("finished node has an edge of each input");
            self.propagate_side_locations(input, start_index);
        }
    }

    /// Propagates the known side locations CCW around the node into edges
    /// whose locations for the input are still unknown.
    fn propagate_side_locations(&mut self, input: InputIndex, start_index: usize) {
        let mut curr_loc = self.edges[start_index]
            .location(input, Direction::Left)
            .expect("known edge has side locations");
        // Edges are stored in CCW order.
        let mut index = next_index(self.edges.len(), start_index);
        while index != start_index {
            let edge = &mut self.edges[index];
            edge.set_unknown_locations(input, curr_loc);
            curr_loc = edge
                .location(input, Direction::Left)
                .expect("locations were just set");
            index = next_index(self.edges.len(), index);
        }
    }

    /// Whether any edge has the input's exterior on either side. Supports
    /// `AdjacentEdgeLocator`.
    pub fn has_exterior_edge(&self, input: InputIndex) -> bool {
        self.edges.iter().any(|e| {
            e.location(input, Direction::Left) == Some(CoordPos::Outside)
                || e.location(input, Direction::Right) == Some(CoordPos::Outside)
        })
    }
}
