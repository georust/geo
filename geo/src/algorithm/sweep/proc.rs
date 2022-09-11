use std::{
    cmp::Ordering,
    collections::{BTreeSet, BinaryHeap},
};

use super::*;

pub(crate) struct Sweep<C: Cross> {
    is_simple: bool,
    events: BinaryHeap<Event<C::Scalar, IMSegment<C>>>,
    active_segments: BTreeSet<Active<IMSegment<C>>>,
}

impl<C: Cross + Clone> Sweep<C> {
    pub(crate) fn new<I>(iter: I, is_simple: bool) -> Self
    where
        I: IntoIterator<Item = C>,
    {
        let iter = iter.into_iter();
        let size = {
            let (min_size, max_size) = iter.size_hint();
            max_size.unwrap_or(min_size)
        };

        let mut sweep = Sweep {
            events: BinaryHeap::with_capacity(size),
            active_segments: Default::default(),
            is_simple,
        };
        for cr in iter {
            IMSegment::create_segment(cr, None, None, |ev| sweep.events.push(ev));
        }

        sweep
    }

    /// Process the next event in heap.
    ///
    /// Calls the callback unless the event is spurious.
    #[inline]
    pub(super) fn next_event<F>(&mut self, mut cb: F) -> Option<SweepPoint<C::Scalar>>
    where
        F: for<'a> FnMut(&'a IMSegment<C>, EventType),
    {
        self.events.pop().map(|event| {
            let pt = event.point;
            self.handle_event(event, &mut cb);

            pt
        })
    }

    fn handle_event<F>(&mut self, event: Event<C::Scalar, IMSegment<C>>, cb: &mut F) -> bool
    where
        F: for<'a> FnMut(&'a IMSegment<C>, EventType),
    {
        use EventType::*;
        let segment = match IMSegment::is_correct(&event) {
            false => return false,
            _ => event.payload,
        };
        trace!(
            "handling event: {pt:?} ({ty:?}) @ {seg:?}",
            pt = event.point,
            ty = event.ty,
            seg = segment,
        );

        let prev = self.active_segments.previous(&segment).cloned();
        let next = self.active_segments.next(&segment).cloned();

        match &event.ty {
            LineLeft => {
                let mut should_add = true;
                if !self.is_simple {
                    for adj_segment in prev.into_iter().chain(next.into_iter()) {
                        if let Some(adj_intersection) =
                            segment.geom().intersect_line_ordered(&adj_segment.geom())
                        {
                            trace!("Found intersection (LL):\n\tsegment1: {:?}\n\tsegment2: {:?}\n\tintersection: {:?}", segment, adj_segment, adj_intersection);
                            // 1. Split adj_segment, and extra splits to storage
                            let adj_overlap = adj_segment
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e));

                            // A special case is if adj_segment was split, and the
                            // intersection is at the start of this segment. In this
                            // case, there is an right-end event in the heap, that
                            // needs to be handled before finishing up this event.
                            let handle_end_event = {
                                // Get first point of intersection
                                let int_pt = adj_intersection.left();
                                // Check its not first point of the adjusted, but is
                                // first point of current segment
                                int_pt != adj_segment.geom().left()
                                    && int_pt == segment.geom().left()
                            };
                            if handle_end_event {
                                let event = self.events.pop().unwrap();
                                let done = self.handle_event(event, cb);
                                debug_assert!(done, "special right-end event handling failed")
                            }

                            // 2. Split segment, adding extra segments as needed.
                            let seg_overlap_key = segment
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e));

                            assert_eq!(
                                adj_overlap.is_some(),
                                seg_overlap_key.is_some(),
                                "one of the intersecting segments had an overlap, but not the other!"
                            );
                            if let Some(adj_ovl) = adj_overlap {
                                let tgt = seg_overlap_key.unwrap();
                                trace!("setting overlap: {adj_ovl:?} -> {tgt:?}");
                                adj_ovl.chain_overlap(tgt.clone());

                                if tgt == segment {
                                    // The whole event segment is now overlapping
                                    // some other active segment.
                                    //
                                    // We do not need to continue iteration, but
                                    // should callback if the left event of the
                                    // now-parent has already been processed.
                                    if adj_ovl.is_left_event_done() {
                                        should_add = false;
                                        break;
                                    }
                                    return true;
                                }
                            }
                        }
                    }
                }

                if should_add {
                    // Add current segment as active
                    // Safety: `self.segments` is a `Box` that is not
                    // de-allocated until `self` is dropped.
                    debug!("insert_active: {segment:?}");
                    self.active_segments.insert_active(segment.clone());
                }

                let mut cb_seg = Some(segment);
                while let Some(seg) = cb_seg {
                    cb(&seg, event.ty);
                    seg.set_left_event_done();
                    cb_seg = seg.overlap();
                }
            }
            LineRight => {
                // Safety: `self.segments` is a `Box` that is not
                // de-allocated until `self` is dropped.
                debug!("remove_active: {segment:?}");
                self.active_segments.remove_active(&segment);

                let mut cb_seg = Some(segment);
                while let Some(seg) = cb_seg {
                    cb(&seg, event.ty);
                    cb_seg = seg.overlap();
                }

                if !self.is_simple {
                    if let (Some(prev), Some(next)) = (prev, next) {
                        let prev_geom = prev.geom();
                        let next_geom = next.geom();
                        if let Some(adj_intersection) = prev_geom.intersect_line_ordered(&next_geom)
                        {
                            // 1. Split prev_segment, and extra splits to storage
                            let first = prev
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e))
                                .is_none();
                            let second = next
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e))
                                .is_none();
                            debug_assert!(
                                first && second,
                                "adjacent segments @ removal can't overlap!"
                            );
                        }
                    }
                }
            }
            PointLeft => {
                if !self.is_simple {
                    for adj_segment in prev.into_iter().chain(next.into_iter()) {
                        let geom = adj_segment.geom();
                        if let Some(adj_intersection) = segment.geom().intersect_line_ordered(&geom)
                        {
                            trace!("Found intersection:\n\tsegment1: {:?}\n\tsegment2: {:?}\n\tintersection: {:?}", segment, adj_segment, adj_intersection);
                            // 1. Split adj_segment, and extra splits to storage
                            let adj_overlap = adj_segment
                                .adjust_one_segment(adj_intersection, |e| self.events.push(e));

                            // Can't have overlap with a point
                            debug_assert!(adj_overlap.is_none());
                        }
                    }
                }

                // Points need not be active segments.
                // Send the point-segment to callback.
                cb(&segment, event.ty);
            }
            PointRight => {
                // Nothing to do. We could remove this variant once we
                // are confident about the logic.
            }
        }
        true
    }

    #[inline]
    pub(super) fn with_prev_active<F: FnOnce(&Segment<C>) -> R, R>(
        &self,
        c: &Crossing<C>,
        f: F,
    ) -> Option<R> {
        debug_assert!(c.at_left);
        {
            debug!("with_prev_active: {c:?}");
            debug!("previous:");
            self.active_segments.previous_find(&c.segment, |aseg| {
                debug!("\t{geom:?}", geom = aseg.0.geom());
                false
            });
            debug!("next:");
            self.active_segments.next_find(&c.segment, |aseg| {
                debug!("\t{geom:?}", geom = aseg.0.geom());
                false
            });

        }
        self.active_segments
            .previous_find(&c.segment, |aseg| {
                let is_ovl = aseg.0.geom().partial_cmp(&c.line) == Some(Ordering::Equal);
                if is_ovl {
                    debug!(
                        "prev_active: found overlap: {l1:?} - {l2:?}",
                        l1 = aseg.0.geom(),
                        l2 = c.line,
                    )
                }
                !is_ovl
            })
            .map(|aseg| aseg.with_segment(f))
    }

    #[inline]
    pub fn peek_point(&self) -> Option<SweepPoint<C::Scalar>> {
        self.events.peek().map(|e| e.point)
    }
}
