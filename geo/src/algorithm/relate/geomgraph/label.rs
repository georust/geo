use super::{CoordPos, Direction, TopologyPosition};

use std::fmt;

/// A GeometryGraph has components (nodes and edges) which are labeled with their topological
/// relations to the geometries.
///
/// More precisely, each `Label` holds a `TopologyPosition` for each geometry that states whether
/// the node or edge being labeled occurs `Inside`, `Outside`, or `OnBoundary` of the geometry.
///
/// For lines and points, a `TopologyPosition` tracks only an `On` position,
/// while areas have positions for `On`, `Left`, and `Right`.
///
/// If the component has *no* incidence with one of the geometries, than the `Label`'s
/// `TopologyPosition` for that geometry is called `empty`.
#[derive(Clone)]
pub(crate) struct Label {
    geometry_topologies: [TopologyPosition; 2],
}

impl fmt::Debug for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Label {{ A: {:?}, B: {:?} }}",
            &self.geometry_topologies[0], &self.geometry_topologies[1]
        )
    }
}

impl Label {
    /// Construct an empty `Label` for relating a 1-D line or 0-D point to both geometries.
    pub fn empty_line_or_point() -> Label {
        Label {
            geometry_topologies: [
                TopologyPosition::empty_line_or_point(),
                TopologyPosition::empty_line_or_point(),
            ],
        }
    }

    /// Construct an empty `Label` for relating a 2-D area to both geometries.
    pub fn empty_area() -> Self {
        Self {
            geometry_topologies: [
                TopologyPosition::empty_area(),
                TopologyPosition::empty_area(),
            ],
        }
    }

    /// Construct a `Label` initialized with `position` for the geometry specified by
    /// `geom_index`.
    ///
    /// The label's position for the other geometry will be initialized as empty.
    pub fn new(geom_index: usize, position: TopologyPosition) -> Self {
        let mut label = match position {
            TopologyPosition::LineOrPoint { .. } => Self::empty_line_or_point(),
            TopologyPosition::Area { .. } => Self::empty_area(),
        };
        label.geometry_topologies[geom_index] = position;
        label
    }

    pub fn flip(&mut self) {
        self.geometry_topologies[0].flip();
        self.geometry_topologies[1].flip();
    }

    pub fn position(&self, geom_index: usize, direction: Direction) -> Option<CoordPos> {
        self.geometry_topologies[geom_index].get(direction)
    }

    pub fn on_position(&self, geom_index: usize) -> Option<CoordPos> {
        self.geometry_topologies[geom_index].get(Direction::On)
    }

    pub fn set_position(&mut self, geom_index: usize, direction: Direction, position: CoordPos) {
        self.geometry_topologies[geom_index].set_position(direction, position);
    }

    pub fn set_on_position(&mut self, geom_index: usize, position: CoordPos) {
        self.geometry_topologies[geom_index].set_position(Direction::On, position);
    }

    pub fn set_all_positions(&mut self, geom_index: usize, position: CoordPos) {
        self.geometry_topologies[geom_index].set_all_positions(position)
    }

    pub fn set_all_positions_if_empty(&mut self, geom_index: usize, postion: CoordPos) {
        self.geometry_topologies[geom_index].set_all_positions_if_empty(postion)
    }

    pub fn geometry_count(&self) -> usize {
        self.geometry_topologies
            .iter()
            .filter(|location| !location.is_empty())
            .count()
    }

    pub fn is_empty(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_empty()
    }

    pub fn is_any_empty(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_any_empty()
    }

    pub fn is_area(&self) -> bool {
        self.geometry_topologies[0].is_area() || self.geometry_topologies[1].is_area()
    }

    pub fn is_geom_area(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_area()
    }

    pub fn is_line(&self, geom_index: usize) -> bool {
        self.geometry_topologies[geom_index].is_line()
    }
}
