use super::util::is_monotone;
use crate::algorithm::bounding_rect::BoundingRect;
use crate::geometry::*;
use crate::{CoordNum, GeoNum};
use std::cmp::Ordering;

/// A [`MonotoneChainSegment`] is a slice of a [`LineString`] where the coordinates are monotonic  
/// ie. the coordinates are monotonically ordered in both x and y directions  
/// This primitive is not constructed directly, it is created when building a [`MonotoneChain`](`super::MonotoneChain`)
#[derive(Debug, Clone)]
pub struct MonotoneChainSegment<'a, T: CoordNum> {
    ls: &'a [Coord<T>],
    bounding_rect: Rect<T>,
}

impl<'a: 'caller, 'caller, T: GeoNum> MonotoneChainSegment<'a, T> {
    /// Create a new `MonotoneChainSegment` from a `LineString`
    /// Requires the `LineString` to be monotone
    /// i.e. the coordinates must be in increasing or decreasing order in both x and y
    pub(crate) fn new(linestring: &'a [Coord<T>]) -> Self {
        debug_assert!(!linestring.is_empty());
        debug_assert!(is_monotone(linestring));

        Self {
            ls: linestring,
            bounding_rect: Rect::<T>::new(
                linestring[0],
                *linestring.last().expect("LineString should not be empty"),
            ),
        }
    }

    /// Split the segment into two segments at the halfway mark for binary search
    pub fn divide(
        &'caller self,
    ) -> (
        MonotoneChainSegment<'a, T>,
        Option<MonotoneChainSegment<'a, T>>,
    ) {
        // if segment is a line, cannot be divided
        if self.ls.len() == 2 {
            return (self.clone(), None);
        }

        let mid = self.ls.len() / 2;
        let left = Self::new(&self.ls[0..mid + 1]);
        let right = Self::new(&self.ls[mid..self.ls.len()]);

        (left, Some(right))
    }
}

impl<'a: 'caller, 'caller, T: GeoNum> MonotoneChainSegment<'a, T> {
    pub fn ls(&'caller self) -> &'a [Coord<T>] {
        self.ls
    }
}

/// A factory for [`MonotoneChainSegment`]s from a [`Coord`] slice
pub(crate) struct MonotoneChainSegmentFactory<'a, T: GeoNum> {
    coords: &'a [Coord<T>],
    position: usize,
}

impl<'a, T: GeoNum> MonotoneChainSegmentFactory<'a, T> {
    pub fn new(coords: &'a [Coord<T>]) -> Self {
        Self {
            coords,
            position: 0,
        }
    }
}

impl<'a, T: GeoNum> Iterator for MonotoneChainSegmentFactory<'a, T> {
    type Item = MonotoneChainSegment<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        // terminate iterator
        if self.position == self.coords.len() {
            return None;
        }

        // edge case when there is only one point
        if self.position == self.coords.len() - 1 {
            self.position = self.coords.len();

            return Some(MonotoneChainSegment::<T>::new(
                &self.coords[self.coords.len() - 1..self.coords.len()],
            ));
        }

        let c_low = self.position;

        let mut dx = self.coords[c_low + 1].x.total_cmp(&self.coords[c_low].x);
        let mut dy = self.coords[c_low + 1].y.total_cmp(&self.coords[c_low].y);

        for i in (c_low + 2)..self.coords.len() {
            let c0_i = i - 1;
            let c1_i = i;
            let c0 = self.coords[c0_i];
            let c1 = self.coords[c1_i];

            let cmpx = c1.x.total_cmp(&c0.x);
            let cmpy = c1.y.total_cmp(&c0.y);

            let monotonic_x = (dx == Ordering::Equal) || (dx == cmpx) || (cmpx == Ordering::Equal);
            let monotonic_y = (dy == Ordering::Equal) || (dy == cmpy) || (cmpy == Ordering::Equal);

            if monotonic_x && monotonic_y {
                // c1 extends the current segment
                dx = if dx == Ordering::Equal { cmpx } else { dx };
                dy = if dy == Ordering::Equal { cmpy } else { dy };
            } else {
                // c0 is the inclusive start of a next segment
                // c1 is the exclusive end of the current segment
                self.position = c0_i;
                return Some(MonotoneChainSegment::<T>::new(&self.coords[c_low..c1_i]));
            }
        }

        // close off the last segment
        self.position = self.coords.len();
        Some(MonotoneChainSegment::<T>::new(
            &self.coords[c_low..self.coords.len()],
        ))
    }
}

impl<'a, T: GeoNum> BoundingRect<T> for MonotoneChainSegment<'a, T> {
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        self.bounding_rect
    }
}
