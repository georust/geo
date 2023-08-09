use geo_types::CoordFloat;

use super::{LineSplitResult, LineSplitTwiceResult};


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
