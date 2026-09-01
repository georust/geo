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
//! and its other callers (RelateNG, coverage validation).

use std::cmp::Ordering;

use crate::kernels::{Kernel, Orientation};
use crate::{Coord, GeoNum};

/// Utility functions for working with quadrants of the cartesian plane,
/// which are labeled as follows:
/// ```ignore
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

/// Checks if four segments at a node cross.
///
/// The segments lie in two different rings, or in different sections of one
/// ring. The node is topologically valid if the rings do not cross. If any
/// segments are collinear, the test returns `false`.
///
/// `a0`/`a1` are the segment endpoints adjacent to the node in one ring,
/// `b0`/`b1` those in the other ring.
// Consumed by the RelateNG port (see RELATENG_PLAN.md); remove the allow when
// it lands.
#[allow(dead_code)]
pub(crate) fn is_crossing<T: GeoNum>(
    node_pt: Coord<T>,
    a0: Coord<T>,
    a1: Coord<T>,
    b0: Coord<T>,
    b1: Coord<T>,
) -> bool {
    let (a_lo, a_hi) = if is_angle_greater(node_pt, a0, a1) {
        (a1, a0)
    } else {
        (a0, a1)
    };
    // The edges cross if b0 and b1 lie on opposite sides of the sector
    // between a_lo and a_hi. A collinear edge is reported as not crossing.
    let comp_between_0 = compare_between(node_pt, b0, a_lo, a_hi);
    if comp_between_0 == Ordering::Equal {
        return false;
    }
    let comp_between_1 = compare_between(node_pt, b1, a_lo, a_hi);
    if comp_between_1 == Ordering::Equal {
        return false;
    }
    comp_between_0 != comp_between_1
}

/// Tests whether the segment `node_pt -> b` lies in the interior of the
/// corner formed by the segments `a0 -> node_pt -> a1`.
///
/// The ring interior is assumed to be on the right of the corner (a CW shell
/// or a CCW hole). The test segment must not be collinear with the corner
/// segments.
// Consumed by the RelateNG port (see RELATENG_PLAN.md); remove the allow when
// it lands.
#[allow(dead_code)]
pub(crate) fn is_interior_segment<T: GeoNum>(
    node_pt: Coord<T>,
    a0: Coord<T>,
    a1: Coord<T>,
    b: Coord<T>,
) -> bool {
    let (a_lo, a_hi, is_interior_between) = if is_angle_greater(node_pt, a0, a1) {
        (a1, a0, false)
    } else {
        (a0, a1, true)
    };
    is_between(node_pt, b, a_lo, a_hi) == is_interior_between
}

/// Tests if the angle of vector `origin -> p` is greater than that of vector
/// `origin -> q`.
fn is_angle_greater<T: GeoNum>(origin: Coord<T>, p: Coord<T>, q: Coord<T>) -> bool {
    compare_angle(origin, p, q) == Ordering::Greater
}

/// Tests if an edge `p` is between edges `e0` and `e1`, where all edges
/// originate at `origin`. The "inside" of `e0` and `e1` is the arc that does
/// not include the positive X-axis at the origin. The edges are assumed to be
/// distinct (non-collinear).
fn is_between<T: GeoNum>(origin: Coord<T>, p: Coord<T>, e0: Coord<T>, e1: Coord<T>) -> bool {
    if !is_angle_greater(origin, p, e0) {
        return false;
    }
    !is_angle_greater(origin, p, e1)
}

/// Compares whether an edge `p` is between (`Greater`), outside (`Less`), or
/// collinear with (`Equal`) the edges `e0` and `e1`, where all edges
/// originate at `origin`. The "inside" of `e0` and `e1` is the arc that does
/// not include the positive X-axis at the origin.
fn compare_between<T: GeoNum>(
    origin: Coord<T>,
    p: Coord<T>,
    e0: Coord<T>,
    e1: Coord<T>,
) -> Ordering {
    let comp_0 = compare_angle(origin, p, e0);
    if comp_0 == Ordering::Equal {
        return Ordering::Equal;
    }
    let comp_1 = compare_angle(origin, p, e1);
    if comp_1 == Ordering::Equal {
        return Ordering::Equal;
    }
    if comp_0 == Ordering::Greater && comp_1 == Ordering::Less {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

fn quadrant<T: GeoNum>(origin: Coord<T>, p: Coord<T>) -> Option<Quadrant> {
    let d = p - origin;
    Quadrant::new(d.x, d.y)
}

#[cfg(test)]
mod tests {
    // Tests ported from JTS PolygonNodeTopologyTest.java (master, ab57bff).
    // Fixtures are three-point linestrings; the middle point is the node.
    use super::*;
    use crate::LineString;
    use crate::wkt;

    fn check_crossing(a: LineString, b: LineString, expected: bool) {
        let (a, b) = (a.0, b.0);
        assert_eq!(a[1], b[1], "fixture invariant: shared node");
        assert_eq!(is_crossing(a[1], a[0], a[2], b[0], b[2]), expected);
    }

    fn check_interior(a: LineString, b: LineString, expected: bool) {
        let (a, b) = (a.0, b.0);
        assert_eq!(a[1], b[0], "fixture invariant: shared node");
        assert_eq!(is_interior_segment(a[1], a[0], a[2], b[1]), expected);
    }

    // In JTS this test is named testNonCrossing, but its helper asserts that
    // the segments DO cross; the name is corrected here.
    #[test]
    fn test_crossing() {
        check_crossing(
            wkt!(LINESTRING (500. 1000., 1000. 1000., 1000. 1500.)),
            wkt!(LINESTRING (1000. 500., 1000. 1000., 500. 1500.)),
            true,
        );
    }

    #[test]
    fn test_non_crossing_quadrant2() {
        check_crossing(
            wkt!(LINESTRING (500. 1000., 1000. 1000., 1000. 1500.)),
            wkt!(LINESTRING (300. 1200., 1000. 1000., 500. 1500.)),
            false,
        );
    }

    #[test]
    fn test_non_crossing_quadrant4() {
        check_crossing(
            wkt!(LINESTRING (500. 1000., 1000. 1000., 1000. 1500.)),
            wkt!(LINESTRING (1000. 500., 1000. 1000., 1500. 1000.)),
            false,
        );
    }

    #[test]
    fn test_non_crossing_collinear() {
        check_crossing(
            wkt!(LINESTRING (3. 1., 5. 5., 9. 9.)),
            wkt!(LINESTRING (2. 1., 5. 5., 9. 9.)),
            false,
        );
    }

    #[test]
    fn test_non_crossing_both_collinear() {
        check_crossing(
            wkt!(LINESTRING (3. 1., 5. 5., 9. 9.)),
            wkt!(LINESTRING (3. 1., 5. 5., 9. 9.)),
            false,
        );
    }

    #[test]
    fn test_interior_segment() {
        check_interior(
            wkt!(LINESTRING (5. 9., 5. 5., 9. 5.)),
            wkt!(LINESTRING (5. 5., 0. 0.)),
            true,
        );
    }

    #[test]
    fn test_exterior_segment() {
        check_interior(
            wkt!(LINESTRING (5. 9., 5. 5., 9. 5.)),
            wkt!(LINESTRING (5. 5., 9. 9.)),
            false,
        );
    }

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
