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

        let segment_start = self.position;

        let mut slope_x = Ordering::Equal;
        let mut slope_y = Ordering::Equal;
        for (segment_length, line) in self.coords[segment_start..].windows(2).enumerate() {
            let coord = line[0];
            let next_coord = line[1];

            let next_slope_x = next_coord.x.total_cmp(&coord.x);
            let next_slope_y = next_coord.y.total_cmp(&coord.y);

            fn is_monotonic(previous_slope: &mut Ordering, next_slope: Ordering) -> bool {
                if next_slope == Ordering::Equal {
                    return true;
                }
                match std::mem::replace(previous_slope, next_slope) {
                    Ordering::Equal => true,
                    old_slope => old_slope == next_slope,
                }
            }

            if !is_monotonic(&mut slope_x, next_slope_x)
                || !is_monotonic(&mut slope_y, next_slope_y)
            {
                // `coord` is the end of this segment
                // `next_coord` is the start of the next segment
                self.position += segment_length;
                return Some(MonotoneChainSegment::<T>::new(
                    &self.coords[segment_start..=self.position],
                ));
            }
        }

        // close off the last segment
        self.position = self.coords.len();
        Some(MonotoneChainSegment::<T>::new(
            &self.coords[segment_start..],
        ))
    }
}

impl<'a, T: GeoNum> BoundingRect<T> for MonotoneChainSegment<'a, T> {
    type Output = Rect<T>;

    fn bounding_rect(&self) -> Self::Output {
        self.bounding_rect
    }
}
