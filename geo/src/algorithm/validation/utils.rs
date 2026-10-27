use crate::algorithm::sweep::{Cross, Intersections};
use crate::{Coord, CoordFloat, GeoFloat, Line, LineString, RemoveRepeatedPoints};
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
    // A sweep finds every pair of segments that touch.
    // Neighbouring segments always touch at their shared point, so they don't count.
    Intersections::from_iter(geom.lines().map(RingSegment))
        .any(|(a, b, _)| a.0.start != b.0.end && a.0.end != b.0.start)
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
