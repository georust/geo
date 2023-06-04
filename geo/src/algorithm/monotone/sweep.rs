use crate::sweep::{Active, Event, EventType, SweepPoint};
use crate::GeoNum;
use std::ops::Bound;
use std::{
    collections::{BTreeSet, BinaryHeap},
    fmt::Debug,
    rc::Rc,
};

use super::Segment;

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
    events: BinaryHeap<Event<T, Rc<Segment<T, P>>>>,
    active_segments: BTreeSet<Active<Rc<Segment<T, P>>>>,
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
        let active_segments = BTreeSet::new();

        for cr in iter {
            let segment = Rc::new(cr.into());
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
    pub(crate) fn next_point<F: FnMut(Rc<Segment<T, P>>, EventType)>(
        &mut self,
        mut f: F,
    ) -> Option<SweepPoint<T>> {
        let point = self.peek_point();
        debug!("pt: {point:?}");

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
    pub(super) fn next_event(&mut self) -> Option<Event<T, Rc<Segment<T, P>>>> {
        self.events.pop().map(|event| {
            let _segment = self.handle_event(&event);
            event
        })
    }

    fn handle_event(&mut self, event: &Event<T, Rc<Segment<T, P>>>) {
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
                self.active_segments.insert(Active(segment.clone()));
            }
            LineRight => {
                self.active_segments.remove(Active::active_ref(segment));
            }
            _ => {}
        }
    }

    #[inline]
    pub(super) fn peek_point(&self) -> Option<SweepPoint<T>> {
        self.events.peek().map(|e| e.point)
    }

    /// Returns the active segment, strictly before the current segment.
    pub(super) fn prev_active_segments(
        &self,
        segment: &Rc<Segment<T, P>>,
    ) -> impl Iterator<Item = Rc<Segment<T, P>>> + '_ {
        self.active_segments
            .range::<Active<_>, _>((
                Bound::Unbounded,
                Bound::Excluded(Active::active_ref(segment)),
            ))
            .rev()
            .map(|a| a.0.clone())
    }

    pub(crate) fn prev_active(&self, segment: &Rc<Segment<T, P>>) -> Option<Rc<Segment<T, P>>> {
        self.prev_active_segments(segment).next()
    }
}
