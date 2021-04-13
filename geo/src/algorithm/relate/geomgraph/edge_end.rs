use super::{CoordNode, Edge, Label, Quadrant};
use crate::{Coordinate, GeoFloat};

use std::cell::RefCell;
use std::fmt;

/// Models the end of an edge incident on a node.
///
/// EdgeEnds have a direction determined by the direction of the ray from the initial
/// point to the next point.
///
/// EdgeEnds are comparable by their EdgeEndKey, under the ordering
/// "a has a greater angle with the x-axis than b".
///
/// This ordering is used to sort EdgeEnds around a node.
///
/// This is based on [JTS's EdgeEnd as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/EdgeEnd.java)
#[derive(Clone, Debug)]
pub(crate) struct EdgeEnd<F>
where
    F: GeoFloat,
{
    label: Label,
    key: EdgeEndKey<F>,
}

#[derive(Clone)]
pub(crate) struct EdgeEndKey<F>
where
    F: GeoFloat,
{
    coord_0: Coordinate<F>,
    coord_1: Coordinate<F>,
    delta: Coordinate<F>,
    quadrant: Option<Quadrant>,
}

impl<F: GeoFloat> fmt::Debug for EdgeEndKey<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EdgeEndKey")
            .field(
                "coords",
                &format!("{:?} -> {:?}", &self.coord_0, &self.coord_1),
            )
            .field("quadrant", &self.quadrant)
            .finish()
    }
}

impl<F> EdgeEnd<F>
where
    F: GeoFloat,
{
    pub fn new(coord_0: Coordinate<F>, coord_1: Coordinate<F>, label: Label) -> EdgeEnd<F> {
        let delta = coord_1 - coord_0;
        let quadrant = Quadrant::new(delta.x, delta.y);
        EdgeEnd {
            label,
            key: EdgeEndKey {
                coord_0,
                coord_1,
                delta,
                quadrant,
            },
        }
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn label_mut(&mut self) -> &mut Label {
        &mut self.label
    }

    pub fn coordinate(&self) -> &Coordinate<F> {
        &self.key.coord_0
    }

    pub fn key(&self) -> &EdgeEndKey<F> {
        &self.key
    }
}

impl<F> std::cmp::Eq for EdgeEndKey<F> where F: GeoFloat {}

impl<F> std::cmp::PartialEq for EdgeEndKey<F>
where
    F: GeoFloat,
{
    fn eq(&self, other: &EdgeEndKey<F>) -> bool {
        self.delta == other.delta
    }
}

impl<F> std::cmp::PartialOrd for EdgeEndKey<F>
where
    F: GeoFloat,
{
    fn partial_cmp(&self, other: &EdgeEndKey<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F> std::cmp::Ord for EdgeEndKey<F>
where
    F: GeoFloat,
{
    fn cmp(&self, other: &EdgeEndKey<F>) -> std::cmp::Ordering {
        self.compare_direction(other)
    }
}

impl<F> EdgeEndKey<F>
where
    F: GeoFloat,
{
    pub(crate) fn compare_direction(&self, other: &EdgeEndKey<F>) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self.delta == other.delta {
            return Ordering::Equal;
        }

        match (self.quadrant, other.quadrant) {
            (Some(q1), Some(q2)) if q1 > q2 => Ordering::Greater,
            (Some(q1), Some(q2)) if q1 < q2 => Ordering::Less,
            _ => {
                use crate::algorithm::kernels::{Kernel, Orientation};
                match F::Ker::orient2d(other.coord_0, other.coord_1, self.coord_1) {
                    Orientation::Clockwise => Ordering::Less,
                    Orientation::CounterClockwise => Ordering::Greater,
                    Orientation::Collinear => Ordering::Equal,
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ord() {
        let fake_label = Label::empty_line_or_point();
        let edge_end_1 = EdgeEnd::new(
            Coordinate::zero(),
            Coordinate { x: 1.0, y: 1.0 },
            fake_label.clone(),
        );
        let edge_end_2 = EdgeEnd::new(
            Coordinate::zero(),
            Coordinate { x: 1.0, y: 1.0 },
            fake_label.clone(),
        );
        assert_eq!(
            edge_end_1.key().cmp(&edge_end_2.key()),
            std::cmp::Ordering::Equal
        );

        // edge_end_3 is clockwise from edge_end_1
        let edge_end_3 = EdgeEnd::new(
            Coordinate::zero(),
            Coordinate { x: 1.0, y: -1.0 },
            fake_label.clone(),
        );
        assert_eq!(
            edge_end_1.key().cmp(&edge_end_3.key()),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            edge_end_3.key().cmp(&edge_end_1.key()),
            std::cmp::Ordering::Greater
        );
    }
}
