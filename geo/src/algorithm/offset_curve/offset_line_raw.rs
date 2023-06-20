use crate::{Coord, CoordFloat, Line};
use super::{vector_extensions::VectorExtensions, line_measured::LineMeasured};


/// The result of the [offset_line_raw()] function
// #[derive(Clone)]
// pub(super) struct OffsetLineRawResult<T> where T:CoordFloat {
//     pub a_offset:Coord<T>,
//     pub b_offset:Coord<T>,
//     pub ab_len:T,
// }


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
) -> Option<LineMeasured<T>>
where
    T: CoordFloat,
{
    let ab = b - a;
    let length = ab.magnitude();
    if length == T::zero() {
        return None;
    }
    let ab_offset = ab.left() / length * distance;

    Some(LineMeasured {
        line: Line{
            start:a + ab_offset,
            end: b  + ab_offset
        },
        length,
    })
}


// TODO: test

#[cfg(test)]
mod test {

    use crate::{
        Coord,
        Line,
        offset_curve::{offset_line_raw::offset_line_raw, line_measured::LineMeasured},
    };

    #[test]
    fn test_offset_line_raw() {
        let a = Coord { x: 0f64, y: 0f64 };
        let b = Coord { x: 0f64, y: 1f64 };
        let output_actual = offset_line_raw(a, b, 1f64);
        let output_expected = Some(LineMeasured{
            line:Line { start: Coord { x: 1f64, y: 0f64 }, end: Coord { x: 1f64, y: 1f64 } },
            length:1f64,
        });
        assert_eq!(output_actual, output_expected);
    }
}