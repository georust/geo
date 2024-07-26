use super::{
    node_map::{NodeFactory, NodeMap},
    CoordNode, CoordPos, Edge, Label,
};
use crate::{Coord, GeoFloat};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub(crate) struct PlanarGraphNode;

/// The basic node constructor does not allow for incident edges
impl<F> NodeFactory<F> for PlanarGraphNode
where
    F: GeoFloat,
{
    type Node = CoordNode<F>;
    fn create_node(coordinate: Coord<F>) -> Self::Node {
        CoordNode::new(coordinate)
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct PlanarGraph<F: GeoFloat> {
    pub(crate) nodes: NodeMap<F, PlanarGraphNode>,
    edges: Vec<Rc<RefCell<Edge<F>>>>,
}

impl<F: GeoFloat> PlanarGraph<F> {
    pub fn clone_for_arg_index(&self, from_arg_index: usize, to_arg_index: usize) -> Self {
        let mut graph = Self {
            nodes: self.nodes.clone(),
            // deep copy edges
            edges: self
                .edges
                .iter()
                .map(|e| Rc::new(RefCell::new(e.borrow().clone())))
                .collect(),
        };
        assert_eq!(from_arg_index, 0);
        if from_arg_index != to_arg_index {
            graph.swap_labels();
        }
        graph
    }

    fn swap_labels(&mut self) {
        for node in self.nodes.iter_mut() {
            node.swap_label_args();
        }
        for edge in &mut self.edges {
            edge.borrow_mut().swap_label_args();
        }
    }

    pub fn assert_eq_graph(&self, other: &Self) {
        assert_eq!(self.nodes, other.nodes);
        assert_eq!(self.edges, other.edges);
    }

    pub fn edges(&self) -> &[Rc<RefCell<Edge<F>>>] {
        &self.edges
    }

    pub fn new() -> Self {
        PlanarGraph {
            nodes: NodeMap::new(),
            edges: vec![],
        }
    }

    pub fn is_boundary_node(&self, geom_index: usize, coord: Coord<F>) -> bool {
        self.nodes
            .find(coord)
            .and_then(|node| node.label().on_position(geom_index))
            .map(|position| position == CoordPos::OnBoundary)
            .unwrap_or(false)
    }

    pub fn insert_edge(&mut self, edge: Edge<F>) {
        self.edges.push(Rc::new(RefCell::new(edge)));
    }

    pub fn add_node_with_coordinate(&mut self, coord: Coord<F>) -> &mut CoordNode<F> {
        self.nodes.insert_node_with_coordinate(coord)
    }

    pub fn boundary_nodes(&self, geom_index: usize) -> impl Iterator<Item = &CoordNode<F>> {
        self.nodes.iter().filter(move |node| {
            matches!(
                node.label().on_position(geom_index),
                Some(CoordPos::OnBoundary)
            )
        })
    }
}
