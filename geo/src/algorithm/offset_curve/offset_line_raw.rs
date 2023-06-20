use crate::{Coord, CoordFloat};
use super::vector_extensions::VectorExtensions;


/// The result of the [offset_line_raw()] function
#[derive(Clone)]
pub(super) struct OffsetLineRawResult<T> where T:CoordFloat {
    pub a_offset:Coord<T>,
    pub b_offset:Coord<T>,
    pub ab_len:T,
}


/// Offset a line defined by [Coord]s `a` and `b` by `distance`.
/// 
/// In a coordinate system where positive is up and to the right;
/// Positive `distance` will offset the line to the left (when standing
/// at `a` and facing `b`)
/// 
/// This could be implemented on [geo_types::Line]...
/// 
/// There are 2 reasons
/// 
/// 1. I am trying to localize my changes to the offset_curve module for now.
/// 2. I am trying to do is avoid repeated calculation of segment length.
///    This function has a special return type which also yields the length.
/// 
/// TODO: In future it may be preferable to create new types called
/// `LineMeasured` and `LineStringMeasured` which store pre-computed length.
/// 
/// - Confirm if significant performance benefit to using a bigger structs to
///   avoid recomputing the line segment length?
/// - I think there certainly might be in future parts of the algorithm which
///   need the length repeatedly)
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
