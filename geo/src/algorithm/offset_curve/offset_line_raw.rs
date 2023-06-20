use crate::{Coord, CoordFloat};
use super::vector_extensions::VectorExtensions;

pub(super) struct OffsetLineRawResult<T> where T:CoordFloat {
    pub a_offset:Coord<T>,
    pub b_offset:Coord<T>,
    pub ab_len:T,
}


///
/// TODO: Document properly
/// 
/// 
pub(super) fn offset_line_raw<T>(
    a: Coord<T>,
    b: Coord<T>,
    distance: T,
) -> Option<OffsetLineRawResult<T>>
where
    T: CoordFloat,
{
    let ab = b - a;
    let ab_len = ab.magnitude();
    if ab_len == T::zero() {
        return None;
    }
    let ab_offset = ab.left() / ab_len * distance;

    Some(OffsetLineRawResult {
        a_offset: a + ab_offset,
        b_offset: b + ab_offset,
        ab_len,
    })
}
