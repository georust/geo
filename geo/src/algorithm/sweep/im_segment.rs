use std::{cell::UnsafeCell, cmp::Ordering, fmt::Debug, rc::Rc};

use super::*;

/// A wrapped segment that allows interior mutability.
pub(super) struct IMSegment<C: Cross> {
    inner: Rc<UnsafeCell<Segment<C>>>,
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

    pub fn chain_overlap(&self, mut child: Self) {
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
            (
                segment.adjust_for_intersection(adj_intersection),
                segment.geom,
            )
        };

        use SplitSegments::*;
        if matches!(adjust_output, SplitOnce { .. } | SplitTwice { .. }) {
            todo!("move cb to call-site");
            // cb(self.right_event());
        }

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
    fn create_segment(
        crossable: C,
        geom: Option<LineOrPoint<C::Scalar>>,
        parent: Option<&Self>,
    ) -> Self {
        let segment: Self = Segment::new(crossable, geom).into();

        // Push events to process the created segment.
        for e in [segment.left_event(), segment.right_event()] {
            todo!();
            // self.events.push(e);
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

    pub fn adjust_one_segment(&self, adj_intersection: LineOrPoint<C::Scalar>) -> Option<Self> {
        let adj_segment = &mut unsafe { self.get() };
        let adj_cross = adj_segment.cross.clone();
        use SplitSegments::*;
        match self.adjust_for_intersection(adj_intersection) {
            Unchanged { overlap } => overlap.then(|| self.clone()),
            SplitOnce { overlap, right } => {
                let new_key = Self::create_segment(adj_cross, Some(right), Some(self));
                match overlap {
                    Some(false) => Some(self.clone()),
                    Some(true) => Some(new_key),
                    None => None,
                }
            }
            SplitTwice { right } => {
                Self::create_segment(adj_cross.clone(), Some(right), Some(self));
                let middle = Self::create_segment(adj_cross, Some(adj_intersection), Some(self));
                Some(middle)
            }
        }
    }

    pub fn for_event(event: &Event<C::Scalar, IMSegment<C>>) -> Option<&Segment<C>> {
        use EventType::*;
        Some({
            let segment = unsafe { event.payload.get() };
            if let LineRight = event.ty {
                debug_assert!(segment.geom.is_line());
                if !segment.is_overlapping && segment.geom.right() == event.point {
                    segment
                } else {
                    return None;
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
                segment
            }
        })
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
