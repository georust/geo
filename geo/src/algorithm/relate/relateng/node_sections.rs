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
        let has_input = |input| self.sections.iter().any(|ns| ns.input() == input);
        has_input(InputIndex::A) && has_input(InputIndex::B)
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
        for run in self.sections.chunk_by(|a, b| a.is_same_polygon(b)) {
            // Multiple polygon sections incident at the node are converted
            // to maximal-ring structure.
            if run.len() > 1 && run[0].is_area() {
                node.add_edges_list(&polygon_node_converter::convert(run.to_vec()));
            } else {
                node.add_edges_list(run);
            }
        }
        node
    }
}
