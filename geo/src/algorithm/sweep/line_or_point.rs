use std::{cmp::Ordering, ops::Deref};

use super::SweepPoint;
use crate::{
    line_intersection::line_intersection, Coordinate, GeoFloat, GeoNum, Kernel, Line,
    LineIntersection,
};

/// Either a line segment or a point.
///
/// The coordinates are ordered (see [`SweepPoint`]) and a line
/// segment must have distinct points (use the `Point` variant if the
/// coordinates are the equal).
#[derive(Clone, Copy)]
pub struct LineOrPoint<T: GeoNum> {
    left: SweepPoint<T>,
    right: SweepPoint<T>,
}

impl<T: GeoNum> std::fmt::Debug for LineOrPoint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple(if self.is_line() { "LPt" } else { "Pt" })
            .field(&self.left.x_y())
            .field(&self.right.x_y())
            .finish()
    }
}

impl<T: GeoNum> From<SweepPoint<T>> for LineOrPoint<T> {
    fn from(pt: SweepPoint<T>) -> Self {
        Self {
            left: pt,
            right: pt,
        }
    }
}

impl<T: GeoNum> From<(SweepPoint<T>, SweepPoint<T>)> for LineOrPoint<T> {
    fn from(pt: (SweepPoint<T>, SweepPoint<T>)) -> Self {
        let (start, end) = pt;
        match start.cmp(&end) {
            Ordering::Less => Self {
                left: start,
                right: end,
            },
            _ => Self {
                left: end,
                right: start,
            },
        }
    }
}

/// Convert from a [`Line`] ensuring end point ordering.
impl<T: GeoNum> From<Line<T>> for LineOrPoint<T> {
    fn from(l: Line<T>) -> Self {
        let start: SweepPoint<T> = l.start.into();
        let end = l.end.into();
        (start, end).into()
    }
}

/// Convert from a [`Coordinate`]
impl<T: GeoNum> From<Coordinate<T>> for LineOrPoint<T> {
    fn from(c: Coordinate<T>) -> Self {
        Self {
            left: c.into(),
            right: c.into(),
        }
    }
}

impl<T: GeoNum> LineOrPoint<T> {
    /// Checks if the variant is a line.
    #[inline]
    pub fn is_line(&self) -> bool {
        self.left != self.right
    }

    /// Return a [`Line`] representation of self.
    #[inline]
    pub fn line(&self) -> Line<T> {
        Line::new(*self.left, *self.right)
    }

    #[inline]
    pub fn left(&self) -> SweepPoint<T> {
        self.left
    }

    #[inline]
    pub fn right(&self) -> SweepPoint<T> {
        self.right
    }

    #[cfg(test)]
    pub fn coords_equal(&self, other: &LineOrPoint<T>) -> bool {
        self.is_line() == other.is_line() && self.end_points() == other.end_points()
    }

    #[inline]
    pub fn end_points(&self) -> (SweepPoint<T>, SweepPoint<T>) {
        (self.left, self.right)
    }

    pub fn new(left: SweepPoint<T>, right: SweepPoint<T>) -> Self {
        Self { left, right }
    }
}

/// Equality based on ordering defined for segments as per algorithm.
impl<T: GeoNum> PartialEq for LineOrPoint<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

/// Ordering defined for segments as per algorithm.
///
/// Requires the following conditions:
///
/// 1. If comparing two lines, both the left ends must be strictly
/// smaller than both right ends.
///
/// 2. A point is treated as a infinitesimal small vertical segment
/// centered at its coordinates.
impl<T: GeoNum> PartialOrd for LineOrPoint<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.is_line(), other.is_line()) {
            (false, false) => {
                if self.left == other.left {
                    Some(Ordering::Equal)
                } else {
                    // Unequal points do not satisfy pre-condition and
                    // can't be ordered.
                    None
                }
            }
            (false, true) => other.partial_cmp(self).map(Ordering::reverse),
            (true, false) => {
                let (p, q) = self.end_points();
                let r = other.left;
                if r > q || p > r {
                    return None;
                }
                Some(
                    T::Ker::orient2d(*p, *q, *r)
                        .as_ordering()
                        .then(Ordering::Greater),
                )
            }
            (true, true) => {
                let (p1, q1) = self.end_points();
                let (p2, q2) = other.end_points();
                if p1 > p2 {
                    return other.partial_cmp(self).map(Ordering::reverse);
                }
                if p1 >= q2 || p2 >= q1 {
                    return None;
                }

                // Assertion: p1 <= p2
                // Assertion: pi < q_j
                Some(
                    T::Ker::orient2d(*p1, *q1, *p2)
                        .as_ordering()
                        .then_with(|| T::Ker::orient2d(*p1, *q1, *q2).as_ordering()),
                )
            }
        }
    }
}

