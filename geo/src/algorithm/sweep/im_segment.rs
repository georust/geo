use std::{cell::UnsafeCell, cmp::Ordering, fmt::Debug, rc::Rc, borrow::Borrow, ops::Deref};

use super::*;

/// A wrapped segment that allows interior mutability.
pub(super) struct IMSegment<C: Cross> {
    inner: Rc<UnsafeCell<Segment<C>>>,
}

impl<C: Cross> Borrow<Segment<C>> for IMSegment<C> {
    fn borrow(&self) -> &Segment<C> {
        unsafe { self.get() }
    }
}

// Unfortunately, a generic impl<U, T: Borrow<U>> Borrow<U> for Active<T>
// doesn't work for some complicated reasons. AsRef is another trait with this
// issue.
impl<C: Cross> Borrow<Active<Segment<C>>> for Active<IMSegment<C>> {
    fn borrow(&self) -> &Active<Segment<C>> {
        Active::active_ref( unsafe { self.deref().get() } )
    }
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
        unsafe { self.get() }.fmt(f)
    }
}

impl<C: Cross> IMSegment<C> {
    unsafe fn get(&self) -> &Segment<C> {
        &*self.inner.get()
    }

    unsafe fn get_mut(&self) -> &mut Segment<C> {
        &mut *self.inner.get()
    }

    pub fn overlapping(&self) -> Option<&Self> {
        unsafe { self.get() }.overlapping.as_ref()
    }

    pub fn cross(&self) -> &C {
        &unsafe { self.get() }.cross
    }

    pub fn set_left_event_done(&self) {
        unsafe { self.get_mut() }.left_event_done = true;
    }

    pub fn geom(&self) -> LineOrPoint<<C as Cross>::Scalar> {
        unsafe { self.get() }.geom
    }

    pub fn left_event(&self) -> Event<C::Scalar, Self> {
        let inner = unsafe { self.get() };
        let geom = inner.geom;
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
        let inner = unsafe { self.get() };
        let geom = inner.geom;
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
        let mut this = self;
        loop {
            let inner = unsafe { this.get() };
            match inner.overlapping.as_ref() {
                Some(ovl) => {
                    this = ovl;
                }
                _ => break,
            }
        }

        let this = unsafe { this.get_mut() };
        unsafe { child.get_mut() }.is_overlapping = true;
        this.overlapping = Some(child);
    }

    pub fn adjust_for_intersection(
        &self,
        adj_intersection: LineOrPoint<C::Scalar>,
    ) -> SplitSegments<C::Scalar> {
        let (adjust_output, new_geom) = {
            let segment = unsafe { self.get_mut() };
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

        let mut this = self;
        loop {
            let inner = unsafe { this.get() };
            match inner.overlapping.as_ref() {
                Some(ovl) => {
                    this = ovl;
                }
                _ => break,
            }
            unsafe { this.get_mut() }.geom = new_geom;
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
            let segment_geom = unsafe { segment.get() }.geom;

            let mut child = &unsafe { parent.get() }.overlapping;
            let mut tgt = segment.clone();

            while let Some(child_seg) = child {
                let child_inner_seg = unsafe { child_seg.get() };

                let child_overlapping = &child_inner_seg.overlapping;
                let child_crossable = child_inner_seg.cross.clone();

                let new_segment: Self = Segment::new(child_crossable, Some(segment_geom)).into();

                unsafe { tgt.get_mut() }.overlapping = Some(new_segment.clone());
                unsafe { new_segment.get_mut() }.is_overlapping = true;

                tgt = new_segment;
                child = child_overlapping;
            }
        }
        segment
    }

    pub fn adjust_one_segment<F: FnMut(Event<C::Scalar, Self>)>(
        &self,
        adj_intersection: LineOrPoint<C::Scalar>,
        mut cb: F,
    ) -> Option<Self> {
        let adj_segment = &mut unsafe { self.get() };
        let adj_cross = adj_segment.cross.clone();
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
                let middle = Self::create_segment(adj_cross, Some(adj_intersection), Some(self), &mut cb);
                Some(middle)
            }
        }
    }

    pub fn is_correct(event: &Event<C::Scalar, IMSegment<C>>) -> bool {
        use EventType::*;
        let segment = unsafe { event.payload.get() };
        if let LineRight = event.ty {
            debug_assert!(segment.geom.is_line());
            if !segment.is_overlapping && segment.geom.right() == event.point {
                true
            } else {
                false
            }
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
}

impl<C: Cross> PartialEq for IMSegment<C> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<C: Cross> PartialOrd for IMSegment<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        unsafe { self.get() }
            .partial_cmp(unsafe { other.get() })
            .map(|o| {
                o.then_with(|| {
                    let addr_self = Rc::as_ptr(&self.inner) as usize;
                    let addr_other = Rc::as_ptr(&other.inner) as usize;
                    addr_self.cmp(&addr_other)
                })
            })
    }
}
