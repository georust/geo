use super::{LineSplit, LineSplitResult};
use crate::Vector2DOps;
use geo_types::{CoordFloat, Line};

impl<Scalar> LineSplit<Scalar> for Line<Scalar>
where
    Scalar: CoordFloat,
{
    /// Split a [Line] or at some `fraction` of its length.
    ///
    /// The `fraction` argument is any real number.
    /// Only values between 0.0 and 1.0 will split the line.
    /// Values outside of this range (including infinite values) will be clamped to 0.0 or 1.0.
    ///
    /// Returns [None] when
    /// - The provided `fraction` is NAN
    /// - The the object being sliced includes NAN or infinite coordinates
    ///
    /// Otherwise Returns a [`Some(LineSplitResult<Line>)`](LineSplitResult)
    ///
    /// example
    ///
    /// ```
    /// use geo::{Line, coord};
    /// use geo::algorithm::{LineSplit, LineSplitResult};
    /// let line = Line::new(
    ///     coord! {x: 0.0, y:0.0},
    ///     coord! {x:10.0, y:0.0},
    /// );
    /// let result = line.line_split(0.6);
    /// assert_eq!(
    ///     result,
    ///     Some(LineSplitResult::FirstSecond(
    ///         Line::new(
    ///             coord! {x: 0.0, y:0.0},
    ///             coord! {x: 6.0, y:0.0},
    ///         ),
    ///         Line::new(
    ///             coord! {x: 6.0, y:0.0},
    ///             coord! {x:10.0, y:0.0},
    ///         )
    ///     ))
    /// );
    /// ```
    fn line_split(&self, fraction: Scalar) -> Option<LineSplitResult<Self>> {
        if fraction.is_nan() {
            return None;
        }
        if fraction <= Scalar::zero() {
            Some(LineSplitResult::Second(*self))
        } else if fraction >= Scalar::one() {
            Some(LineSplitResult::First(*self))
        } else {
            let new_midpoint = self.start + self.delta() * fraction;
            if new_midpoint.is_finite() {
                Some(LineSplitResult::FirstSecond(
                    Line::new(self.start, new_midpoint),
                    Line::new(new_midpoint, self.end),
                ))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::LineSplitTwiceResult;
    use super::*;
    use geo_types::coord;

    // =============================================================================================
    // Line::line_split()
    // =============================================================================================

    #[test]
    fn test_line_split_first_second() {
        // simple x-axis aligned check
        let line = Line::new(
            coord! {x: 0.0_f32, y:0.0_f32},
            coord! {x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(0.6);
        assert_eq!(
            result,
            Some(LineSplitResult::FirstSecond(
                Line::new(
                    coord! {x: 0.0_f32, y:0.0_f32},
                    coord! {x: 6.0_f32, y:0.0_f32},
                ),
                Line::new(
                    coord! {x: 6.0_f32, y:0.0_f32},
                    coord! {x:10.0_f32, y:0.0_f32},
                )
            ))
        );

        // simple y-axis aligned check
        let line = Line::new(
            coord! {x:0.0_f32, y: 0.0_f32},
            coord! {x:0.0_f32, y:10.0_f32},
        );
        let result = line.line_split(0.3);
        assert_eq!(
            result,
            Some(LineSplitResult::FirstSecond(
                Line::new(coord! {x:0.0_f32, y:0.0_f32}, coord! {x:0.0_f32, y:3.0_f32},),
                Line::new(
                    coord! {x:0.0_f32, y:3.0_f32},
                    coord! {x:0.0_f32, y:10.0_f32},
                )
            ))
        );

        // non_trivial check
        let line = Line::new(
            coord! {x: 1.0_f32, y:  1.0_f32},
            coord! {x:10.0_f32, y:-10.0_f32},
        );
        let split_point = line.start + line.delta() * 0.7;
        let result = line.line_split(0.7);
        assert_eq!(
            result,
            Some(LineSplitResult::FirstSecond(
                Line::new(line.start, split_point,),
                Line::new(split_point, line.end,)
            ))
        );
    }

    #[test]
    fn test_line_split_first() {
        // test one
        let line = Line::new(
            coord! {x: 0.0_f32, y:0.0_f32},
            coord! {x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(1.0);
        assert_eq!(result, Some(LineSplitResult::First(line)));

        // Test numbers larger than one
        let line = Line::new(
            coord! {x: 0.0_f32, y:0.0_f32},
            coord! {x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(2.0);
        assert_eq!(result, Some(LineSplitResult::First(line)));
    }
    #[test]
    fn test_line_split_second() {
        // test zero
        let line = Line::new(
            coord! {x: 0.0_f32, y:0.0_f32},
            coord! {x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(0.0);
        assert_eq!(result, Some(LineSplitResult::Second(line)));

        // Test negative numbers
        let line = Line::new(
            coord! {x: 0.0_f32, y:0.0_f32},
            coord! {x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(-2.0);
        assert_eq!(result, Some(LineSplitResult::Second(line)));
    }

    // =============================================================================================
    // Line::line_split_twice()
    // =============================================================================================

    macro_rules! test_line_split_twice_helper{
        ($a:expr, $b:expr, $enum_variant:ident, $(($x1:expr, $x2:expr)),*)=>{{
            let line = Line::new(
                coord!{x: 0.0_f32, y:0.0_f32},
                coord!{x:10.0_f32, y:0.0_f32},
            );
            let result = line.line_split_twice($a, $b).unwrap();
            // println!("{result:?}");
            assert_eq!(
                result,
                LineSplitTwiceResult::$enum_variant(
                    $(
                        Line::new(
                            coord!{x: $x1, y:0.0_f32},
                            coord!{x: $x2, y:0.0_f32},
                        ),
                    )*
                )
            );
        }}
    }

    #[test]
    fn test_line_split_twice() {
        test_line_split_twice_helper!(
            0.6,
            0.8,
            FirstSecondThird,
            (0.0, 6.0),
            (6.0, 8.0),
            (8.0, 10.0)
        );
        test_line_split_twice_helper!(0.6, 1.0, FirstSecond, (0.0, 6.0), (6.0, 10.0));
        test_line_split_twice_helper!(0.6, 0.6, FirstThird, (0.0, 6.0), (6.0, 10.0));
        test_line_split_twice_helper!(0.0, 0.6, SecondThird, (0.0, 6.0), (6.0, 10.0));
        test_line_split_twice_helper!(1.0, 1.0, First, (0.0, 10.0));
        test_line_split_twice_helper!(0.0, 1.0, Second, (0.0, 10.0));
        test_line_split_twice_helper!(0.0, 0.0, Third, (0.0, 10.0));
    }

    // =============================================================================================
    // Line::line_split_many()
    // =============================================================================================

    #[test]
    fn test_line_split_many() {
        let line = Line::new(
            coord! {x: 0.0, y:0.0},
            coord! {x:10.0, y:0.0},
        );
        let result = line.line_split_many(&vec![0.1, 0.2, 0.5]);
        assert_eq!(
            result,
            Some(vec![
                Some(Line::new(
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 1.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 1.0, y: 0.0 },
                    coord! { x: 2.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 2.0, y: 0.0 },
                    coord! { x: 5.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 5.0, y: 0.0 },
                    coord! { x: 10.0, y: 0.0 },
                ))
            ])
        );
    }

    #[test]
    fn test_line_split_many_edge_right() {
        let line = Line::new(
            coord! {x: 0.0, y:0.0},
            coord! {x:10.0, y:0.0},
        );
        let result = line.line_split_many(&vec![0.1, 0.2, 2.0]);
        assert_eq!(
            result,
            Some(vec![
                Some(Line::new(
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 1.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 1.0, y: 0.0 },
                    coord! { x: 2.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 2.0, y: 0.0 },
                    coord! { x:10.0, y: 0.0 },
                )),
                None
            ])
        );
    }

    #[test]
    fn test_line_split_many_double_edge_right() {
        let line = Line::new(
            coord! {x: 0.0, y:0.0},
            coord! {x:10.0, y:0.0},
        );
        let result = line.line_split_many(&vec![0.1, 1.2, 2.0]);
        assert_eq!(
            result,
            Some(vec![
                Some(Line::new(
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 1.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 1.0, y: 0.0 },
                    coord! { x:10.0, y: 0.0 },
                )),
                None,
                None
            ])
        );
    }

    #[test]
    fn test_line_split_many_edge_left() {
        let line = Line::new(
            coord! {x: 0.0, y:0.0},
            coord! {x:10.0, y:0.0},
        );
        let result = line.line_split_many(&vec![-1.0, 0.2, 0.5]);
        assert_eq!(
            result,
            Some(vec![
                None,
                Some(Line::new(
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 2.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 2.0, y: 0.0 },
                    coord! { x: 5.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 5.0, y: 0.0 },
                    coord! { x: 10.0, y: 0.0 },
                ))
            ])
        );
    }

    #[test]
    fn test_line_split_many_double_edge_left() {
        let line = Line::new(
            coord! {x: 0.0, y:0.0},
            coord! {x:10.0, y:0.0},
        );
        let result = line.line_split_many(&vec![-1.0, -0.5, 0.5]);
        assert_eq!(
            result,
            Some(vec![
                None,
                None,
                Some(Line::new(
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 5.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 5.0, y: 0.0 },
                    coord! { x: 10.0, y: 0.0 },
                ))
            ])
        );
    }

    #[test]
    fn test_line_split_many_same_value() {
        let line = Line::new(
            coord! {x: 0.0, y:0.0},
            coord! {x:10.0, y:0.0},
        );
        let result = line.line_split_many(&vec![0.2, 0.2, 0.5]);
        assert_eq!(
            result,
            Some(vec![
                Some(Line::new(
                    coord! { x: 0.0, y: 0.0 },
                    coord! { x: 2.0, y: 0.0 },
                )),
                None,
                Some(Line::new(
                    coord! { x: 2.0, y: 0.0 },
                    coord! { x: 5.0, y: 0.0 },
                )),
                Some(Line::new(
                    coord! { x: 5.0, y: 0.0 },
                    coord! { x: 10.0, y: 0.0 },
                ))
            ])
        );
    }
}
