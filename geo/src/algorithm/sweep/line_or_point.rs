use std::cmp::Ordering;

use crate::{ GeoNum, Line, Coordinate, GeoFloat, line_intersection::line_intersection, LineIntersection, HasKernel, Kernel, Orientation };
use super::SweepPoint;

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
            Ordering::Less => Self{ left: start, right: end },
            Ordering::Equal => Self{ left: start, right: start },
            Ordering::Greater => Self{ left: end, right: start },
        }
    }
}

/// Convert from a [`Coordinate`]
impl<T: GeoNum> From<Coordinate<T>> for LineOrPoint<T> {
    fn from(c: Coordinate<T>) -> Self {
        Self { left: c.into(), right: c.into() }
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
            if <T as HasKernel>::Ker::orient2d(line.start, *p, line.end)
                == Orientation::Collinear
            {
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
}
