use super::{LineIntersection, LineIntersector};
use crate::algorithm::kernels::{Kernel, Orientation, RobustKernel};
use crate::bounding_rect::BoundingRect;
use crate::contains::Contains;
use crate::intersects::Intersects;
use crate::num_traits::Zero;
use crate::{Coordinate, GeoFloat, Line, Rect};

/// A robust version of [LineIntersector](traits.LineIntersector).
#[derive(Clone)]
pub(crate) struct RobustLineIntersector;

impl RobustLineIntersector {
    pub fn new() -> RobustLineIntersector {
        RobustLineIntersector
    }
}

impl<F: GeoFloat> LineIntersector<F> for RobustLineIntersector {
    fn compute_intersection(&mut self, p: Line<F>, q: Line<F>) -> Option<LineIntersection<F>> {
        crate::algorithm::line_intersection::line_intersection(p, q)
    }
}

impl RobustLineIntersector {
    /// Computes the "edge distance" of an intersection point p along a segment.
    ///
    /// The edge distance is a metric of the point along the edge.
    /// The metric used is a robust and easy to compute metric function.
    /// It is _not_ equivalent to the usual Euclidean metric.
    /// It relies on the fact that either the x or the y ordinates of the
    /// points in the edge are unique, depending on whether the edge is longer in
    /// the horizontal or vertical direction.
    ///
    /// NOTE: This function may produce incorrect distances for inputs where p is not precisely
    /// on p1-p2 (E.g. p = (139,9) p1 = (139,10), p2 = (280,1) produces distance 0.0, which is
    /// incorrect.
    ///
    /// My hypothesis is that the function is safe to use for points which are the
    /// result of _rounding_ points which lie on the line,
    /// but not safe to use for _truncated_ points.
    pub fn compute_edge_distance<F: GeoFloat>(intersection: Coordinate<F>, line: Line<F>) -> F {
        let dx = (line.end.x - line.start.x).abs();
        let dy = (line.end.y - line.start.y).abs();

        let mut dist: F;
        if intersection == line.start {
            dist = F::zero();
        } else if intersection == line.end {
            if dx > dy {
                dist = dx;
            } else {
                dist = dy;
            }
        } else {
            let intersection_dx = (intersection.x - line.start.x).abs();
            let intersection_dy = (intersection.y - line.start.y).abs();
            if dx > dy {
                dist = intersection_dx;
            } else {
                dist = intersection_dy;
            }
            // hack to ensure that non-endpoints always have a non-zero distance
            if dist == F::zero() && intersection != line.start {
                dist = intersection_dx.max(intersection_dy);
            }
        }
        debug_assert!(
            !(dist == F::zero() && intersection != line.start),
            "Bad distance calculation"
        );
        dist
    }
}
