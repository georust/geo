//! A direction-edge at a relate node, carrying the topological locations
//! (Left/Right/On) of each input geometry for the edge.
//!
//! Port of JTS `RelateEdge`. Where JTS holds a back-pointer to the owning
//! `RelateNode`, this port stores the node coordinate directly. The
//! per-input field pairs (`aLocLeft`/`bLocLeft`, ...) collapse into one
//! [`EdgeLabel`] per input, and the JTS `LOC_UNKNOWN`/`DIM_UNKNOWN`
//! sentinels become `Option`.

use std::cmp::Ordering;

use crate::algorithm::polygon_node_topology::compare_angle;
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::Direction;
use crate::{Coord, GeoFloat};

use super::topology_predicate::InputIndex;

/// The role of an edge in the input that contributes it. Area edges assume
/// CW shell orientation.
#[derive(Debug, Clone, Copy)]
pub(crate) enum EdgeKind {
    /// Both sides are exterior and the edge line is interior.
    Line,
    /// An edge entering the node: the area interior lies on its left, with
    /// the edge itself on the boundary.
    AreaEntering,
    /// An edge exiting the node: the area interior lies on its right, with
    /// the edge itself on the boundary.
    AreaExiting,
}

/// The topological locations of one input geometry for an edge: the left
/// and right sides and the edge line itself. `None` is "not yet known".
#[derive(Debug, Clone, Copy, Default)]
struct EdgeLabel {
    dim: Option<Dimensions>,
    left: Option<CoordPos>,
    right: Option<CoordPos>,
    on: Option<CoordPos>,
}

impl EdgeLabel {
    fn new(kind: EdgeKind) -> Self {
        use CoordPos::{Inside, OnBoundary, Outside};
        let (dim, on, left, right) = match kind {
            EdgeKind::Line => (Dimensions::OneDimensional, Inside, Outside, Outside),
            EdgeKind::AreaEntering => (Dimensions::TwoDimensional, OnBoundary, Inside, Outside),
            EdgeKind::AreaExiting => (Dimensions::TwoDimensional, OnBoundary, Outside, Inside),
        };
        Self {
            dim: Some(dim),
            left: Some(left),
            right: Some(right),
            on: Some(on),
        }
    }

    fn get(&self, direction: Direction) -> Option<CoordPos> {
        match direction {
            Direction::Left => self.left,
            Direction::Right => self.right,
            Direction::On => self.on,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RelateEdge<F: GeoFloat> {
    node_pt: Coord<F>,
    dir_pt: Coord<F>,
    a: EdgeLabel,
    b: EdgeLabel,
}

impl<F: GeoFloat> RelateEdge<F> {
    pub fn create(node_pt: Coord<F>, dir_pt: Coord<F>, input: InputIndex, kind: EdgeKind) -> Self {
        let mut edge = Self {
            node_pt,
            dir_pt,
            a: EdgeLabel::default(),
            b: EdgeLabel::default(),
        };
        *edge.label_mut(input) = EdgeLabel::new(kind);
        edge
    }

    fn label(&self, input: InputIndex) -> &EdgeLabel {
        match input {
            InputIndex::A => &self.a,
            InputIndex::B => &self.b,
        }
    }

    fn label_mut(&mut self, input: InputIndex) -> &mut EdgeLabel {
        match input {
            InputIndex::A => &mut self.a,
            InputIndex::B => &mut self.b,
        }
    }

    /// Compares this edge's direction point angle at the node against
    /// another direction point.
    pub fn compare_to_edge(&self, edge_dir_pt: Coord<F>) -> Ordering {
        compare_angle(self.node_pt, self.dir_pt, edge_dir_pt)
    }

    /// Merges a coincident edge contribution for one input into this edge.
    ///
    /// Area edges override line edges: merging an area edge into a line
    /// edge sets the dimension to 2 and the On location to boundary. On a
    /// side, interior takes precedence over exterior.
    pub fn merge(&mut self, input: InputIndex, kind: EdgeKind) {
        let new = EdgeLabel::new(kind);
        if !self.is_known(input) {
            *self.label_mut(input) = new;
            return;
        }
        let label = self.label_mut(input);
        if new.dim == Some(Dimensions::TwoDimensional)
            && label.dim == Some(Dimensions::OneDimensional)
        {
            label.dim = new.dim;
            label.on = new.on;
        }
        if label.left != Some(CoordPos::Inside) {
            label.left = new.left;
        }
        if label.right != Some(CoordPos::Inside) {
            label.right = new.right;
        }
    }

    pub fn set_unknown_locations(&mut self, input: InputIndex, loc: CoordPos) {
        let label = self.label_mut(input);
        if label.left.is_none() {
            label.left = Some(loc);
        }
        if label.right.is_none() {
            label.right = Some(loc);
        }
        if label.on.is_none() {
            label.on = Some(loc);
        }
    }

    /// Marks the edge as lying in the interior of the input's area.
    pub fn set_area_interior(&mut self, input: InputIndex) {
        let label = self.label_mut(input);
        label.left = Some(CoordPos::Inside);
        label.right = Some(CoordPos::Inside);
        label.on = Some(CoordPos::Inside);
    }

    /// The location of the input at the given edge position, or `None` when
    /// it has not been determined yet.
    pub fn location(&self, input: InputIndex, direction: Direction) -> Option<CoordPos> {
        self.label(input).get(direction)
    }

    pub fn is_interior(&self, input: InputIndex, direction: Direction) -> bool {
        self.location(input, direction) == Some(CoordPos::Inside)
    }

    /// Whether the input's contribution to this edge is known (its
    /// dimension has been set).
    pub fn is_known(&self, input: InputIndex) -> bool {
        self.label(input).dim.is_some()
    }
}
