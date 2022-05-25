use std::cmp::Ordering;

use super::SweepPoint;
use crate::{
    line_intersection::line_intersection, Coordinate, GeoFloat, GeoNum, HasKernel, Kernel, Line,
    LineIntersection, Orientation,
};

/// Either a line segment or a point.
///
/// The coordinates are ordered (see [`SweepPoint`]) and a line
/// segment must have distinct points (use the `Point` variant if the
/// coordinates are the equal).
#[derive(Debug, Clone, Copy)]
pub(crate) struct LineOrPoint<T: GeoNum> {
    left: SweepPoint<T>,
    right: SweepPoint<T>,
}

/// Convert from a [`Line`] ensuring end point ordering.
impl<T: GeoNum> From<Line<T>> for LineOrPoint<T> {
    fn from(l: Line<T>) -> Self {
        let start: SweepPoint<T> = l.start.into();
        let end = l.end.into();
        match start.cmp(&end) {
            Ordering::Less => Self {
                left: start,
                right: end,
            },
            Ordering::Equal => Self {
                left: start,
                right: start,
            },
            Ordering::Greater => Self {
                left: end,
                right: start,
            },
        }
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

    /// Return a [`Line`] if it is one, otherwise `None`.
    #[inline]
    pub fn line(&self) -> Line<T> {
        Line::new(*self.left, *self.right)
    }

    #[inline]
    pub fn end_points(&self) -> (SweepPoint<T>, SweepPoint<T>) {
        (self.left, self.right)
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
/// 1. If comparing two lines, the both left ends must be strictly
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
                if p1 >= q2 || p2 >= q2 {
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
    /// The `other` argument must be a line variant (panics otherwise).
    pub fn intersect_line(&self, other: &Self) -> Option<Self> {
        debug_assert!(other.is_line(), "tried to intersect with a point variant!");

        let line = other.line();
        if !self.is_line() {
            let p = self.left;
            if <T as HasKernel>::Ker::orient2d(line.start, *p, line.end) == Orientation::Collinear {
                let ls = line.start.into();
                let le = line.end.into();
                if p >= ls && p <= le {
                    Some(*self)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            line_intersection(self.line(), line).map(|l| match l {
                LineIntersection::SinglePoint { intersection, .. } => intersection.into(),
                LineIntersection::Collinear { intersection } => intersection.into(),
            })
        }
    }

    pub fn intersect_line_ordered(&self, other: &Self) -> Option<Self> {
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
                Some((*pt).into())
            },
            e => e,
        }
    }
}
