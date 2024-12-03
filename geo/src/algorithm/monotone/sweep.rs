use crate::sweep::{Active, Event, EventType, LineOrPoint, SweepPoint, VecSet};
use crate::{GeoNum, Orientation};
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
///   the segments currently intersecting the sweep line, and are ordered by their
///    position on the line
///
/// # Note
///
/// This is a simpler version, which does not support intersections that are
/// interior to both segments.  That is, every intersection between two segments
/// should be at the end point of at least one of them.  In particular, overlaps
/// are also not supported (will panic).
pub(crate) struct SimpleSweep<T: GeoNum, P: Debug> {
    events: BinaryHeap<Event<T, RcSegment<T, P>>>,
    active_segments: VecSet<Active<RcSegment<T, P>>>,
}

impl<T: GeoNum, P: Debug + Clone> SimpleSweep<T, P> {
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
            let ev = self.events.pop().unwrap();
            self.handle_event(ev, &mut |ev| {
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

    fn handle_event<F>(&mut self, event: Event<T, RcSegment<T, P>>, cb: &mut F)
    where
        F: FnMut(Event<T, RcSegment<T, P>>),
    {
        // We may get spurious events from adjusting the line segment.  Ignore.
        if event.point != event.payload.line().left() && event.point != event.payload.line().right()
        {
            return;
        }

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
                let mut idx = self.active_segments.index_not_of(segment);
                for is_next in [false, true] {
                    let (active, split) = if !is_next {
                        if idx > 0 {
                            let active = &self.active_segments[idx - 1];
                            (active, self.check_interior_intersection(active, segment))
                        } else {
                            continue;
                        }
                    } else if idx < self.active_segments.len() {
                        let active = &self.active_segments[idx];
                        (active, self.check_interior_intersection(active, segment))
                    } else {
                        continue;
                    };

                    match split {
                        SplitResult::SplitA(pt) => {
                            let new_seg = active.split_at(pt);
                            let [_, ev] = active.events();
                            self.events.push(ev);
                            self.events.extend(new_seg.events());
                        }
                        SplitResult::SplitB(pt) => {
                            let new_seg = segment.split_at(pt);
                            let [_, ev] = segment.events();
                            self.events.push(ev);
                            self.events.extend(new_seg.events());
                        }
                        SplitResult::None => {}
                    }

                    // Special case:  if we split at the current event point, then
                    // we have LineRight events in the queue that have to be
                    // processed before this.

                    // There's always a top as this is a left event.
                    while self.events.peek().unwrap() > &event {
                        debug_assert_eq!(self.events.peek().unwrap().ty, LineRight);
                        debug_assert_eq!(self.events.peek().unwrap().point, event.point);

                        let ev = self.events.pop().unwrap();
                        self.handle_event(ev, cb);
                        if !is_next {
                            idx -= 1;
                        }
                    }
                }

                self.active_segments.insert_at(idx, segment.clone());
            }
            LineRight => {
                let idx = self.active_segments.index_of(segment);
                self.active_segments.remove_at(idx);

                if idx > 0 && idx < self.active_segments.len() {
                    let prev = &self.active_segments[idx - 1];
                    let next = &self.active_segments[idx];
                    self.check_interior_intersection(prev, next);
                }
            }
            _ => {}
        }
        cb(event);
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

    /// Check if the two segments intersect at a point interior to one of them.
    fn check_interior_intersection(
        &self,
        a: &RcSegment<T, P>,
        b: &RcSegment<T, P>,
    ) -> SplitResult<T> {
        let la = a.line();
        let lb = b.line();

        let lal = la.left();
        let lar = la.right();

        let lbl = lb.left();
        let lbr = lb.right();

        if lal < lbl && lbl < lar && la.orient2d(*lbl) == Orientation::Collinear {
            SplitResult::SplitA(lbl)
        } else if lal < lbr && lbr < lar && la.orient2d(*lbr) == Orientation::Collinear {
            SplitResult::SplitA(lbr)
        } else if lbl < lal && lal < lbr && lb.orient2d(*lal) == Orientation::Collinear {
            SplitResult::SplitB(lal)
        } else if lbl < lar && lar < lbr && lb.orient2d(*lar) == Orientation::Collinear {
            SplitResult::SplitB(lar)
        } else {
            SplitResult::None
        }
    }
}

enum SplitResult<T: GeoNum> {
    SplitA(SweepPoint<T>),
    SplitB(SweepPoint<T>),
    None,
}
