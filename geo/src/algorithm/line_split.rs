use crate::{CoordFloat, EuclideanLength, Line, LineString};



#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum LineSplitResult<Result> {
    First            (Result        ),
    Second           (        Result),
    FirstSecond      (Result, Result),
    InvalidLineString,
}

#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum LineSplitTwiceResult<Result> {
    First            (Result                ),
    Second           (        Result        ),
    Third            (                Result),
    FirstSecond      (Result, Result        ),
    SecondThird      (        Result, Result),
    FirstThird       (Result,         Result),
    FirstSecondThird (Result, Result, Result),
    InvalidLineString
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

    fn line_split(&self, fraction: Scalar) -> LineSplitResult<Self>;

    // TODO: I only want to skip formatting the match block, but because attributes on expressions
    //       are experimental we are forced to put it on the function to avoid an error message.
    #[rustfmt::skip]
    fn line_split_twice(
        &self,
        start_fraction: Scalar,
        end_fraction: Scalar,
    ) -> LineSplitTwiceResult<Self> {
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
            LineSplitResult::FirstSecond(line1, line2) => match line2.line_split(second_fraction) {
                LineSplitResult::FirstSecond(line2, line3) => FirstSecondThird(line1, line2, line3),
                LineSplitResult::First      (line2       ) => FirstSecond     (line1, line2       ),
                LineSplitResult::Second     (       line3) => FirstThird      (line1,        line3),
                LineSplitResult::InvalidLineString => InvalidLineString,
            },
            LineSplitResult::First (line1) => First(line1),
            LineSplitResult::Second(line2) => match line2.line_split(second_fraction) {
                LineSplitResult::FirstSecond(line2, line3) => SecondThird     (       line2, line3), 
                LineSplitResult::First      (line2       ) => Second          (       line2       ),
                LineSplitResult::Second     (       line3) => Third           (              line3),
                LineSplitResult::InvalidLineString => InvalidLineString,
            },
            LineSplitResult::InvalidLineString => InvalidLineString,
        }
    }
}

impl<Scalar> LineSplit<Scalar> for Line<Scalar> where Scalar: CoordFloat {
    fn line_split(&self, fraction: Scalar) -> LineSplitResult<Self> {
        if fraction <= Scalar::zero() {
            LineSplitResult::Second(self.clone())
        } else if fraction >= Scalar::one() {
            LineSplitResult::First(self.clone())
        } else {
            let new_midpoint = self.start + self.delta() * fraction;
            LineSplitResult::FirstSecond(
                Line::new(self.start, new_midpoint),
                Line::new(new_midpoint, self.end),
            )
        }
    }
}

impl<Scalar> LineSplit<Scalar> for LineString<Scalar>
where
    Scalar: CoordFloat + std::iter::Sum,
{
    fn line_split(&self, fraction: Scalar) -> LineSplitResult<Self> {
        // import enum variants
        use LineSplitResult::*;
        match (fraction.is_finite(), fraction <= Scalar::zero(), fraction >= Scalar::one()){
            (false, _, _) => InvalidLineString,
            (true, true, false) => First(self.clone()),
            (true, false, true) => Second(self.clone()),
            (true, _, _) => {
                // find the total length, and at the same time the length of each segment
                // TODO: consider the possibility of a `LineStringMeasured` datatype in the future
                //       as this will be a common requirement in several algorithms, and would be a big
                //       performance boost when repeatedly slicing portions from the same LineStrings
                //       I think I saw a PreparedGeometry PR? maybe that will cover this?
                let LineStringMeasurements{length_total, length_segments} = match measure_line_string(&self) {
                    Some(x) =>x,
                    None=> return InvalidLineString
                };
                if ! Scalar::is_finite(length_total) || Scalar::is_zero(&length_total) {
                    // TODO: Does this cover a linestring with zero or one points?
                    return InvalidLineString
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
                            FirstSecond(line1, _line2) => {
                                coords_first_part.push(line1.end);
                                coords_second_part.push(line1.end);
                                coords_second_part.push(b);
                            },
                            First      (_line1       ) => {
                                coords_first_part.push(b);
                                coords_second_part.push(b);
                            },
                            Second     (       _line2) => {
                                coords_second_part.push(a);
                                coords_second_part.push(b);
                            },
                            InvalidLineString => return InvalidLineString
                        }
                    }
                }
                FirstSecond(coords_first_part.into(), coords_second_part.into())
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
        assert_eq!(result, LineSplitResult::FirstSecond(
            Line::new(
                coord!{x: 0.0_f32, y:0.0_f32},
                coord!{x: 6.0_f32, y:0.0_f32},
            ),
            Line::new(
                coord!{x: 6.0_f32, y:0.0_f32},
                coord!{x:10.0_f32, y:0.0_f32},
            )
        ));

        // simple y-axis aligned check
        let line = Line::new(
            coord!{x:0.0_f32, y: 0.0_f32},
            coord!{x:0.0_f32, y:10.0_f32},
        );
        let result = line.line_split(0.3);
        assert_eq!(result, LineSplitResult::FirstSecond(
            Line::new(
                coord!{x:0.0_f32, y:0.0_f32},
                coord!{x:0.0_f32, y:3.0_f32},
            ),
            Line::new(
                coord!{x:0.0_f32, y:3.0_f32},
                coord!{x:0.0_f32, y:10.0_f32},
            )
        ));

        // non_trivial check
        let line = Line::new(
            coord!{x: 1.0_f32, y:  1.0_f32},
            coord!{x:10.0_f32, y:-10.0_f32},
        );
        let split_point = line.start + line.delta() * 0.7;
        let result = line.line_split(0.7);
        assert_eq!(result, LineSplitResult::FirstSecond(
            Line::new(
                line.start,
                split_point,
            ),
            Line::new(
                split_point,
                line.end,
            )
        ));
    }

    #[test]
    fn test_measure_line_split_first() {
        // test one
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(1.0);
        assert_eq!(result, LineSplitResult::First(line));

        // Test numbers larger than one
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(2.0);
        assert_eq!(result, LineSplitResult::First(line));
    }
    #[test]
    fn test_measure_line_split_second() {
        // test zero
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(0.0);
        assert_eq!(result, LineSplitResult::Second(line));

        // Test negative numbers
        let line = Line::new(
            coord!{x: 0.0_f32, y:0.0_f32},
            coord!{x:10.0_f32, y:0.0_f32},
        );
        let result = line.line_split(-2.0);
        assert_eq!(result, LineSplitResult::Second(line));
    }

}


