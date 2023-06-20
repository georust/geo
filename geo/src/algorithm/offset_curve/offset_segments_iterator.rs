/// I am trying to get a custom iterator working to replace the 
/// [super::slice_itertools::pairwise()] function.
/// 
/// It is turning out to be very complicated :(
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
///    - the pre-calculated length of offset line segments (for miter limit calculation)
///

use crate::{Coord, CoordFloat, CoordNum, LineString};

use super::line_intersection::{
    line_segment_intersection_with_relationships, LineIntersectionResultWithRelationships,
};
use super::offset_line_raw::{offset_line_raw, OffsetLineRawResult};


pub(super) struct OffsetSegmentsIterator<'a, T>
where
    T: CoordFloat,
{
    line_string: &'a LineString<T>,
    distance: T,
    last_offset_segment: Option<OffsetLineRawResult<T>>,
    next_offset_segment: OffsetLineRawResult<T>,
    index: usize,
}

impl<'a, T> OffsetSegmentsIterator<'a, T>
where
    T: CoordFloat,
{
    fn try_new(line_string: &'a LineString<T>, distance: T) -> Option<OffsetSegmentsIterator<'a, T>>
    where
        T: CoordNum,
    {
        if line_string.0.len() < 3 {
            None
        } else {
            let a = line_string.0[0];
            let b = line_string.0[1];
            match offset_line_raw(a, b, distance) {
                Some(offset_result) => Some(OffsetSegmentsIterator {
                    line_string,
                    distance,
                    last_offset_segment: None,
                    next_offset_segment: offset_result,
                    index: 0,
                }),
                _ => None,
            }
        }
    }
}

///
/// ```text
///        a
/// m        \
///   \         \
///     \         b---------c
///       n
///
///          i    o---------p
/// ```
pub(super) struct OffsetSegmentsIteratorItem<T>
where
    T: CoordNum,
{
    a: Coord<T>,
    b: Coord<T>,
    c: Coord<T>,

    m: Coord<T>,
    n: Coord<T>,
    o: Coord<T>,
    p: Coord<T>,

    mn_len:T,
    op_len:T,

    i: LineIntersectionResultWithRelationships<T>,
}

impl<'a, T> Iterator for OffsetSegmentsIterator<'a, T>
where
    T: CoordFloat,
{
    type Item = OffsetSegmentsIteratorItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + 3 > self.line_string.0.len() {
            // TODO: cant tell the difference between terminating early and
            //       completing the iterator all the way.
            //       I think type Item = Option<...> is needed?
            return None;
        } else {
            let a = self.line_string[self.index];
            let b = self.line_string[self.index + 1];
            let c = self.line_string[self.index + 2];

            self.index += 1;

            let Some(OffsetLineRawResult {
                a_offset: m,
                b_offset: n,
                ab_len: mn_len,
            }) = self.last_offset_segment else {
                return None
            };

            let Some(OffsetLineRawResult {
                a_offset: o,
                b_offset: p,
                ab_len: op_len,
            }) = offset_line_raw(b, c, self.distance) else {
                return None
            };

            match line_segment_intersection_with_relationships(&m, &n, &o, &p) {
                Some(i)=>Some(OffsetSegmentsIteratorItem {
                    a,
                    b,
                    c,
                    m,
                    n,
                    o,
                    p,
                    mn_len,
                    op_len,
                    i,
                }),
                _=>None
            }
        }
    }
}
