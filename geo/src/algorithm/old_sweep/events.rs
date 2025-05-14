use std::cmp::Ordering;

use super::SweepPoint;
use crate::GeoNum;

/// Events generated during a sweep.
#[derive(Debug)]
pub(crate) struct Event<T: GeoNum, P> {
    pub point: SweepPoint<T>,
    pub ty: EventType,
    pub payload: P,
}

/// Equality check for usage in ordered sets. Note that it ignores
/// segment_key.
impl<T: GeoNum, P> PartialEq for Event<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.point == other.point && self.ty == other.ty
    }
}

/// Assert total equality
impl<T: GeoNum, P> Eq for Event<T, P> {}

/// Ordering for use with a max-heap (`BinaryHeap`). Note that it
/// ignores the segment_key. This suffices for heap usage, where
/// repeated items are allowed.
impl<T: GeoNum, P> PartialOrd for Event<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Derive `Ord` from `PartialOrd` and expect to not fail.
impl<T: GeoNum, P> Ord for Event<T, P> {
    fn cmp(&self, other: &Self) -> Ordering {
        // The reverse here is to confirm to max-heap / queue impl.
        self.point
            .cmp(&other.point)
            .then_with(|| self.ty.cmp(&other.ty))
            .reverse()
    }
}

/// Types of sweep events.
///
/// Sweep events are generated as the sweep reaches the start/end of
/// line-segments as they are encountered during the sweep. In addition, we also
/// support point geometries in the sweep, which is mathematically interpreted
/// as a infinitesimal vertical segment centered at the point.
///
/// The ordering of the variants is important for the algorithm. We require the
/// right end points to be ordered before the left end points to ensure the
/// active-segments of the sweep are always totally ordered. A point segment is
/// interpreted as infinitesimal vertical segment around the point, and thus its
/// left and right events are before and after the line variants respectively.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
pub(crate) enum EventType {
    PointLeft,
    LineRight,
    LineLeft,
    PointRight,
}
