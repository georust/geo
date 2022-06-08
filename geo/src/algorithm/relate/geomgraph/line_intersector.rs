pub(crate) use crate::LineIntersection;
use crate::{Coordinate, GeoFloat, Line};

pub(crate) trait LineIntersector<F: GeoFloat> {
    fn compute_intersection(&mut self, l1: Line<F>, l2: Line<F>) -> Option<LineIntersection<F>>;
}
