use std::{cmp::Ordering, ops::Deref};

use super::SweepPoint;
use crate::{
    line_intersection::line_intersection, Coord, GeoFloat, GeoNum, Kernel, Line, LineIntersection,
    Orientation,
};

/// Either a line segment or a point.
///
/// The coordinates are ordered (see [`SweepPoint`]) and a line
/// segment must have distinct points (use the `Point` variant if the
/// coordinates are the equal).
#[derive(Clone, Copy)]
pub enum LineOrPoint<T: GeoNum> {
    Point(SweepPoint<T>),
    Line {
        left: SweepPoint<T>,
        right: SweepPoint<T>,
    },
}

impl<T: GeoNum> std::fmt::Debug for LineOrPoint<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineOrPoint::Point(p) => f.debug_tuple("Pt").field(&p.x_y()).finish(),
            LineOrPoint::Line { left, right } => f
                .debug_tuple("LPt")
                .field(&left.x_y())
                .field(&right.x_y())
                .finish(),
        }
    }
}

impl<T: GeoNum> From<SweepPoint<T>> for LineOrPoint<T> {
    fn from(pt: SweepPoint<T>) -> Self {
        Self::Point(pt)
    }
}

impl<T: GeoNum> From<(SweepPoint<T>, SweepPoint<T>)> for LineOrPoint<T> {
    fn from((start, end): (SweepPoint<T>, SweepPoint<T>)) -> Self {
        match start.cmp(&end) {
            Ordering::Less => Self::Line {
                left: start,
                right: end,
            },
            Ordering::Equal => Self::Point(start),
            Ordering::Greater => Self::Line {
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

/// Convert from a [`Coord`]
impl<T: GeoNum> From<Coord<T>> for LineOrPoint<T> {
    fn from(c: Coord<T>) -> Self {
        Self::Point(c.into())
    }
}

impl<T: GeoNum> LineOrPoint<T> {
    /// Checks if the variant is a line.
    #[inline]
    pub fn is_line(&self) -> bool {
        matches!(self, Self::Line { .. })
    }

    /// Return a [`Line`] representation of self.
    #[inline]
    pub fn line(&self) -> Line<T> {
        match self {
            LineOrPoint::Point(p) => Line::new(**p, **p),
            LineOrPoint::Line { left, right } => Line::new(**left, **right),
        }
    }

    #[inline]
    pub fn left(&self) -> SweepPoint<T> {
        match self {
            LineOrPoint::Point(p) => *p,
            LineOrPoint::Line { left, .. } => *left,
        }
    }

    #[inline]
    pub fn right(&self) -> SweepPoint<T> {
        match self {
            LineOrPoint::Point(p) => *p,
            LineOrPoint::Line { right, .. } => *right,
        }
    }

    #[cfg(test)]
    pub fn coords_equal(&self, other: &LineOrPoint<T>) -> bool {
        self.is_line() == other.is_line() && self.end_points() == other.end_points()
    }

    #[inline]
    pub fn end_points(&self) -> (SweepPoint<T>, SweepPoint<T>) {
        match self {
            LineOrPoint::Point(p) => (*p, *p),
            LineOrPoint::Line { left, right } => (*left, *right),
        }
    }

    pub fn new(left: SweepPoint<T>, right: SweepPoint<T>) -> Self {
        if left == right {
            Self::Point(left)
        } else {
            Self::Line { left, right }
        }
    }

    pub fn orient2d(&self, other: Coord<T>) -> Orientation {
        let (left, right) = match self {
            LineOrPoint::Point(p) => (**p, **p),
            LineOrPoint::Line { left, right } => (**left, **right),
        };
        T::Ker::orient2d(left, right, other)
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
///    smaller than both right ends.
///
/// 2. A point is treated as a infinitesimal small vertical segment
///    centered at its coordinates.
impl<T: GeoNum> PartialOrd for LineOrPoint<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (LineOrPoint::Point(p), LineOrPoint::Point(o)) => {
                if p == o {
                    Some(Ordering::Equal)
                } else {
                    // Unequal points do not satisfy pre-condition and
                    // can't be ordered.
                    None
                }
            }
            (LineOrPoint::Point(_), LineOrPoint::Line { .. }) => {
                other.partial_cmp(self).map(Ordering::reverse)
            }
            (LineOrPoint::Line { left, right }, LineOrPoint::Point(p)) => {
                if p > right || left > p {
                    return None;
                }
                Some(
                    T::Ker::orient2d(**left, **right, **p)
                        .as_ordering()
                        .then(Ordering::Greater),
                )
            }
            (
                LineOrPoint::Line {
                    left: left_a,
                    right: right_a,
                },
                LineOrPoint::Line {
                    left: left_b,
                    right: right_b,
                },
            ) => {
                if left_a > left_b {
                    return other.partial_cmp(self).map(Ordering::reverse);
                }
                if left_a >= right_b || left_b >= right_a {
                    return None;
                }

                // Assertion: p1 <= p2
                // Assertion: pi < q_j
                Some(
                    T::Ker::orient2d(**left_a, **right_a, **left_b)
                        .as_ordering()
                        .then_with(|| {
                            T::Ker::orient2d(**left_a, **right_a, **right_b).as_ordering()
                        }),
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
        match self {
            LineOrPoint::Point(p) => {
                use crate::Intersects;
                if line.intersects(&**p) {
                    Some(*self)
                } else {
                    None
                }
            }
            LineOrPoint::Line { left, right } => {
                line_intersection(self.line(), line).map(|l| match l {
                    LineIntersection::SinglePoint {
                        intersection,
                        is_proper,
                    } => {
                        let mut pt = intersection;
                        if is_proper && (&pt == left.deref()) {
                            if left.x == right.x {
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
    }

    pub fn intersect_line_ordered(&self, other: &Self) -> Option<Self> {
        let ord = self.partial_cmp(other);
        match self.intersect_line(other) {
            Some(Self::Point(p)) => {
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
                let (mut x, y) = p.x_y();

                let c = self.left();
                if x == c.x && y < c.y {
                    x = x.next_after(T::infinity());
                }

                let p = Coord { x, y }.into();
                debug_assert!(
                    p >= self.left(),
                    "line intersection before first line: {p:?}\n\tLine({lp1:?} - {lp2:?}) X Line({lp3:?} - {lp4:?})",
                    lp1 = self.left(),
                    lp2 = self.right(),
                    lp3 = other.left(),
                    lp4 = other.right(),
                );
                debug_assert!(
                    p >= other.left(),
                    "line intersection before second line: {p:?}\n\tLine({lp1:?} - {lp2:?}) X Line({lp3:?} - {lp4:?})",
                    lp1 = self.left(),
                    lp2 = self.right(),
                    lp3 = other.left(),
                    lp4 = other.right(),
                );

                if let Some(ord) = ord {
                    let l1 = LineOrPoint::from((self.left(), p));
                    let l2 = LineOrPoint::from((other.left(), p));
                    let cmp = l1.partial_cmp(&l2).unwrap();
                    if l1.is_line() && l2.is_line() && cmp.then(ord) != ord {
                        debug!(
                            "ordering changed by intersection: {l1:?} {ord:?} {l2:?}",
                            l1 = self,
                            l2 = other
                        );
                        debug!("\tparts: {l1:?}, {l2:?}");
                        debug!("\tintersection: {p:?} {cmp:?}");

                        // RM: This is a complicated intersection that is changing the ordering.
                        // Heuristic: approximate with a trivial intersection point that preserves the topology.
                        return Some(if self.left() > other.left() {
                            self.left().into()
                        } else {
                            other.left().into()
                        });
                    }
                }
                Some(Self::Point(p))
            }
            e => e,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use geo_types::{Coord, LineString};
    use wkt::ToWkt;

    use crate::{GeoFloat, GeoNum, Kernel};

    use super::LineOrPoint;

    // Used for debugging sweep fp issues
    #[test]
    #[ignore]
    fn check_ordering() {
        let pt_7 = Coord::from((-32.57812499999999, 241.33427773853316));
        let pt_8 = Coord::from((-36.11348070978957, 237.7989220287436));
        let pt_13 = Coord::from((-25.507080078124993, 248.40532266040816));
        let pt_14 = Coord::from((-36.48784219165816, 237.424560546875));
        let _pt_15 = Coord::from((4.4929199218750036, 196.44379843334184));
        let pt_16 = Coord::from((-36.048578439260666, 237.8638242992725));
        let pt_17 = Coord::from((3.545624214480127, 197.39109414073673));

        fn check_isection<T: GeoFloat>(abcd: [Coord<T>; 4]) -> Option<LineOrPoint<T>> {
            let l1 = LineOrPoint::from((abcd[0].into(), abcd[1].into()));
            let l2 = LineOrPoint::from((abcd[2].into(), abcd[3].into()));
            l1.intersect_line_ordered(&l2)
        }
        fn check_lines<T: GeoNum>(abcd: [Coord<T>; 4]) -> Ordering {
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
            <f64 as GeoNum>::Ker::orient2d(pt_8, pt_7, pt_17)
        );
        eprintln!(
            "pred: {:?}",
            <f64 as GeoNum>::Ker::orient2d(pt_8, pt_14, pt_16)
        );
    }
}
