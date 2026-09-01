//! Accumulates all the node sections at one node point and builds the
//! corresponding [`RelateNode`].
//!
//! Port of JTS `NodeSections`.

use crate::{Coord, GeoFloat};

use super::node_section::NodeSection;
use super::polygon_node_converter;
use super::relate_node::RelateNode;
use super::topology_predicate::InputIndex;

pub(crate) struct NodeSections<F: GeoFloat> {
    node_pt: Coord<F>,
    sections: Vec<NodeSection<F>>,
}

impl<F: GeoFloat> NodeSections<F> {
    pub fn new(node_pt: Coord<F>) -> Self {
        Self {
            node_pt,
            sections: Vec::new(),
        }
    }

    pub fn coordinate(&self) -> Coord<F> {
        self.node_pt
    }

    pub fn add_node_section(&mut self, section: NodeSection<F>) {
        self.sections.push(section);
    }

    /// Whether sections of both inputs meet at this node; only such nodes
    /// contribute topology.
    pub fn has_interaction_ab(&self) -> bool {
        let mut is_a = false;
        let mut is_b = false;
        for ns in &self.sections {
            match ns.input() {
                InputIndex::A => is_a = true,
                InputIndex::B => is_b = true,
            }
            if is_a && is_b {
                return true;
            }
        }
        false
    }

    /// The parent polygonal element id of the first section of the given
    /// input that lies on a polygon boundary, if any.
    pub fn polygonal_id(&self, input: InputIndex) -> Option<usize> {
        self.sections
            .iter()
            .filter(|ns| ns.input() == input)
            .find_map(|ns| ns.polygonal_id())
    }

    /// Builds the node with its edges, converting runs of sections that
    /// belong to one polygon from the touching-rings model to the
    /// self-touch model first.
    pub fn create_node(&mut self) -> RelateNode<F> {
        // Sort the sections so that lines come before areas and sections
        // from the same polygon are contiguous.
        self.sections.sort_by(|a, b| a.compare(b));

        let mut node = RelateNode::new(self.node_pt);
        let mut i = 0;
        while i < self.sections.len() {
            let ns = &self.sections[i];
            // If there are multiple polygon sections incident at the node,
            // convert them to maximal-ring structure.
            if ns.is_area() && self.has_multiple_polygon_sections(i) {
                let poly_sections = self.collect_polygon_sections(i);
                let count = poly_sections.len();
                let converted = polygon_node_converter::convert(poly_sections);
                node.add_edges_list(&converted);
                i += count;
            } else {
                // The most common case: a line or a single polygon ring
                // section.
                node.add_edges(ns);
                i += 1;
            }
        }
        node
    }

    fn has_multiple_polygon_sections(&self, i: usize) -> bool {
        // The last section can only be a single one.
        if i >= self.sections.len() - 1 {
            return false;
        }
        // Check if there are at least two sections for the same polygon.
        self.sections[i].is_same_polygon(&self.sections[i + 1])
    }

    /// The run of sections belonging to the same polygon as the section at
    /// `i`. Element ids are only unique within one input geometry.
    fn collect_polygon_sections(&self, i: usize) -> Vec<NodeSection<F>> {
        let poly_section = &self.sections[i];
        self.sections[i..]
            .iter()
            .take_while(|ns| poly_section.is_same_polygon(ns))
            .cloned()
            .collect()
    }
}
