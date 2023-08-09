//!


use crate::{CoordFloat, EuclideanLength, Line, LineString};


#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum LineSplitResult<T> {
    First       (T   ),
    Second      (   T),
    FirstSecond (T, T),
}


#[rustfmt::skip]
impl<T> LineSplitResult<T>{
    pub fn first(&self) -> Option<&T> {
        match self {
            Self::First      (x   ) => Some(x),
            Self::Second     (   _) => None,
            Self::FirstSecond(x, _) => Some(x),
        }
    }
    pub fn into_first(self) -> Option<T> {
        match self {
            Self::First      (x   ) => Some(x),
            Self::Second     (   _) => None,
            Self::FirstSecond(x, _) => Some(x),
        }
    }
    pub fn second(&self) -> Option<&T> {
        match self {
            Self::First      (_   ) => None,
            Self::Second     (   x) => Some(x),
            Self::FirstSecond(_, x) => Some(x),
        }
    }
    pub fn into_second(self) -> Option<T> {
        match self {
            Self::First      (_   ) => None,
            Self::Second     (   x) => Some(x),
            Self::FirstSecond(_, x) => Some(x),
        }
    }

    pub fn into_tuple(self) -> (Option<T>, Option<T>) {
        match self {
            Self::First      (a   ) => (Some(a), None   ),
            Self::Second     (   b) => (None   , Some(b)),
            Self::FirstSecond(a, b) => (Some(a), Some(b)),
        }
    }
}

#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum LineSplitTwiceResult<T> {
    First            (T      ),
    Second           (   T   ),
    Third            (      T),
    FirstSecond      (T, T   ),
    SecondThird      (   T, T),
    FirstThird       (T,    T),
    FirstSecondThird (T, T, T),
}


#[rustfmt::skip]
impl<T> LineSplitTwiceResult<T> {
    pub fn first(&self) -> Option<&T> {
        match self {
            Self::First           (x      ) => Some(x),
            Self::Second          (   _   ) => None,
            Self::Third           (      _) => None,
            Self::FirstSecond     (x, _   ) => Some(x),
            Self::SecondThird     (   _, _) => None,
            Self::FirstThird      (x,    _) => Some(x),
            Self::FirstSecondThird(x, _, _) => Some(x),
        }
    }
    pub fn into_first(self) -> Option<T> {
        match self {
            Self::First           (x      ) => Some(x),
            Self::Second          (   _   ) => None,
            Self::Third           (      _) => None,
            Self::FirstSecond     (x, _   ) => Some(x),
            Self::SecondThird     (   _, _) => None,
            Self::FirstThird      (x,    _) => Some(x),
            Self::FirstSecondThird(x, _, _) => Some(x),
        }
    }
    pub fn second(&self) -> Option<&T> {
        match self {
            Self::First           (_      ) => None,
            Self::Second          (   x   ) => Some(x),
            Self::Third           (      _) => None,
            Self::FirstSecond     (_, x   ) => Some(x),
            Self::SecondThird     (   x, _) => Some(x),
            Self::FirstThird      (_,    _) => None,
            Self::FirstSecondThird(_, x, _) => Some(x),
        }
    }
    pub fn into_second(self) -> Option<T> {
        match self {
            Self::First           (_      ) => None,
            Self::Second          (   x   ) => Some(x),
            Self::Third           (      _) => None,
            Self::FirstSecond     (_, x   ) => Some(x),
            Self::SecondThird     (   x, _) => Some(x),
            Self::FirstThird      (_,    _) => None,
            Self::FirstSecondThird(_, x, _) => Some(x),
        }
    }
    pub fn third(&self) -> Option<&T> {
        match self {
            Self::First           (_      ) => None,
            Self::Second          (   x   ) => Some(x),
            Self::Third           (      _) => None,
            Self::FirstSecond     (_, x   ) => Some(x),
            Self::SecondThird     (   x, _) => Some(x),
            Self::FirstThird      (_,    _) => None,
            Self::FirstSecondThird(_, x, _) => Some(x),
        }
    }
    pub fn into_third(self) -> Option<T> {
        match self {
            Self::First           (_      ) => None,
            Self::Second          (   _   ) => None,
            Self::Third           (      x) => Some(x),
            Self::FirstSecond     (_, _   ) => None,
            Self::SecondThird     (   _, x) => Some(x),
            Self::FirstThird      (_,    x) => Some(x),
            Self::FirstSecondThird(_, _, x) => Some(x),
        }
    }
    pub fn into_tuple(self) -> (Option<T>, Option<T>, Option<T>) {
        match self {
            Self::First           (a      ) => (Some(a), None   , None   ),
            Self::Second          (   b   ) => (None   , Some(b), None   ),
            Self::Third           (      c) => (None   , None   , Some(c)),
            Self::FirstSecond     (a, b   ) => (Some(a), Some(b), None   ),
            Self::SecondThird     (   b, c) => (None   , Some(b), Some(c)),
            Self::FirstThird      (a,    c) => (Some(a), None   , Some(c)),
            Self::FirstSecondThird(a, b, c) => (Some(a), Some(b), Some(c)),
        }
    }
}

