use std::{cmp::Ordering, fmt::Debug, rc::Rc};

use crate::sweep::{Active, Event, EventType, LineOrPoint};
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
pub(crate) struct RcSegment<T: GeoNum, P>(Rc<Segment<T, P>>);

impl<T: GeoNum, P> Clone for RcSegment<T, P> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: GeoNum, P> RcSegment<T, P> {
    pub(crate) fn payload(&self) -> &P {
        &self.0.payload
    }

    pub(crate) fn line(&self) -> LineOrPoint<T> {
        self.0.line
    }

    pub fn events(&self) -> [Event<T, RcSegment<T, P>>; 2] {
        let geom = self.0.line;
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
        RcSegment(Rc::new(value))
    }
}

// Implement partial eq, parital ord, and eq for RcSegment
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

// Implement borrow for RcSegment as a line-or-point.
// This is hack to allow querying the active segments for any line-or-point.
use std::borrow::Borrow;
impl<T: GeoNum, P> Borrow<LineOrPoint<T>> for RcSegment<T, P> {
    fn borrow(&self) -> &LineOrPoint<T> {
        &self.0.line
    }
}

impl<T: GeoNum, P> Borrow<Active<LineOrPoint<T>>> for Active<RcSegment<T, P>> {
    fn borrow(&self) -> &Active<LineOrPoint<T>> {
        Active::active_ref(self.0.borrow())
    }
}
