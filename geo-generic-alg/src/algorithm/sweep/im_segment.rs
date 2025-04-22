use std::{borrow::Cow, cell::RefCell, cmp::Ordering, fmt::Debug, rc::Rc};

use super::*;

/// A wrapped segment that allows interior mutability.
pub(super) struct IMSegment<C: Cross> {
    inner: Rc<RefCell<Segment<C>>>,
}

impl<C: Cross> Clone for IMSegment<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<C: Cross> From<Segment<C>> for IMSegment<C> {
    fn from(segment: Segment<C>) -> Self {
        Self {
            inner: Rc::new(segment.into()),
        }
    }
}

impl<C: Cross> Debug for IMSegment<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        RefCell::borrow(&self.inner).fmt(f)
    }
}

impl<C: Cross> IMSegment<C> {
    pub fn is_overlapping(&self) -> bool {
        RefCell::borrow(&self.inner).overlapping.is_some()
    }
    pub fn overlap(&self) -> Option<Self> {
        RefCell::borrow(&self.inner).overlapping.as_ref().cloned()
    }
    pub fn is_first_segment(&self) -> bool {
        RefCell::borrow(&self.inner).first_segment
    }

    pub fn set_left_event_done(&self) {
        RefCell::borrow_mut(&self.inner).left_event_done = true;
    }
    pub fn is_left_event_done(&self) -> bool {
        RefCell::borrow(&self.inner).left_event_done
    }

    pub fn geom(&self) -> LineOrPoint<<C as Cross>::Scalar> {
        RefCell::borrow(&self.inner).geom
    }

    pub fn left_event(&self) -> Event<C::Scalar, Self> {
        let geom = self.geom();
        let left = geom.left();
        Event {
            point: left,
            ty: if geom.is_line() {
                EventType::LineLeft
            } else {
                EventType::PointLeft
            },
            payload: self.clone(),
        }
    }

    pub fn right_event(&self) -> Event<C::Scalar, Self> {
        let geom = self.geom();
        let right = geom.right();
        Event {
            point: right,
            ty: if geom.is_line() {
                EventType::LineRight
            } else {
                EventType::PointRight
            },
            payload: self.clone(),
        }
    }

    pub fn chain_overlap(&self, child: Self) {
        let mut this = self.clone();
        while let Some(ovl) = this.overlap() {
            this = ovl;
        }
        {
            RefCell::borrow_mut(&child.inner).is_overlapping = true;
        }
        {
            let mut this_mut = RefCell::borrow_mut(&this.inner);
            this_mut.overlapping = Some(child);
        }
    }

    pub fn adjust_for_intersection(
        &self,
        adj_intersection: LineOrPoint<C::Scalar>,
    ) -> SplitSegments<C::Scalar> {
        let (adjust_output, new_geom) = {
            let mut segment = RefCell::borrow_mut(&self.inner);
            trace!(
                "adjust_for_intersection: {:?}\n\twith: {:?}",
                segment,
                adj_intersection
            );
            (
                segment.adjust_for_intersection(adj_intersection),
                segment.geom,
            )
        };
        trace!("adjust_output: {:?}", adjust_output);

        let mut this = self.clone();
        while let Some(ovl) = this.overlap() {
            this = ovl;
            {
                let mut this_mut = RefCell::borrow_mut(&this.inner);
                this_mut.geom = new_geom;
            }
        }
        adjust_output
    }
}

impl<C: Cross + Clone> IMSegment<C> {
    pub(super) fn create_segment<F: FnMut(Event<C::Scalar, Self>)>(
        crossable: C,
        geom: Option<LineOrPoint<C::Scalar>>,
        parent: Option<&Self>,
        mut cb: F,
    ) -> Self {
        let segment: Self = Segment::new(crossable, geom).into();

        // Push events to process the created segment.
        for e in [segment.left_event(), segment.right_event()] {
            cb(e)
        }

        if let Some(parent) = parent {
            let segment_geom = RefCell::borrow(&segment.inner).geom;

            let mut child = RefCell::borrow(&parent.inner).overlapping.as_ref().cloned();
            let mut tgt = Cow::Borrowed(&segment);

            while let Some(child_seg) = child {
                let child_inner_seg = RefCell::borrow(&child_seg.inner);

                let child_overlapping = &child_inner_seg.overlapping;
                let child_crossable = child_inner_seg.cross.clone();

                let new_segment: Self = Segment::new(child_crossable, Some(segment_geom)).into();

                {
                    RefCell::borrow_mut(&tgt.inner).overlapping = Some(new_segment.clone());
                }
                {
                    RefCell::borrow_mut(&new_segment.inner).is_overlapping = true;
                }

                tgt = Cow::Owned(new_segment);
                child = child_overlapping.as_ref().cloned();
            }
        }
        segment
    }

    pub fn adjust_one_segment<F: FnMut(Event<C::Scalar, Self>)>(
        &self,
        adj_intersection: LineOrPoint<C::Scalar>,
        mut cb: F,
    ) -> Option<Self> {
        let adj_cross = self.cross_cloned();
        use SplitSegments::*;
        match self.adjust_for_intersection(adj_intersection) {
            Unchanged { overlap } => overlap.then(|| self.clone()),
            SplitOnce { overlap, right } => {
                cb(self.right_event());
                let new_key = Self::create_segment(adj_cross, Some(right), Some(self), &mut cb);
                match overlap {
                    Some(false) => Some(self.clone()),
                    Some(true) => Some(new_key),
                    None => None,
                }
            }
            SplitTwice { right } => {
                cb(self.right_event());
                Self::create_segment(adj_cross.clone(), Some(right), Some(self), &mut cb);
                let middle =
                    Self::create_segment(adj_cross, Some(adj_intersection), Some(self), &mut cb);
                Some(middle)
            }
        }
    }

    pub fn is_correct(event: &Event<C::Scalar, IMSegment<C>>) -> bool {
        use EventType::*;
        let segment = RefCell::borrow(&event.payload.inner);
        if let LineRight = event.ty {
            debug_assert!(segment.geom.is_line());
            !segment.is_overlapping && segment.geom.right() == event.point
        } else {
            match event.ty {
                LineLeft => {
                    debug_assert!(segment.geom.is_line());
                    debug_assert_eq!(segment.geom.left(), event.point);
                }
                PointLeft | PointRight => {
                    debug_assert!(!segment.geom.is_line());
                    debug_assert_eq!(segment.geom.left(), event.point);
                }
                _ => unreachable!(),
            }
            true
        }
    }

    pub fn cross_cloned(&self) -> C {
        let inner = RefCell::borrow(&self.inner);
        inner.cross.clone()
    }
}

impl<C: Cross> PartialEq for IMSegment<C> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<C: Cross> PartialOrd for IMSegment<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        RefCell::borrow(&self.inner)
            .partial_cmp(&RefCell::borrow(&other.inner))
            .map(|o| {
                o.then_with(|| {
                    let addr_self = Rc::as_ptr(&self.inner) as usize;
                    let addr_other = Rc::as_ptr(&other.inner) as usize;
                    addr_self.cmp(&addr_other)
                    // .reverse()
                })
            })
    }
}
