use geo_types::CoordFloat;

use super::{LineSplitResult, LineSplitTwiceResult};

pub trait LineSplit<Scalar>
where
    Self: Sized,
    Scalar: CoordFloat,
{
    /// Split a line or linestring at some fraction of its length.
    ///
    /// Returns `None` when
    /// - The provided fraction is nan (infinite values are allowed and saturate to 0.0 or 1.0)
    /// - The `Line` or `LineString` include nan or infinite values
    ///
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

    /// This default implementation is inefficient because it uses repeated application of
    /// the line_split function. Implementing types should override this with a more efficient
    /// algorithm if possible.
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
                    if let &[a, b] = fraction {
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
                                Some(LineSplitResult::Second(_)) => {
                                    output.push(None);
                                    None
                                }
                                None => return None,
                            }
                        } else {
                            output.push(None);
                            None
                        }
                    }
                }
                Some(output)
            }
        }
    }

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
