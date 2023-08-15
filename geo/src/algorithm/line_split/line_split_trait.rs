use geo_types::CoordFloat;

use super::{LineSplitResult, LineSplitTwiceResult};

/// Defines functions to split a [Line](crate::Line) or [LineString](crate::LineString)
pub trait LineSplit<Scalar>
where
    Self: Sized,
    Scalar: CoordFloat,
{
    /// Split a [Line](crate::Line) or [LineString](crate::LineString) at some `fraction` of its length.
    ///
    /// `fraction` is any real number. Only values between 0.0 and 1.0 will split the line.
    /// Values outside of this range (including infinite values) will be clamped to 0.0 or 1.0.
    ///
    /// Returns `None` when
    /// - The provided `fraction` is NAN
    /// - The the object being sliced includes NAN or infinite coordinates
    ///
    /// Otherwise returns [`Some(LineSplitResult)`](crate::algorithm::LineSplitResult)
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
    ///
    /// match result {
    ///     Some(LineSplitResult::First(line1))=>{},
    ///     Some(LineSplitResult::Second(line2))=>{},
    ///     Some(LineSplitResult::FirstSecond(line1, line2))=>{},
    ///     None=>{},
    /// }
    /// ```
    fn line_split(&self, fraction: Scalar) -> Option<LineSplitResult<Self>>;

    ///
    ///
    /// example
    ///
    /// ```
    ///
    /// ```
    /// > Note: Currently the default implementation of this function provided by the trait is
    /// > inefficient because it uses repeated application of the
    /// > [.line_split()](LineSplit::line_split) function. In future, types implementing this trait
    /// > should override this with a more efficient algorithm if possible.
    fn line_split_many(&self, fractions: &Vec<Scalar>) -> Option<Vec<Option<Self>>>
    where
        Self: Clone,
    {
        match fractions.len() {
            0 => None,
            1 => self.line_split(fractions[0]).map(|item| {
                let (a, b) = item.into_tuple();
                vec![a, b]
            }),
            _ => {
                let mut fractions: Vec<Scalar> = fractions
                    .iter()
                    .map(|item| item.min(Scalar::one()).max(Scalar::zero()))
                    .collect();
                fractions
                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                fractions.insert(0, Scalar::zero());
                fractions.push(Scalar::one());
                let fractions = fractions; // remove mutability
                let mut output: Vec<Option<Self>> = Vec::new();
                let mut remaining_self = Some(self.clone());
                for fraction in fractions.windows(2) {
                    // cannot be irrefutably unwrapped in for loop *sad crab noises*:
                    let (a, b) = match fraction {
                        &[a, b] => (a, b),
                        _ => return None,
                    };
                    let fraction_interval = b - a;
                    let fraction_to_end = Scalar::one() - a;
                    let next_fraction = fraction_interval / fraction_to_end;
                    remaining_self = if let Some(remaining_self) = remaining_self {
                        match remaining_self.line_split(next_fraction) {
                            Some(LineSplitResult::FirstSecond(line1, line2)) => {
                                output.push(Some(line1));
                                Some(line2)
                            }
                            Some(LineSplitResult::First(line1)) => {
                                output.push(Some(line1));
                                None
                            }
                            Some(LineSplitResult::Second(line2)) => {
                                output.push(None);
                                Some(line2)
                            }
                            None => return None,
                        }
                    } else {
                        output.push(None);
                        None
                    }
                }

                Some(output)
            }
        }
    }

    /// Split a [Line](crate::Line) or [LineString](crate::LineString)
    /// at `fraction_start` and at `fraction_end`.
    /// 
    /// `fraction_start`/`fraction_end` are any real numbers. Only values between 0.0 and 1.0 will
    /// split the line. Values outside of this range (including infinite values) will be clamped to
    /// 0.0 or 1.0.
    /// 
    /// If `fraction_start > fraction_end`, then the values will be swapped prior splitting.
    /// 
    /// Returns [None] when
    /// - Either`fraction_start` or `fraction_end` are NAN
    /// - The the object being sliced includes NAN or infinite coordinates
    ///
    /// Otherwise Returns a [`Some(LineSplitTwiceResult<T>)`](LineSplitTwiceResult)
    /// 
    /// A [`LineSplitTwiceResult<T>`](LineSplitTwiceResult) can contain between one and three
    /// line parts where `T` is either [Line](crate::Line) or [LineString](crate::LineString).
    /// 
    /// Note that [LineSplitTwiceResult] provides various helper methods to get the desired part(s)
    /// of the output.
    /// 
    /// The following example shows how to always obtain the "middle" part between the two splits
    /// using the [`.into_second()`](LineSplitTwiceResult#method.into_second) method:
    /// 
    /// ```
    /// use geo::{LineString, line_string};
    /// use geo::algorithm::{LineSplit, EuclideanLength};
    /// use approx::assert_relative_eq;
    /// let my_road_line_string:LineString<f32> = line_string![
    ///     (x: 0.0,y: 0.0),
    ///     (x:10.0,y: 0.0),
    ///     (x:10.0,y:10.0),
    /// ];
    /// let my_road_len = my_road_line_string.euclidean_length();
    /// let fraction_from =  5.0 / my_road_len;
    /// let fraction_to   = 12.0 / my_road_len;
    /// // Extract the road section between `fraction_from` and `fraction_to` using `.into_second()`
    /// let my_road_section = match my_road_line_string.line_split_twice(fraction_from, fraction_to) {
    ///     Some(result) => match result.into_second() { // get the second part of the result
    ///         Some(linestring)=>Some(linestring),
    ///         _=>None
    ///     },
    ///     _=>None
    /// };
    /// assert_relative_eq!(my_road_section.unwrap(), line_string![
    ///     (x: 5.0,y: 0.0),
    ///     (x:10.0,y: 0.0),
    ///     (x:10.0,y: 2.0),
    /// ]);
    /// ```
    /// 
    #[rustfmt::skip]
    fn line_split_twice(
        &self,
        fraction_start: Scalar,
        fraction_end: Scalar,
    ) -> Option<LineSplitTwiceResult<Self>> {
        // import enum variants
        use LineSplitTwiceResult::*;
        // reject nan fractions
        if fraction_start.is_nan() || fraction_end.is_nan() {
            return None;
        }
        // clamp
        let fraction_start = fraction_start.min(Scalar::one()).max(Scalar::zero());
        let fraction_end = fraction_end.min(Scalar::one()).max(Scalar::zero());

        // swap interval if incorrectly ordered
        let (start_fraction, end_fraction) = if fraction_start > fraction_end {
            (fraction_end, fraction_start)
        } else {
            (fraction_start, fraction_end)
        };

        // find the fraction to split the second portion of the line
        let second_fraction =
            (end_fraction - start_fraction)
            / (Scalar::one() - start_fraction);

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
