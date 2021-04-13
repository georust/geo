use super::{CoordNode, CoordPos, EdgeEnd};
use crate::{Coordinate, GeoFloat};

use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

/// A map of nodes, indexed by the coordinate of the node
pub(crate) struct NodeMap<F, NF>
where
    F: GeoFloat,
    NF: NodeFactory<F>,
{
    map: BTreeMap<NodeKey<F>, NF::Node>,
    _node_factory: PhantomData<NF>,
}

/// Creates the node stored in `NodeMap`
pub(crate) trait NodeFactory<F: GeoFloat> {
    type Node;
    fn create_node(coordinate: Coordinate<F>) -> Self::Node;
}

impl<F, NF> fmt::Debug for NodeMap<F, NF>
where
    F: GeoFloat,
    NF: NodeFactory<F>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeMap")
            .field("map.len()", &self.map.len())
            .finish()
    }
}

#[derive(Clone)]
struct NodeKey<F: GeoFloat>(Coordinate<F>);

impl<F: GeoFloat> std::cmp::Ord for NodeKey<F> {
    fn cmp(&self, other: &NodeKey<F>) -> std::cmp::Ordering {
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());
        crate::utils::lex_cmp(&self.0, &other.0)
    }
}

impl<F: GeoFloat> std::cmp::PartialOrd for NodeKey<F> {
    fn partial_cmp(&self, other: &NodeKey<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: GeoFloat> std::cmp::PartialEq for NodeKey<F> {
    fn eq(&self, other: &NodeKey<F>) -> bool {
        debug_assert!(!self.0.x.is_nan());
        debug_assert!(!self.0.y.is_nan());
        debug_assert!(!other.0.x.is_nan());
        debug_assert!(!other.0.y.is_nan());
        self.0 == other.0
    }
}

impl<F: GeoFloat> std::cmp::Eq for NodeKey<F> {}

impl<F, NF> NodeMap<F, NF>
where
    F: GeoFloat,
    NF: NodeFactory<F>,
{
    pub fn new() -> Self {
        NodeMap {
            map: BTreeMap::new(),
            _node_factory: PhantomData,
        }
    }
    /// Adds a `NF::Node` with the given `Coordinate`.
    ///
    /// Note: Coordinates must be non-NaN.
    pub fn insert_node_with_coordinate(&mut self, coord: Coordinate<F>) -> &mut NF::Node {
        debug_assert!(
            !coord.x.is_nan() && !coord.y.is_nan(),
            "NaN coordinates are not supported"
        );
        let key = NodeKey(coord);
        self.map
            .entry(key)
            .or_insert_with(|| NF::create_node(coord))
    }

    /// returns the `NF::Node`, if any, matching `coord`
    pub fn find(&self, coord: Coordinate<F>) -> Option<&NF::Node> {
        self.map.get(&NodeKey(coord))
    }

    /// Iterates across `NF::Node`s in lexical order of their `Coordinate`
    pub fn iter(&self) -> impl Iterator<Item = &NF::Node> {
        self.map.values()
    }

    /// Iterates across `NF::Node`s in lexical order of their `Coordinate`
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut NF::Node> {
        self.map.values_mut()
    }

    /// Iterates across `NF::Node`s in lexical order of their `Coordinate`
    pub fn into_iter(self) -> impl Iterator<Item = NF::Node> {
        self.map.into_iter().map(|(_k, v)| v)
    }
}
