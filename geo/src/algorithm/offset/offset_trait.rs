use super::{cross_product, line_intersection_with_parameter, LineIntersectionWithParameterResult};
use crate::{
    CoordFloat,
    // Kernel,
    // Orientation,
    Line,
    LineString,
    MultiLineString,
};
use geo_types::Coord;

/// # Offset Trait
///
/// Signed offset of Geometry assuming cartesian coordinate system.
///
/// This is a cheap offset algorithm that is suitable for flat coordinate systems
/// (or if your lat/lon data is near the equator)
///
/// My Priority for implementing the trait is as follows:
/// - [X] Line<impl CoordFloat>
/// - [X] LineString<impl CoordFloat>
/// - [X] MultiLineString<impl CoordFloat>
/// - ... maybe some closed shapes like triangle, polygon?
///
/// The following are a list of known limitations,
/// some may be removed during development,
/// others are very hard to fix.
///
/// - No checking for zero length input.
///   Invalid results may be caused by division by zero.
/// - No check is implemented to prevent execution if the specified offset
///   distance is zero.
/// - Only local cropping where the output is self-intersecting.
///   Non-adjacent line segments in the output may be self-intersecting.
/// - There is no mitre-limit; A LineString which
///   doubles back on itself will produce an elbow at infinity
pub trait Offset<T>
where
    T: CoordFloat,
{
    /// Offset the edges of the geometry by `distance`, where `distance` may be
    /// negative.
    ///
    /// Negative `distance` values will offset the edges of the geometry to the
    /// left, when facing the direction of increasing coordinate index.
    ///
    /// ```
    /// #use crate::{line_string, Coord};
    /// let input = line_string![
    ///     Coord { x: 0f64, y: 0f64 },
    ///     Coord { x: 0f64, y: 2f64 },
    ///     Coord { x: 2f64, y: 2f64 },
    /// ];
    /// let output_expected = line_string![
    ///     Coord { x: 1f64, y: 0f64 },
    ///     Coord { x: 1f64, y: 1f64 },
    ///     Coord { x: 2f64, y: 1f64 },
    /// ];
    /// let output_actual = input.offset(1f64);
    /// assert_eq!(output_actual, output_expected);
    /// ```
    fn offset(&self, distance: T) -> Self;
}

impl<T> Offset<T> for Line<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
        let delta = self.delta();
        let len = (delta.x * delta.x + delta.y * delta.y).sqrt();
        let delta = Coord {
            x: delta.y / len,
            y: -delta.x / len,
        };
        Line::new(self.start + delta * distance, self.end + delta * distance)
    }
}

/// Iterate over a slice in overlapping pairs
///
/// ```ignore
/// let items = vec![1, 2, 3, 4, 5];
/// let actual_result: Vec<(i32, i32)> = pairwise(&items[..]).map(|(a, b)| (*a, *b)).collect();
/// let expected_result = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
/// assert_eq!(actual_result, expected_result);
/// ```
fn pairwise<T>(iterable: &[T]) -> std::iter::Zip<std::slice::Iter<T>, std::slice::Iter<T>> {
    iterable.iter().zip(iterable[1..].iter())
}

