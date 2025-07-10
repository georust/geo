use super::*;
use crate::GeoFloat;
use std::{cmp::Ordering, fmt::Debug};

/// A segment of input [`LineOrPoint`] generated during the sweep.
#[derive(Clone)]
pub(super) struct Segment<C: Cross> {
    pub(super) geom: LineOrPoint<C::Scalar>,
    pub(super) cross: C,
    pub(super) first_segment: bool,
    pub(super) left_event_done: bool,
    pub(super) overlapping: Option<IMSegment<C>>,
    pub(super) is_overlapping: bool,
}

impl<C: Cross> Segment<C> {
    pub fn new(cross: C, geom: Option<LineOrPoint<C::Scalar>>) -> Self {
        let first_segment = geom.is_none();
        let geom = geom.unwrap_or_else(|| cross.line());
        Self {
            geom,
            cross,
            first_segment,
            left_event_done: false,
            overlapping: None,
            is_overlapping: false,
        }
    }

    /// Split a line segment into pieces at points of intersection.
    ///
    /// The initial segment is mutated in place, and extra-segment(s) are
    /// returned if any. Assume exact arithmetic, the ordering of self should
    /// remain the same among active segments. However, with finite-precision,
    /// this may not be the case.
    pub fn adjust_for_intersection(
        &mut self,
        intersection: LineOrPoint<C::Scalar>,
    ) -> SplitSegments<C::Scalar> {
        use SplitSegments::*;

        // We only support splitting a line segment.
        debug_assert!(self.geom.is_line());
        let (p, q) = self.geom.end_points();

        if !intersection.is_line() {
            // Handle point intersection
            let r = intersection.left();
            debug_assert!(
                p <= r,
                "intersection point was not ordered within the line: {p:?} <= {r:?} <=> {q:?}",
            );
            if p == r || q == r {
                // If the intersection is at the end point, the
                // segment doesn't need to be split.
                Unchanged { overlap: false }
            } else {
                // Otherwise, split it. Mutate `self` to be the
                // first part, and return the second part.
                self.geom = (p, r).into();
                // self.first_segment = false;
                SplitOnce {
                    overlap: None,
                    right: (r, q).into(),
                }
            }
        } else {
            let (r1, r2) = intersection.end_points();
            debug_assert!(
                p <= r1 && r2 <= q,
                "overlapping segment was not ordered within the line!"
            );
            if p == r1 {
                if r2 == q {
                    // The whole segment overlaps.
                    Unchanged { overlap: true }
                } else {
                    self.geom = (p, r2).into();
                    // self.first_segment = false;
                    SplitOnce {
                        overlap: Some(false),
                        right: (r2, q).into(),
                    }
                }
            } else if r2 == q {
                self.geom = (p, r1).into();
                // self.first_segment = false;
                SplitOnce {
                    overlap: Some(true),
                    right: (r1, q).into(),
                }
            } else {
                self.geom = (p, r1).into();
                // self.first_segment = false;
                SplitTwice {
                    right: (r2, q).into(),
                }
            }
        }
    }
}

/// A more concise debug impl.
impl<C: Cross> Debug for Segment<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Segment{{ {geom:?}\n\tof {c:?}\n\t{first} [{has}/{ovl}] }}",
            c = self.cross,
            geom = self.geom,
            first = if self.first_segment { "[1st]" } else { "" },
            has = if self.overlapping.is_some() {
                "HAS"
            } else {
                "NON"
            },
            ovl = if self.is_overlapping { "OVL" } else { "NON" },
        )
    }
}

/// Partial equality based on key.
///
/// This is consistent with the `PartialOrd` impl.
impl<C: Cross> PartialEq for Segment<C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

/// Partial ordering defined as per algorithm.
///
/// This is requires the same pre-conditions as for [`LineOrPoint`].
impl<C: Cross> PartialOrd for Segment<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.geom.partial_cmp(&other.geom)
    }
}

