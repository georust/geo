use super::{CoordPos, Dimensions, EdgeEnd, EdgeEndBundleStar, IntersectionMatrix, Label};
use crate::{Coordinate, GeoFloat};

#[derive(Debug, Clone)]
pub(crate) struct CoordNode<F>
where
    F: GeoFloat,
{
    coordinate: Coordinate<F>,
    label: Label,
}

impl<F: GeoFloat> CoordNode<F> {
    pub(crate) fn label(&self) -> &Label {
        &self.label
    }

    pub(crate) fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub(crate) fn is_isolated(&self) -> bool {
        self.label.geometry_count() == 1
    }
}

impl<F> CoordNode<F>
where
    F: GeoFloat,
{
    pub fn new(coordinate: Coordinate<F>) -> CoordNode<F> {
        CoordNode {
            coordinate,
            label: Label::empty_line_or_point(),
        }
    }

    pub fn coordinate(&self) -> &Coordinate<F> {
        &self.coordinate
    }

    pub fn set_label_on_position(&mut self, geom_index: usize, position: CoordPos) {
        self.label.set_on_position(geom_index, position)
    }

    /// Updates the label of a node to BOUNDARY, obeying the mod-2 rule.
    pub fn set_label_boundary(&mut self, geom_index: usize) {
        let new_position = match self.label.on_position(geom_index) {
            Some(CoordPos::OnBoundary) => CoordPos::Inside,
            Some(CoordPos::Inside) => CoordPos::OnBoundary,
            None | Some(CoordPos::Outside) => CoordPos::OnBoundary,
        };
        self.label.set_on_position(geom_index, new_position);
    }

    // In JTS this method is implemented on a `GraphComponent` superclass, but since it's only used
    // by this one "subclass" I've implemented it directly on the node, rather than introducing
    // something like a `GraphComponent` trait
    pub fn update_intersection_matrix(&self, intersection_matrix: &mut IntersectionMatrix) {
        assert!(self.label.geometry_count() >= 2, "found partial label");
        intersection_matrix.set_at_least_if_in_both(
            self.label.on_position(0),
            self.label.on_position(1),
            Dimensions::ZeroDimensional,
        );
        debug!(
            "updated intersection_matrix: {:?} from node: {:?}",
            intersection_matrix, self
        );
    }
}