#[derive(PartialEq, Debug)]
struct LineStringMeasurements<Scalar>{
    length_total:Scalar,
    length_segments:Vec<Scalar>,
}
/// Simultaneously measure the total length of a line and the length of each segment
/// Returns `None` when
/// 
/// - The `LineString` has less than two coords
/// - The resulting total_length is not finite
// TODO: consider re-implementing as a trait?
fn measure_line_string<Scalar>(line_string:&LineString<Scalar>) -> Option<LineStringMeasurements<Scalar>> where 
    Scalar:CoordFloat,
    Line<Scalar>: EuclideanLength<Scalar>
{
    let result = line_string.lines().fold(
        LineStringMeasurements{length_total:Scalar::zero(), length_segments: Vec::new()},
        |LineStringMeasurements{length_total, mut length_segments}, current| {
            let segment_length = current.euclidean_length();
            length_segments.push(segment_length);
            LineStringMeasurements{
                length_total:length_total+segment_length,
                length_segments
            }
        }
    );
    if result.length_total==Scalar::zero() || !result.length_total.is_finite() {
        None
    }else{
        Some(result)
    }
}

///
///
///
pub trait LineSplit<Scalar> where Self:Sized, Scalar: CoordFloat {

    /// Note on choice of return type:
    /// 
    /// You may wonder why this does not return `Option<(Option<Line>, Option<Line>)>`?
    /// It is because then the return type causes uncertainty; The user may expect to possibly
    /// receive `Some((None, None))` which is never possible, this would lead to clutter in match
    /// statements.
    /// 
    /// To make it easier to 'just get the first' or 'just get the second' you can use
    /// `LineSplitResult::first()` and `LineSplitResult::second()` which return `Option<T>`
    /// 
    /// 
    fn line_split(&self, fraction: Scalar) -> Option<LineSplitResult<Self>>;

    /// Note on choice of return type:
    /// 
    /// You may wonder why this does not return `Option<(Option<Line>,Option<Line>,Option<Line>)>`?
    /// It is because then the return type causes uncertainty; The user may expect to possibly
    /// receive `Some((None, None, None))` which is never possible.
    /// The user would have a hard time writing an exhaustive match statement.
    /// 
    /// To make it easier to 'just get the second' the `LineSplitResult` has a function called `first()->Option<T>`
    /// 
    // TODO: I only want to skip formatting the match block, but because attributes on expressions
    //       are experimental we are forced to put it on the function to avoid an error message.
    #[rustfmt::skip]
    fn line_split_twice(
        &self,
        start_fraction: Scalar,
        end_fraction: Scalar,
    ) -> Option<LineSplitTwiceResult<Self>> {
        // import enum variants
        use LineSplitTwiceResult::*;

        // forgive the user for passing in the wrong order
        // because it simplifies the interface of the output type
        let (start_fraction, end_fraction) = if start_fraction > end_fraction {
            (end_fraction, start_fraction)
        } else {
            (start_fraction, end_fraction)
        };
        // TODO: check for nan
        let second_fraction = (end_fraction - start_fraction) / (Scalar::one() - start_fraction);

        match self.line_split(start_fraction) {
            Some(LineSplitResult::FirstSecond(line1, line2)) => match line2.line_split(second_fraction) {
                Some(LineSplitResult::FirstSecond(line2, line3)) => Some(FirstSecondThird(line1, line2, line3)),
                Some(LineSplitResult::First      (line2       )) => Some(FirstSecond     (line1, line2       )),
                Some(LineSplitResult::Second     (       line3)) => Some(FirstThird      (line1,        line3)),
                None => None,
            },
            Some(LineSplitResult::First (line1)) => Some(First(line1)),
            Some(LineSplitResult::Second(line2)) => match line2.line_split(second_fraction) {
                Some(LineSplitResult::FirstSecond(line2, line3)) => Some(SecondThird     (       line2, line3)), 
                Some(LineSplitResult::First      (line2       )) => Some(Second          (       line2       )),
                Some(LineSplitResult::Second     (       line3)) => Some(Third           (              line3)),
                None => None,
            },
            None => None,
        }
    }
}

