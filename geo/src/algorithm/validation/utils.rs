use crate::algorithm::sweep::{Cross, Intersections};
use crate::{
    Coord, CoordFloat, GeoFloat, Line, LineIntersection, LineString, RemoveRepeatedPoints,
};
use robust::{Coord as RobustCoord, orient2d};

pub(crate) fn check_coord_is_not_finite<T: CoordFloat>(geom: &Coord<T>) -> bool {
    if geom.x.is_finite() && geom.y.is_finite() {
        return false;
    }
    true
}

pub(crate) fn robust_check_points_are_collinear<T: CoordFloat>(
    p0: &Coord<T>,
    p1: &Coord<T>,
    p2: &Coord<T>,
) -> bool {
    orient2d(
        RobustCoord {
            x: p0.x.to_f64().unwrap(),
            y: p0.y.to_f64().unwrap(),
        },
        RobustCoord {
            x: p1.x.to_f64().unwrap(),
            y: p1.y.to_f64().unwrap(),
        },
        RobustCoord {
            x: p2.x.to_f64().unwrap(),
            y: p2.y.to_f64().unwrap(),
        },
    ) == 0.
}

pub(crate) fn check_too_few_points<T: CoordFloat>(geom: &LineString<T>, is_ring: bool) -> bool {
    let n_pts = if is_ring { 4 } else { 2 };
    if geom.remove_repeated_points().0.len() < n_pts {
        return true;
    }
    false
}

pub(crate) fn linestring_has_self_intersection<F: GeoFloat>(geom: &LineString<F>) -> bool {
    // The sweep yields every pair of segments that touch.
    // Only neighbours that meet at their shared point are allowed.
    Intersections::from_iter(geom.lines().map(RingSegment))
        .any(|(a, b, intersection)| !touch_at_shared_point(a.0, b.0, intersection))
}

/// A ring segment for the sweep.
#[derive(Clone, Copy)]
struct RingSegment<F: GeoFloat>(Line<F>);

impl<F: GeoFloat> Cross for RingSegment<F> {
    type Scalar = F;

    fn line(&self) -> Line<F> {
        self.0
    }
}

/// Whether `a` and `b` are neighbours that touch only at their shared point.
/// Any other touch, like the overlap at a spike, is a self-intersection.
fn touch_at_shared_point<F: GeoFloat>(
    a: Line<F>,
    b: Line<F>,
    intersection: LineIntersection<F>,
) -> bool {
    let point = match intersection {
        LineIntersection::SinglePoint { intersection, .. } => intersection,
        // A zero-length segment from a repeated point overlaps its neighbour in one point.
        LineIntersection::Collinear { intersection } if intersection.start == intersection.end => {
            intersection.start
        }
        LineIntersection::Collinear { .. } => return false,
    };
    (a.end == b.start && point == a.end) || (a.start == b.end && point == a.start)
}
