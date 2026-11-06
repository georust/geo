//! Functions that compute topological information about nodes (ring
//! intersections) in polygonal geometry.
//!
//! All comparisons order direction vectors by their angle counter-clockwise
//! from the positive X-axis at a shared origin. The order is computed without
//! trigonometry: the quadrants of the two vectors decide the order when they
//! differ, and a robust orientation predicate decides it when the vectors
//! share a quadrant. This gives a total order that is exact for all
//! representable inputs.
//!
//! This is a port of [JTS's `PolygonNodeTopology` as of `ab57bff`](https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/algorithm/PolygonNodeTopology.java).
//! The angular comparison itself predates this module: it is the computation
//! that `relate::geomgraph::EdgeEndKey` uses to sort edge ends around a node,
//! extracted here so one implementation serves both the old relate machinery
//! and RelateNG.

use std::cmp::Ordering;

use crate::kernels::{Kernel, Orientation};
use crate::{Coord, GeoNum};

/// Utility functions for working with quadrants of the cartesian plane,
/// which are labeled as follows:
/// ```text
///          (+)
///        NW ┃ NE
///    (-) ━━━╋━━━━ (+)
///        SW ┃ SE
///          (-)
/// ```
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Eq)]
pub(crate) enum Quadrant {
    NE,
    NW,
    SW,
    SE,
}

impl Quadrant {
    pub fn new<F: GeoNum>(dx: F, dy: F) -> Option<Quadrant> {
        if dx.is_zero() && dy.is_zero() {
            return None;
        }

        match (dy >= F::zero(), dx >= F::zero()) {
            (true, true) => Quadrant::NE,
            (true, false) => Quadrant::NW,
            (false, false) => Quadrant::SW,
            (false, true) => Quadrant::SE,
        }
        .into()
    }
}

/// Compares the angles of two vectors relative to the positive X-axis at
/// their origin. Angles increase counter-clockwise from the X-axis.
///
/// Returns `Ordering::Greater` when vector `origin -> p` has a greater angle
/// than vector `origin -> q`, `Ordering::Less` when it has a smaller angle,
/// and `Ordering::Equal` when the vectors are collinear.
///
/// A zero-length vector has no quadrant; the comparison then falls through to
/// the orientation predicate alone.
pub(crate) fn compare_angle<T: GeoNum>(origin: Coord<T>, p: Coord<T>, q: Coord<T>) -> Ordering {
    let quadrant_p = quadrant(origin, p);
    let quadrant_q = quadrant(origin, q);

    match (quadrant_p, quadrant_q) {
        // If the vectors are in different quadrants, that decides the order.
        (Some(qp), Some(qq)) if qp > qq => Ordering::Greater,
        (Some(qp), Some(qq)) if qp < qq => Ordering::Less,
        // The vectors are in the same quadrant: P > Q if P is CCW of Q.
        _ => match T::Ker::orient2d(origin, q, p) {
            Orientation::CounterClockwise => Ordering::Greater,
            Orientation::Clockwise => Ordering::Less,
            Orientation::Collinear => Ordering::Equal,
        },
    }
}

fn quadrant<T: GeoNum>(origin: Coord<T>, p: Coord<T>) -> Option<Quadrant> {
    let d = p - origin;
    Quadrant::new(d.x, d.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Not from JTS: pin the total order of compare_angle across quadrant
    // boundaries and within a quadrant.
    #[test]
    fn test_compare_angle_total_order() {
        let origin = Coord::zero();
        let pos_x = coord(1., 0.);
        let ne_diag = coord(1., 1.);
        let pos_y = coord(0., 1.);
        let neg_x = coord(-1., 0.);
        let neg_y = coord(0., -1.);

        // Angle increases CCW from the positive X-axis.
        assert_eq!(compare_angle(origin, pos_x, ne_diag), Ordering::Less);
        assert_eq!(compare_angle(origin, ne_diag, pos_y), Ordering::Less);
        assert_eq!(compare_angle(origin, pos_y, neg_x), Ordering::Less);
        assert_eq!(compare_angle(origin, neg_x, neg_y), Ordering::Less);
        assert_eq!(compare_angle(origin, neg_y, pos_x), Ordering::Greater);
        // Collinear same-direction vectors of different length are Equal.
        assert_eq!(
            compare_angle(origin, ne_diag, coord(2., 2.)),
            Ordering::Equal
        );
    }

    fn coord(x: f64, y: f64) -> Coord<f64> {
        Coord { x, y }
    }
}