impl<Scalar> LineSplit<Scalar> for Line<Scalar> where Scalar: CoordFloat {
    fn line_split(&self, fraction: Scalar) -> Option<LineSplitResult<Self>> {
        if fraction.is_nan() {
            return None
        }
        if fraction <= Scalar::zero() {
            Some(LineSplitResult::Second(self.clone()))
        } else if fraction >= Scalar::one() {
            Some(LineSplitResult::First(self.clone()))
        } else {
            let new_midpoint = self.start + self.delta() * fraction;
            Some(LineSplitResult::FirstSecond(
                Line::new(self.start, new_midpoint),
                Line::new(new_midpoint, self.end),
            ))
        }
    }
}

impl<Scalar> LineSplit<Scalar> for LineString<Scalar>
where
    Scalar: CoordFloat + std::iter::Sum,
{
    fn line_split(&self, fraction: Scalar) -> Option<LineSplitResult<Self>> {
        // import enum variants
        use LineSplitResult::*;
        if fraction.is_nan(){
            return None
        }
        match (fraction <= Scalar::zero(), fraction >= Scalar::one()){
            (false, true) => Some(First(self.clone())),
            (true, false) => Some(Second(self.clone())),
            _ => {
                // find the total length, and at the same time the length of each segment
                // TODO: consider the possibility of a `LineStringMeasured` datatype in the future
                //       as this will be a common requirement in several algorithms, and would be a big
                //       performance boost when repeatedly slicing portions from the same LineStrings
                //       I think I saw a PreparedGeometry PR? maybe that will cover this?
                let LineStringMeasurements{length_total, length_segments} = match measure_line_string(&self) {
                    Some(x) =>x,
                    None=> return None
                };
                if ! Scalar::is_finite(length_total) || Scalar::is_zero(&length_total) {
                    // TODO: Does this cover a linestring with zero or one points?
                    return None
                }

                // Find the length of the first part of the line string before the split;
                let length_fraction = fraction * length_total;
                // Set up some variables to track state in the for-loop
                let mut length_accumulated = Scalar::zero();
                // TODO: unwrap used; but should be safe since we check the length above
                let mut coords_first_part  = vec![*self.0.first().unwrap()];
                let mut coords_second_part = Vec::new();
                
                // Convert window slices to tuples because destructuring slices of unknown length is not
                // possible
                // TODO: the itertools crate has a pairwise function which returns tuples
                let pairs = self.0.as_slice().windows(2).map(|item| (item[0], item[1]));

                for ((a, b), &length_segment) in pairs.zip(length_segments.iter()) {
                    let length_accumulated_before_segment = length_accumulated;
                    length_accumulated                    = length_accumulated + length_segment;
                    let length_accumulated_after_segment  = length_accumulated;
                    if length_accumulated_after_segment < length_fraction {
                        coords_first_part.push(b);
                    } else if length_accumulated_before_segment > length_fraction {
                        coords_second_part.push(b);
                    } else {
                        // TODO: check for divide by zero
                        let fraction_to_split_segment = (length_fraction - length_accumulated_before_segment) / length_segment;
                        match Line::new(a, b).line_split(fraction_to_split_segment) {
                            Some(FirstSecond(line1, _line2)) => {
                                coords_first_part.push(line1.end);
                                coords_second_part.push(line1.end);
                                coords_second_part.push(b);
                            },
                            Some(First      (_line1       )) => {
                                coords_first_part.push(b);
                                coords_second_part.push(b);
                            },
                            Some(Second     (       _line2)) => {
                                coords_second_part.push(a);
                                coords_second_part.push(b);
                            },
                            None => return None
                        }
                    }
                }
                Some(FirstSecond(coords_first_part.into(), coords_second_part.into()))
            }
        }
    }
}