impl<T: GeoFloat> LineOrPoint<T> {
    /// Intersect a line with self and return a point, a overlapping segment or `None`.
    ///
    /// The `other` argument must be a line variant (debug builds will panic otherwise).
    pub fn intersect_line(&self, other: &Self) -> Option<Self> {
        debug_assert!(other.is_line(), "tried to intersect with a point variant!");

        let line = other.line();
        if !self.is_line() {
            let p = self.left;
            use crate::Intersects;
            if line.intersects(&*p) {
                Some(*self)
            } else {
                None
            }
        } else {
            line_intersection(self.line(), line).map(|l| match l {
                LineIntersection::SinglePoint {
                    intersection,
                    is_proper,
                } => {
                    let mut pt = intersection;
                    if is_proper && (&pt == self.left.deref()) {
                        if self.left.x == self.right.x {
                            pt.y = pt.y.next_after(T::infinity());
                        } else {
                            pt.x = pt.x.next_after(T::infinity());
                        }
                    }
                    pt.into()
                }
                LineIntersection::Collinear { intersection } => intersection.into(),
            })
        }
    }

    pub fn intersect_line_ordered(&self, other: &Self) -> Option<Self> {
        let ord = self.partial_cmp(other);
        match self.intersect_line(other) {
            Some(lp) if !lp.is_line() => {
                // NOTE: A key issue with using non-exact numbers (f64, etc.) in
                // this algo. is that line-intersection may return
                // counter-intuitive points.
                //
                // Specifically, this causes two issues:
                //
                // 1. The point of intersection r lies between the end-points in
                // the lexicographic ordering. However, with finite repr., the
                // line (1, 1) - (1 + eps, -1), where eps is ulp(1), does not
                // admit r that lies between the end-points. Further, the
                // end-points may be a very bad approximation to the actual
                // intersection points (eg. intersect with x-axis).
                //
                // We detect and force r to be greater than both end-points; the
                // other case is not easy to handle as the sweep has already
                // progressed to a p strictly > r already.
                //
                // 2. The more severe issue is that in general r may not lie
                // exactly on the line. Thus, with the segment stored on the
                // active-segments tree (B-Tree / Splay), this may in adverse
                // cases, cause the ordering between the segments to be
                // incorrect, hence invalidating the segments. This is not easy
                // to correct without a intrusive data-structure built
                // specifically for this algo., that can track the neighbors of
                // tree-nodes, and fix / report this issue. The crate
                // `btree-slab` seems like a great starting point.
                let pt = lp.left;
                let (mut x, y) = pt.x_y();

                let c = self.left;
                if x == c.x && y < c.y {
                    x = x.next_after(T::infinity());
                }

                let pt: SweepPoint<_> = Coordinate { x, y }.into();
                debug_assert!(
                    pt >= self.left,
                    "line intersection before first line: {pt:?}\n\tLine({lp1:?} - {lp2:?}) X Line({lp3:?} - {lp4:?})",
                    lp1 = self.left,
                    lp2 = self.right,
                    lp3 = other.left,
                    lp4 = other.right,
                );
                debug_assert!(
                    pt >= other.left,
                    "line intersection before second line: {pt:?}\n\tLine({lp1:?} - {lp2:?}) X Line({lp3:?} - {lp4:?})",
                    lp1 = self.left,
                    lp2 = self.right,
                    lp3 = other.left,
                    lp4 = other.right,
                );

                if let Some(ord) = ord {
                    let l1 = LineOrPoint::from((self.left, pt));
                    let l2 = LineOrPoint {
                        left: other.left,
                        right: pt,
                    };
                    let cmp = l1.partial_cmp(&l2).unwrap();
                    if l1.is_line() && l2.is_line() && cmp.then(ord) != ord {
                        debug!(
                            "ordering changed by intersection: {l1:?} {ord:?} {l2:?}",
                            l1 = self,
                            l2 = other
                        );
                        debug!("\tparts: {l1:?}, {l2:?}");
                        debug!("\tintersection: {pt:?} {cmp:?}");

                        // RM: This is a complicated intersection that is changing the ordering.
                        // Heuristic: approximate with a trivial intersection point that preserves the topology.
                        return Some(if self.left > other.left {
                            self.left.into()
                        } else {
                            other.left.into()
                        });
                    }
                }
                Some((*pt).into())
            }
            e => e,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use geo_types::{Coordinate, LineString};
    use wkt::ToWkt;

    use crate::{GeoFloat, GeoNum, HasKernel, Kernel};

    use super::LineOrPoint;

    // Used for debugging sweep fp issues
    #[test]
    #[ignore]
    fn check_ordering() {
        let pt_7 = Coordinate::from((-32.57812499999999, 241.33427773853316));
        let pt_8 = Coordinate::from((-36.11348070978957, 237.7989220287436));
        let pt_13 = Coordinate::from((-25.507080078124993, 248.40532266040816));
        let pt_14 = Coordinate::from((-36.48784219165816, 237.424560546875));
        let _pt_15 = Coordinate::from((4.4929199218750036, 196.44379843334184));
        let pt_16 = Coordinate::from((-36.048578439260666, 237.8638242992725));
        let pt_17 = Coordinate::from((3.545624214480127, 197.39109414073673));

        fn check_isection<T: GeoFloat>(abcd: [Coordinate<T>; 4]) -> Option<LineOrPoint<T>> {
            let l1 = LineOrPoint::from((abcd[0].into(), abcd[1].into()));
            let l2 = LineOrPoint::from((abcd[2].into(), abcd[3].into()));
            l1.intersect_line_ordered(&l2)
        }
        fn check_lines<T: GeoNum>(abcd: [Coordinate<T>; 4]) -> Ordering {
            let l1 = LineOrPoint::from((abcd[0].into(), abcd[1].into()));
            let l2 = LineOrPoint::from((abcd[2].into(), abcd[3].into()));
            l1.partial_cmp(&l2).unwrap()
        }

        eprintln!(
            "(14-17) {cmp:?} (14-16)",
            cmp = check_lines([pt_14, pt_17, pt_14, pt_16])
        );
        eprintln!(
            "(8-16) {cmp:?} (14-16)",
            cmp = check_lines([pt_8, pt_16, pt_14, pt_16]),
        );
        eprintln!(
            "(8-7) {cmp:?} (14-16)",
            cmp = check_lines([pt_8, pt_7, pt_14, pt_16]),
        );
        eprintln!(
            "(8-7) {cmp:?} (14-13)",
            cmp = check_lines([pt_8, pt_7, pt_14, pt_13]),
        );
        eprintln!(
            "(8-7) {isect:?} (14-13)",
            isect = check_isection([pt_8, pt_7, pt_14, pt_13]),
        );
        let l87 = LineString::new(vec![pt_8, pt_16, pt_7]);
        let lo = LineString::new(vec![pt_14, pt_16, pt_13]);
        eprintln!("l1: {}", l87.to_wkt());
        eprintln!("lo: {}", lo.to_wkt());

        eprintln!(
            "pred: {:?}",
            <f64 as HasKernel>::Ker::orient2d(pt_8, pt_7, pt_17)
        );
        eprintln!(
            "pred: {:?}",
            <f64 as HasKernel>::Ker::orient2d(pt_8, pt_14, pt_16)
        );
    }
}
