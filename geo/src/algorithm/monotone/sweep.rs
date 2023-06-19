use crate::sweep::{Active, Event, EventType, LineOrPoint, SweepPoint, VecSet};
use crate::GeoNum;
use std::{collections::BinaryHeap, fmt::Debug};

use super::{RcSegment, Segment};

/// Simple planar sweep algorithm.
///
/// Performs a planar sweep along the x-axis on a set of line or points.  This
/// can be initialized with a iterator of line segments or points, along with an
/// optional payload.
///
/// The sweep is used to:
///
/// - iterate over all end-points of the input line or points in lex. order
///
/// - query the set of active segments at the current iteration point: these are
/// the segments currently intersecting the sweep line, and are ordered by their
/// position on the line
///
/// # Note
///
/// This is a simpler version, which does not support intersections except at
/// end-points.  Specifically, the algorithm expects any pair of segments to
/// maintain a fixed ordering along all common sweep lines, ties broken
/// arbitrarily.
pub(crate) struct SimpleSweep<T: GeoNum, P: Debug> {
    events: BinaryHeap<Event<T, RcSegment<T, P>>>,
    active_segments: VecSet<Active<RcSegment<T, P>>>,
}

impl<T: GeoNum, P: Debug> SimpleSweep<T, P> {
    pub(crate) fn new<I, D>(iter: I) -> Self
    where
        I: IntoIterator<Item = D>,
        D: Into<Segment<T, P>>,
    {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };
        let mut events = BinaryHeap::with_capacity(size);
        let active_segments = VecSet::default();

        for cr in iter {
            let segment = RcSegment::from(cr.into());
            events.extend(segment.events());
        }

        SimpleSweep {
            events,
            active_segments,
        }
    }

    /// Progress and obtain the next sweep point along with the set of segments
    /// ending at the current sweep point.
    ///
    /// The segments are returned as per the ordering of their `EventType`; in
    /// particular, all segments ending at the current sweep point are returned
    /// before the ones starting at the current sweep point.  The idx of the
    /// first segment starting at the current sweep point is returned in the
    /// `split_idx` parameter.
    pub(crate) fn next_point<F: FnMut(RcSegment<T, P>, EventType)>(
        &mut self,
        mut f: F,
    ) -> Option<SweepPoint<T>> {
        let point = self.peek_point();
        while let Some(pt) = point {
            self.next_event().map(|ev| {
                let segment = ev.payload;
                let ty = ev.ty;
                f(segment, ty);
            });
            if self.peek_point() != Some(pt) {
                break;
            }
        }
        point
    }

    /// Process the next event in heap.
    #[inline]
    pub(super) fn next_event(&mut self) -> Option<Event<T, RcSegment<T, P>>> {
        self.events.pop().map(|event| {
            self.handle_event(&event);
            event
        })
    }

    fn handle_event(&mut self, event: &Event<T, RcSegment<T, P>>) {
        use EventType::*;
        let segment = &event.payload;
        trace!(
            "handling event: {pt:?} ({ty:?}) @ {seg:?}",
            pt = event.point,
            ty = event.ty,
            seg = segment,
        );

        match &event.ty {
            LineLeft => {
                let idx = self.active_segments.index_not_of(&segment);
                self.active_segments.insert_at(idx, segment.clone());
            }
            LineRight => {
                let idx = self.active_segments.index_of(&segment);
                self.active_segments.remove_at(idx);
            }
            _ => {}
        }
    }

    #[inline]
    pub(super) fn peek_point(&self) -> Option<SweepPoint<T>> {
        self.events.peek().map(|e| e.point)
    }

    pub(super) fn prev_active_from_geom(&self, geom: LineOrPoint<T>) -> Option<RcSegment<T, P>> {
        let part_idx = self.active_segments.partition_point(|s| s.line() < geom);
        if part_idx == 0 {
            None
        } else {
            Some(self.active_segments[part_idx - 1].0.clone())
        }
    }
}
