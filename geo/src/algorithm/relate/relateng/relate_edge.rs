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

pub(crate) const IS_FORWARD: bool = true;
pub(crate) const IS_REVERSE: bool = false;

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
    fn get(&self, direction: Direction) -> Option<CoordPos> {
        match direction {
            Direction::Left => self.left,
            Direction::Right => self.right,
            Direction::On => self.on,
        }
    }

    fn set(&mut self, direction: Direction, loc: CoordPos) {
        match direction {
            Direction::Left => self.left = Some(loc),
            Direction::Right => self.right = Some(loc),
            Direction::On => self.on = Some(loc),
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
    /// Creates an area edge (for dimension 2) or a line edge (otherwise).
    pub fn create(
        node_pt: Coord<F>,
        dir_pt: Coord<F>,
        input: InputIndex,
        dim: Dimensions,
        is_forward: bool,
    ) -> Self {
        if dim == Dimensions::TwoDimensional {
            Self::new_area(node_pt, dir_pt, input, is_forward)
        } else {
            Self::new_line(node_pt, dir_pt, input)
        }
    }

    /// An area edge: the input's interior lies on the left of a reverse
    /// (entering) edge and on the right of a forward (exiting) edge, with
    /// the edge itself on the boundary. Assumes CW shell orientation.
    pub fn new_area(
        node_pt: Coord<F>,
        dir_pt: Coord<F>,
        input: InputIndex,
        is_forward: bool,
    ) -> Self {
        let mut edge = Self::new_blank(node_pt, dir_pt);
        let (left, right) = if is_forward {
            (CoordPos::Outside, CoordPos::Inside)
        } else {
            (CoordPos::Inside, CoordPos::Outside)
        };
        *edge.label_mut(input) = EdgeLabel {
            dim: Some(Dimensions::TwoDimensional),
            left: Some(left),
            right: Some(right),
            on: Some(CoordPos::OnBoundary),
        };
        edge
    }

    /// A line edge: both sides are exterior and the edge line is interior.
    pub fn new_line(node_pt: Coord<F>, dir_pt: Coord<F>, input: InputIndex) -> Self {
        let mut edge = Self::new_blank(node_pt, dir_pt);
        *edge.label_mut(input) = EdgeLabel {
            dim: Some(Dimensions::OneDimensional),
            left: Some(CoordPos::Outside),
            right: Some(CoordPos::Outside),
            on: Some(CoordPos::Inside),
        };
        edge
    }

    /// An area edge with explicit locations for one input.
    pub fn new_with_locations(
        node_pt: Coord<F>,
        dir_pt: Coord<F>,
        input: InputIndex,
        loc_left: CoordPos,
        loc_right: CoordPos,
        loc_on: CoordPos,
    ) -> Self {
        let mut edge = Self::new_blank(node_pt, dir_pt);
        *edge.label_mut(input) = EdgeLabel {
            dim: Some(Dimensions::TwoDimensional),
            left: Some(loc_left),
            right: Some(loc_right),
            on: Some(loc_on),
        };
        edge
    }

    fn new_blank(node_pt: Coord<F>, dir_pt: Coord<F>) -> Self {
        Self {
            node_pt,
            dir_pt,
            a: EdgeLabel::default(),
            b: EdgeLabel::default(),
        }
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
    pub fn merge(&mut self, input: InputIndex, dim: Dimensions, is_forward: bool) {
        let (loc_edge, loc_left, loc_right) = if dim == Dimensions::TwoDimensional {
            let (left, right) = if is_forward {
                (CoordPos::Outside, CoordPos::Inside)
            } else {
                (CoordPos::Inside, CoordPos::Outside)
            };
            (CoordPos::OnBoundary, left, right)
        } else {
            (CoordPos::Inside, CoordPos::Outside, CoordPos::Outside)
        };

        if !self.is_known(input) {
            let label = self.label_mut(input);
            label.dim = Some(if loc_edge == CoordPos::OnBoundary {
                Dimensions::TwoDimensional
            } else {
                Dimensions::OneDimensional
            });
            label.on = Some(loc_edge);
            label.left = Some(loc_left);
            label.right = Some(loc_right);
            return;
        }

        self.merge_dim_edge_loc(input, loc_edge);
        self.merge_side_location(input, Direction::Left, loc_left);
        self.merge_side_location(input, Direction::Right, loc_right);
    }

    /// Area edges override line edges. Merging edges of the same dimension
    /// is a no-op for the dimension and On location, but merging an area
    /// edge into a line edge sets the dimension to 2 and the On location to
    /// boundary.
    fn merge_dim_edge_loc(&mut self, input: InputIndex, loc_edge: CoordPos) {
        let dim = if loc_edge == CoordPos::OnBoundary {
            Dimensions::TwoDimensional
        } else {
            Dimensions::OneDimensional
        };
        if dim == Dimensions::TwoDimensional
            && self.label(input).dim == Some(Dimensions::OneDimensional)
        {
            let label = self.label_mut(input);
            label.dim = Some(dim);
            label.on = Some(CoordPos::OnBoundary);
        }
    }

    /// Interior takes precedence over exterior on a side.
    fn merge_side_location(&mut self, input: InputIndex, direction: Direction, loc: CoordPos) {
        if self.label(input).get(direction) != Some(CoordPos::Inside) {
            self.label_mut(input).set(direction, loc);
        }
    }

    pub fn set_location(&mut self, input: InputIndex, direction: Direction, loc: CoordPos) {
        self.label_mut(input).set(direction, loc);
    }

    pub fn set_all_locations(&mut self, input: InputIndex, loc: CoordPos) {
        let label = self.label_mut(input);
        label.left = Some(loc);
        label.right = Some(loc);
        label.on = Some(loc);
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

    pub fn set_dim_locations(&mut self, input: InputIndex, dim: Dimensions, loc: CoordPos) {
        let label = self.label_mut(input);
        label.dim = Some(dim);
        label.left = Some(loc);
        label.right = Some(loc);
        label.on = Some(loc);
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
