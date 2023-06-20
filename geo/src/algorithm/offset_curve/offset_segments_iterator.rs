///
///  My requirements are
///
///  - Facilitate iterating over `Line`s in a LineString in a pairwise fashion
///  - Offset the `Line` inside the iterator
///  - Avoid repeatedly calculating length for each line
///  - Make iterator lazier (don't keep all offset `Line`s in memory)
///  - Iterator should provide
///    - the offset points
///    - the intersection point ([LineIntersectionResultWithRelationships])
///    - the pre-calculated length of offset line segments (for miter limit
///      calculation)
///    - support wrapping over to the first segment at the end to simplify
///      closed shapes
///
use crate::{Coord, CoordFloat, CoordNum, LineString};

use super::{
    line_intersection::{
        line_segment_intersection_with_relationships, LineIntersectionResultWithRelationships,
    },
    line_measured::LineMeasured,
    offset_line_raw::offset_line_raw,
};

/// Bring this into scope to imbue [LineString] with
/// [iter_offset_segment_pairs()]
pub(super) trait LineStringOffsetSegmentPairs<T>
where
    T: CoordFloat,
{
    /// Loop over the segments of a [LineString] in a pairwise fashion,
    /// offsetting and intersecting them as we go.
    ///
    /// Returns an [OffsetSegmentsIterator]
    fn iter_offset_segment_pairs(&self, distance: T) -> OffsetSegmentsIterator<T>;
}

pub(super) struct OffsetSegmentsIterator<'a, T>
where
    T: CoordFloat,
{
    line_string: &'a LineString<T>,
    distance: T,
    previous_offset_segment: Option<LineMeasured<T>>,
    index: usize,
}

impl<T> LineStringOffsetSegmentPairs<T> for LineString<T>
where
    T: CoordFloat,
{
    fn iter_offset_segment_pairs(&self, distance: T) -> OffsetSegmentsIterator<T>
    where
        T: CoordNum,
    {
        if self.0.len() < 3 {
            // LineString is not long enough, therefore return an iterator that
            // will return None as first result
            OffsetSegmentsIterator {
                line_string: self,
                distance,
                previous_offset_segment: None,
                index: 0,
            }
        } else {
            // TODO: Length check above prevents panic; use
            // unsafe get_unchecked for performance?
            let a = self.0[0];
            let b = self.0[1];
            OffsetSegmentsIterator {
                line_string: self,
                distance,
                previous_offset_segment: offset_line_raw(a, b, distance),
                index: 0,
            }
        }
    }
}


///  - [LineString] `a---b---c` is offset to form
///  - [LineMeasured] `ab_offset` (`a'---b'`) and
///  - [LineMeasured] `bc_offset` (`b'---c'`)
///  - [LineIntersectionResultWithRelationships] `i` is the intersection point.
///
/// ```text
///          a
///  a'       \
///   \        \
///    \        b---------c
///     b'
///
///        i    b'--------c'
/// ```
#[derive(Clone, Debug)]
pub(super) struct OffsetSegmentsIteratorItem<T>
where
    T: CoordNum,
{
    /// This is true for the first result
    pub first: bool,

    // this is true for the last result
    pub last: bool,

    // TODO: seems a,b,c are unused...
    pub a: Coord<T>,
    pub b: Coord<T>,
    pub c: Coord<T>,

    pub ab_offset: Option<LineMeasured<T>>,
    pub bc_offset: Option<LineMeasured<T>>,

    /// Intersection [Coord] between segments `mn` and `op`
    pub i: Option<LineIntersectionResultWithRelationships<T>>,
}

impl<'a, T> Iterator for OffsetSegmentsIterator<'a, T>
where
    T: CoordFloat,
{
    /// Option since each step of the iteration may fail.
    type Item = OffsetSegmentsIteratorItem<T>;

    /// Return type is confusing; `Option<Option<OffsetSegmentsIteratorItem<T>>>`
    ///
    /// TODO: Revise
    ///
    /// The outer Option is required by the Iterator trait, and indicates if
    /// iteration is finished, (When this iterator is used via `.map()` or
    /// similar the user does not see the outer Option.)
    /// The inner Option indicates if the result of each iteration is valid.
    /// Returning None will halt iteration, returning Some(None) will not,
    /// but the user should stop iterating.
    ///
    fn next(&mut self) -> Option<Self::Item> {
        if self.index + 3 > self.line_string.0.len() {
            // Iteration is complete
            return None;
        } else {
            // TODO: Length check above prevents panic; use
            // unsafe get_unchecked for performance?
            let a = self.line_string[self.index];
            let b = self.line_string[self.index + 1];
            let c = self.line_string[self.index + 2];

            self.index += 1;

            // Fetch previous offset segment
            let ab_offset = self.previous_offset_segment.clone();

            // Compute next offset segment
            self.previous_offset_segment = offset_line_raw(b, c, self.distance);

            Some(OffsetSegmentsIteratorItem {
                first: self.index == 1,
                last: self.index + 3 > self.line_string.0.len(),
                a,
                b,
                c,
                i: match (&ab_offset, &self.previous_offset_segment) {
                    (Some(ab_offset), Some(bc_offset)) => {
                        line_segment_intersection_with_relationships(
                            ab_offset.line.start,
                            ab_offset.line.end,
                            bc_offset.line.start,
                            bc_offset.line.end,
                        )
                    }
                    _ => None,
                },
                ab_offset,
                bc_offset: self.previous_offset_segment.clone(),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::{LineStringOffsetSegmentPairs, OffsetSegmentsIteratorItem};
    use crate::{
        line_string,
        offset_curve::{
            line_intersection::LineIntersectionResultWithRelationships, line_measured::LineMeasured,
        },
        Coord,
    };

    #[test]
    fn test_iterator() {
        let input = line_string![
            Coord { x: 1f64, y: 0f64 },
            Coord { x: 1f64, y: 1f64 },
            Coord { x: 2f64, y: 1f64 },
        ];

        // TODO: this test is a bit useless after recent changes
        let result: Option<Vec<()>> = input
            .iter_offset_segment_pairs(1f64)
            .map(|item| match item {
                OffsetSegmentsIteratorItem {
                    ab_offset: Some(LineMeasured { .. }),
                    bc_offset: Some(LineMeasured { .. }),
                    i: Some(LineIntersectionResultWithRelationships { .. }),
                    ..
                } => Some(()),
                _ => None,
            })
            .collect();
        assert!(result.unwrap().len() == 1);
    }
}