impl<T> Offset<T> for LineString<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
        if self.0.len() < 2 {
            // TODO: How should it fail on invalid input?
            return self.clone();
        }

        let offset_segments: Vec<Line<T>> =
            self.lines().map(|item| item.offset(distance)).collect();

        if offset_segments.len() == 1 {
            return offset_segments[0].into();
        }
        let first_point = offset_segments.first().unwrap().start;
        let last_point = offset_segments.last().unwrap().end;

        let mut result = Vec::with_capacity(self.0.len());
        result.push(first_point);
        result.extend(pairwise(&offset_segments[..]).flat_map(
            |(Line { start: a, end: b }, Line { start: c, end: d })| {
                let ab = *b - *a;
                let cd = *d - *c;
                let ab_cross_cd = cross_product(ab, cd);
                // TODO: I'm still confused about how to use Kernel / RobustKernel;
                //       the following did not work. I need to read more code
                //       from the rest of this repo to understand.
                // if Kernel::orient2d(*a, *b, *d) == Orientation::Collinear {
                //       note that it is sufficient to check that only one of
                //       c or d are colinear with ab because of how they are
                //       related by the original line string.
                // TODO: The following line
                //       - Does not use the Kernel
                //       - uses an arbitrary threshold value which needs more thought
                if <f64 as num_traits::NumCast>::from(ab_cross_cd)
                    .unwrap()
                    .abs()
                    < num_traits::cast(0.0000001f64).unwrap()
                {
                    vec![*b]
                } else {
                    // TODO: if we can inline this function we only need to
                    //       calculate `ab_cross_cd` once
                    let LineIntersectionWithParameterResult {
                        t_ab,
                        t_cd,
                        intersection,
                    } = line_intersection_with_parameter(a, b, c, d);

                    let zero = num_traits::zero::<T>();
                    let one = num_traits::one::<T>();

                    let tip_ab = zero <= t_ab && t_ab <= one;
                    let fip_ab = !tip_ab;
                    let pfip_ab = fip_ab && t_ab > zero;

                    let tip_cd = zero <= t_cd && t_cd <= one;
                    let fip_cd = !tip_cd;

                    if tip_ab && tip_cd {
                        // TODO: test for mitre limit
                        vec![intersection]
                    } else if fip_ab && fip_cd && pfip_ab {
                        // TODO: test for mitre limit
                        vec![intersection]
                    } else {
                        vec![*b, *c]
                    }
                }
            },
        ));
        result.push(last_point);
        // TODO: there are more steps to this algorithm which are not yet
        //       implemented. See rfcs\2022-11-11-offset.md
        result.into()
    }
}

impl<T> Offset<T> for MultiLineString<T>
where
    T: CoordFloat,
{
    fn offset(&self, distance: T) -> Self {
        self.iter().map(|item| item.offset(distance)).collect()
    }
}

#[cfg(test)]
mod test {

    // crate dependencies
    use crate::{line_string, Coord, Line, MultiLineString, Offset};

    // private imports
    use super::pairwise;

    #[test]
    fn test_pairwise() {
        let items = vec![1, 2, 3, 4, 5];
        let actual_result: Vec<(i32, i32)> = pairwise(&items[..]).map(|(a, b)| (*a, *b)).collect();
        let expected_result = vec![(1, 2), (2, 3), (3, 4), (4, 5)];
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn test_offset_line() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let actual_result = input.offset(1.0);
        assert_eq!(
            actual_result,
            Line::new(Coord { x: 2f64, y: 1f64 }, Coord { x: 2f64, y: 2f64 },)
        );
    }
    #[test]
    fn test_offset_line_negative() {
        let input = Line::new(Coord { x: 1f64, y: 1f64 }, Coord { x: 1f64, y: 2f64 });
        let output_actual = input.offset(-1.0);
        let output_expected = Line::new(Coord { x: 0f64, y: 1f64 }, Coord { x: 0f64, y: 2f64 });
        assert_eq!(output_actual, output_expected);
    }

    #[test]
    fn test_offset_line_string() {
        let input = line_string![
            Coord { x: 0f64, y: 0f64 },
            Coord { x: 0f64, y: 2f64 },
            Coord { x: 2f64, y: 2f64 },
        ];
        let output_expected = line_string![
            Coord { x: 1f64, y: 0f64 },
            Coord { x: 1f64, y: 1f64 },
            Coord { x: 2f64, y: 1f64 },
        ];
        let output_actual = input.offset(1f64);
        assert_eq!(output_actual, output_expected);
    }

    #[test]
    fn test_offset_multi_line_string() {
        let input = MultiLineString::new(vec![
            line_string![
                Coord { x: 0f64, y: 0f64 },
                Coord { x: 0f64, y: 2f64 },
                Coord { x: 2f64, y: 2f64 },
            ],
            line_string![
                Coord { x: 0f64, y: 0f64 },
                Coord { x: 0f64, y: -2f64 },
                Coord { x: -2f64, y: -2f64 },
            ],
        ]);
        let output_expected = MultiLineString::new(vec![
            line_string![
                Coord { x: 1f64, y: 0f64 },
                Coord { x: 1f64, y: 1f64 },
                Coord { x: 2f64, y: 1f64 },
            ],
            line_string![
                Coord { x: -1f64, y: 0f64 },
                Coord { x: -1f64, y: -1f64 },
                Coord { x: -2f64, y: -1f64 },
            ],
        ]);
        let output_actual = input.offset(1f64);
        assert_eq!(output_actual, output_expected);
    }
}
