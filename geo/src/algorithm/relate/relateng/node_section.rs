//! A computed node along with the incident edges on either side of it.
//!
//! Port of JTS `NodeSection`.

use std::cmp::Ordering;

use crate::algorithm::polygon_node_topology::compare_angle;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::node_map::NodeKey;
use crate::{Coord, GeoFloat};

use super::topology_predicate::InputIndex;

/// Represents a computed node along with the incident edges on either side
/// of it (if they exist). This captures the information about a node in a
/// geometry component required to determine the component's contribution to
/// the node topology. A node in an area geometry always has edges on both
/// sides of the node. A node in a linear geometry may have one or other
/// incident edge missing, if the node occurs at an endpoint of the line.
///
/// The edges of an area node must be provided with CW-shell orientation
/// (the JTS norm). This must be enforced by the caller.
///
/// Where JTS stores a reference to the parent polygonal `Geometry`
/// (compared by identity), this port stores `polygonal_id`: the index of
/// the parent polygonal element in the owning `RelateGeometry`, or `None`
/// when the section is not on a polygon boundary.
#[derive(Debug, Clone)]
pub(crate) struct NodeSection<F: GeoFloat> {
    input: InputIndex,
    dim: Dimensions,
    id: i32,
    ring_id: i32,
    polygonal_id: Option<usize>,
    is_node_at_vertex: bool,
    node_pt: Coord<F>,
    v0: Option<Coord<F>>,
    v1: Option<Coord<F>>,
}

impl<F: GeoFloat> NodeSection<F> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        input: InputIndex,
        dim: Dimensions,
        id: i32,
        ring_id: i32,
        polygonal_id: Option<usize>,
        is_node_at_vertex: bool,
        v0: Option<Coord<F>>,
        node_pt: Coord<F>,
        v1: Option<Coord<F>>,
    ) -> Self {
        Self {
            input,
            dim,
            id,
            ring_id,
            polygonal_id,
            is_node_at_vertex,
            node_pt,
            v0,
            v1,
        }
    }

    /// The incident vertex: `0` is the entering edge vertex, `1` the
    /// exiting edge vertex. Either may be absent at a line end.
    pub fn vertex(&self, i: usize) -> Option<Coord<F>> {
        if i == 0 { self.v0 } else { self.v1 }
    }

    pub fn node_pt(&self) -> Coord<F> {
        self.node_pt
    }

    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    /// The parent polygonal element this section is part of, or `None` if
    /// the section is not on a polygon boundary.
    pub fn polygonal_id(&self) -> Option<usize> {
        self.polygonal_id
    }

    pub fn is_shell(&self) -> bool {
        self.ring_id == 0
    }

    pub fn is_area(&self) -> bool {
        self.dim == Dimensions::TwoDimensional
    }

    pub fn is_area_area(a: &Self, b: &Self) -> bool {
        a.is_area() && b.is_area()
    }

    pub fn input(&self) -> InputIndex {
        self.input
    }

    pub fn is_same_geometry(&self, other: &Self) -> bool {
        self.input == other.input
    }

    pub fn is_same_polygon(&self, other: &Self) -> bool {
        self.input == other.input && self.id == other.id
    }

    pub fn is_node_at_vertex(&self) -> bool {
        self.is_node_at_vertex
    }

    pub fn is_proper(&self) -> bool {
        !self.is_node_at_vertex
    }

    pub fn is_proper_pair(a: &Self, b: &Self) -> bool {
        a.is_proper() && b.is_proper()
    }

    /// Compares sections by the angle the entering edge makes with the
    /// positive X axis (JTS `NodeSection.EdgeAngleComparator`). Both
    /// sections must be at the same node and have an entering edge.
    pub fn compare_by_edge_angle(&self, other: &Self) -> Ordering {
        compare_angle(
            self.node_pt,
            self.vertex(0).expect("section must have an entering edge"),
            other.vertex(0).expect("section must have an entering edge"),
        )
    }

    /// Compares node sections by parent geometry, dimension, element id,
    /// ring id, and edge vertices. Sections are assumed to be at the same
    /// node point.
    pub fn compare(&self, other: &Self) -> Ordering {
        // Sort A before B.
        self.input
            .cmp(&other.input)
            .then_with(|| self.dim.cmp(&other.dim))
            .then_with(|| self.id.cmp(&other.id))
            .then_with(|| self.ring_id.cmp(&other.ring_id))
            .then_with(|| compare_optional_coord(&self.v0, &other.v0))
            .then_with(|| compare_optional_coord(&self.v1, &other.v1))
    }
}

impl<F: GeoFloat> PartialEq for NodeSection<F> {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == Ordering::Equal
    }
}

impl<F: GeoFloat> Eq for NodeSection<F> {}

/// An absent vertex sorts lower than any present one; present vertices
/// compare lexicographically (as JTS `Coordinate.compareTo`).
fn compare_optional_coord<F: GeoFloat>(a: &Option<Coord<F>>, b: &Option<Coord<F>>) -> Ordering {
    a.map(NodeKey).cmp(&b.map(NodeKey))
}

/// The index after `index` in a cyclic sequence of `len` items.
pub(super) fn next_index(len: usize, index: usize) -> usize {
    if index + 1 >= len { 0 } else { index + 1 }
}

/// The index before `index` in a cyclic sequence of `len` items.
pub(super) fn prev_index(len: usize, index: usize) -> usize {
    if index > 0 { index - 1 } else { len - 1 }
}
