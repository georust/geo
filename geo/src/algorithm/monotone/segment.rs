use std::cell::{Ref, RefCell};
use std::{cmp::Ordering, fmt::Debug, rc::Rc};

use crate::sweep::{Event, EventType, LineOrPoint, SweepPoint};
use crate::GeoNum;

/// A segment in the sweep line algorithm.
///
/// Consists of a line and a payload.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Segment<T: GeoNum, P> {
    line: LineOrPoint<T>,
    payload: P,
}

impl<T: GeoNum, P> Segment<T, P> {}

impl<T: GeoNum, P> PartialOrd for Segment<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.line.partial_cmp(&other.line)
    }
}

impl<T: GeoNum, P> PartialEq for Segment<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl<T: GeoNum> From<LineOrPoint<T>> for Segment<T, ()> {
    fn from(line: LineOrPoint<T>) -> Self {
        Segment { line, payload: () }
    }
}

impl<T: GeoNum, P> From<(LineOrPoint<T>, P)> for Segment<T, P> {
    fn from((line, payload): (LineOrPoint<T>, P)) -> Self {
        Segment { line, payload }
    }
}

#[derive(Debug)]
pub(crate) struct RcSegment<T: GeoNum, P>(Rc<RefCell<Segment<T, P>>>);

impl<T: GeoNum, P> Clone for RcSegment<T, P> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: GeoNum, P: Clone + Debug> RcSegment<T, P> {
    pub(crate) fn split_at(&self, pt: SweepPoint<T>) -> Self {
        debug!("Splitting segment {:?} at {:?}", self, pt);
        let mut borrow = RefCell::borrow_mut(&self.0);
        let right = borrow.line.right();
        borrow.line = LineOrPoint::from((borrow.line.left(), pt));

        let new_seg = Segment::from((LineOrPoint::from((pt, right)), borrow.payload.clone()));
        Self(Rc::new(new_seg.into()))
    }
}

impl<T: GeoNum, P> RcSegment<T, P> {
    pub(crate) fn payload(&self) -> Ref<P> {
        let borrow = RefCell::borrow(&self.0);
        Ref::map(borrow, |s| &s.payload)
    }

    pub(crate) fn line(&self) -> LineOrPoint<T> {
        RefCell::borrow(&self.0).line
    }

    #[inline]
    pub fn events(&self) -> [Event<T, RcSegment<T, P>>; 2] {
        let geom = RefCell::borrow(&self.0).line;
        let left = geom.left();
        let right = geom.right();
        [
            Event {
                point: left,
                ty: if geom.is_line() {
                    EventType::LineLeft
                } else {
                    EventType::PointLeft
                },
                payload: self.clone(),
            },
            Event {
                point: right,
                ty: if geom.is_line() {
                    EventType::LineRight
                } else {
                    EventType::PointRight
                },
                payload: self.clone(),
            },
        ]
    }
}

impl<T: GeoNum, P> From<Segment<T, P>> for RcSegment<T, P> {
    fn from(value: Segment<T, P>) -> Self {
        RcSegment(Rc::new(value.into()))
    }
}

// Implement partial eq, partial ord, and eq for RcSegment
impl<T: GeoNum, P> PartialEq for RcSegment<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T: GeoNum, P> PartialOrd for RcSegment<T, P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