#[cfg(test)]
mod test {

    use geo_types::{line_string, coord};

    use super::*;


    #[test]
    fn test_measure_line_string() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:0.0),
            (x:1.0, y:1.0),
            (x:2.0, y:1.0),
        ];
        let LineStringMeasurements{length_total, length_segments} = measure_line_string(&line_string).unwrap();
        assert_eq!(length_total, 3.0);
        assert_eq!(length_segments, vec![1.0_f32, 1.0_f32, 1.0_f32]);
    }

    #[test]
    fn test_measure_line_string_malformed_zero() {
        let line_string:LineString<f32> = line_string![];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn test_measure_line_string_malformed_one() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn test_measure_line_string_malformed_nan() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:f32::NAN),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn test_measure_line_string_malformed_nan2() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:f32::NAN),
            (x:1.0, y:1.0),
            (x:2.0, y:1.0),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn test_measure_line_string_malformed_inf() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:f32::INFINITY),
        ];
        assert!(measure_line_string(&line_string).is_none());
    }

    #[test]
    fn test_measure_line_split_first_second() {
        // simple x-axis aligned check
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(0.6);
        assert_eq!(result, Some(LineSplitResult::FirstSecond(
            Line::new(
                coord!{x: 0.0_f32, y:0.0_f32},
                coord!{x: 6.0_f32, y:0.0_f32},
            ),
            Line::new(
                coord!{x: 6.0_f32, y:0.0_f32},
                coord!{x:10.0_f32, y:0.0_f32},
            )
        )));

        // simple y-axis aligned check
        let line = Line::new(
            coord!{x:0.0_f32, y: 0.0_f32},
            coord!{x:0.0_f32, y:10.0_f32},
        );
        let result = line.line_split(0.3);
        assert_eq!(result, Some(LineSplitResult::FirstSecond(
            Line::new(
                coord!{x:0.0_f32, y:0.0_f32},
                coord!{x:0.0_f32, y:3.0_f32},
            ),
            Line::new(
                coord!{x:0.0_f32, y:3.0_f32},
                coord!{x:0.0_f32, y:10.0_f32},
            )
        )));

        // non_trivial check
        let line = Line::new(
            coord!{x: 1.0_f32, y:  1.0_f32},
            coord!{x:10.0_f32, y:-10.0_f32},
        );
        let split_point = line.start + line.delta() * 0.7;
        let result = line.line_split(0.7);
        assert_eq!(result, Some(LineSplitResult::FirstSecond(
            Line::new(
                line.start,
                split_point,
            ),
            Line::new(
                split_point,
                line.end,
            )
        )));
    }

    #[test]
    fn test_measure_line_split_first() {
        // test one
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(1.0);
        assert_eq!(result, Some(LineSplitResult::First(line)));

        // Test numbers larger than one
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(2.0);
        assert_eq!(result, Some(LineSplitResult::First(line)));
    }
    #[test]
    fn test_measure_line_split_second() {
        // test zero
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(0.0);
        assert_eq!(result, Some(LineSplitResult::Second(line)));

        // Test negative numbers
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(-2.0);
        assert_eq!(result, Some(LineSplitResult::Second(line)));
    }


    #[test]
    fn test_measure_linestring_split() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:0.0),
            (x:1.0, y:1.0),
            (x:2.0, y:1.0),
        ];
        let mid_line = line_string.lines().nth(1).unwrap();
        let slice_point = mid_line.start + mid_line.delta() * 0.5;
        assert_eq!(
            line_string.line_split(0.5),
            Some(LineSplitResult::FirstSecond(
                LineString::new(vec![line_string.0[0],line_string.0[1], slice_point]),
                LineString::new(vec![slice_point, line_string.0[2],line_string.0[3]])
            ))
        );
    }
    #[test]
    fn test_measure_linestring_split_first() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:0.0),
        ];
        assert_eq!(
            line_string.line_split(1.0),
            Some(LineSplitResult::First(line_string))
        );
    }

    #[test]
    fn test_measure_linestring_split_second() {
        let line_string:LineString<f32> = line_string![
            (x:0.0, y:0.0),
            (x:1.0, y:0.0),
        ];
        assert_eq!(
            line_string.line_split(0.0),
            Some(LineSplitResult::Second(line_string))
        );
    }


}