/// Stores the type of split and extra geometries from adjusting a
/// segment for intersection.
#[derive(Debug)]
pub(super) enum SplitSegments<T: GeoFloat> {
    Unchanged {
        overlap: bool,
    },
    SplitOnce {
        overlap: Option<bool>,
        right: LineOrPoint<T>,
    },
    SplitTwice {
        right: LineOrPoint<T>,
    },
}

#[cfg(test)]
mod tests {

    use super::*;

    impl<T: GeoFloat> PartialEq for SplitSegments<T> {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (
                    Self::Unchanged { overlap: l_overlap },
                    Self::Unchanged { overlap: r_overlap },
                ) => l_overlap == r_overlap,
                (
                    Self::SplitOnce {
                        overlap: l_overlap,
                        right: l_right,
                    },
                    Self::SplitOnce {
                        overlap: r_overlap,
                        right: r_right,
                    },
                ) => l_overlap == r_overlap && l_right.coords_equal(r_right),
                (Self::SplitTwice { right: l_right }, Self::SplitTwice { right: r_right }) => {
                    l_right.coords_equal(r_right)
                }
                _ => false,
            }
        }
    }

    #[test]
    fn test_split() {
        let lines: Vec<_> = vec![
            LineOrPoint::from(((0., 0.).into(), (10., 10.).into())),
            ((10.0, 0.).into(), (0., 10.).into()).into(),
            ((0., 0.).into(), (0., 10.).into()).into(),
            ((0., 0.).into(), (5., 5.).into()).into(),
            ((10., 10.).into(), (5., 5.).into()).into(),
        ]
        .into_iter()
        .map(|lp| Segment::new(lp, None))
        .collect();

        struct TestCase {
            a: usize,
            b: usize,
            isec: Option<LineOrPoint<f64>>,
            split: Option<SplitSegments<f64>>,
        }

        impl TestCase {
            fn assert_equality(&self, lines: &[Segment<LineOrPoint<f64>>]) {
                let isec = lines[self.a]
                    .geom
                    .intersect_line_ordered(&lines[self.b].geom);
                assert_eq!(isec, self.isec);

                if isec.is_none() {
                    return;
                }
                let isec = isec.unwrap();
                let mut copy_seg = lines[self.a].clone();
                let split = copy_seg.adjust_for_intersection(isec);
                assert_eq!(&split, self.split.as_ref().unwrap(),)
            }
        }

        let test_cases = vec![
            TestCase {
                a: 0,
                b: 0,
                isec: Some(lines[0].geom),
                split: Some(SplitSegments::Unchanged { overlap: true }),
            },
            TestCase {
                a: 0,
                b: 1,
                isec: Some(LineOrPoint::from(SweepPoint::from((5., 5.)))),
                split: Some(SplitSegments::SplitOnce {
                    overlap: None,
                    right: LineOrPoint::from(((5., 5.).into(), (10., 10.).into())),
                }),
            },
            TestCase {
                a: 0,
                b: 2,
                isec: Some(LineOrPoint::from(SweepPoint::from((0., 0.)))),
                split: Some(SplitSegments::Unchanged { overlap: false }),
            },
            TestCase {
                a: 0,
                b: 3,
                isec: Some(LineOrPoint::from(((0., 0.).into(), (5., 5.).into()))),
                split: Some(SplitSegments::SplitOnce {
                    overlap: Some(false),
                    right: LineOrPoint::from(((5., 5.).into(), (10., 10.).into())),
                }),
            },
            TestCase {
                a: 0,
                b: 4,
                isec: Some(LineOrPoint::from(((5., 5.).into(), (10., 10.).into()))),
                split: Some(SplitSegments::SplitOnce {
                    overlap: Some(true),
                    right: LineOrPoint::from(((5., 5.).into(), (10., 10.).into())),
                }),
            },
        ];

        test_cases.iter().for_each(|t| t.assert_equality(&lines));
    }
}
