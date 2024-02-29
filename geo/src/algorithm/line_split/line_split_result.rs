/// The result of splitting a line using
/// [LineSplit::line_split()](crate::algorithm::LineSplit::line_split) method.
/// It can contain between one and two [Lines](crate::Line) / [LineStrings](crate::LineString).
/// 
/// Note that it may not be desireable to use a `match` statement directly on this type if you only
/// ever want one part of the split. For this please see the helper functions;
/// [.first()](LineSplitResult#method.first),
/// [.second()](LineSplitResult#method.second),
/// [.into_first()](LineSplitResult#method.into_first), and
/// [.into_second()](LineSplitResult#method.into_second).
/// 
/// ```
/// # use geo::{LineString, coord};
/// # use geo::LineSplit;
/// 
/// # let my_line = LineString::from(vec![coord! {x: 0., y: 0.},coord! {x: 1., y: 0.},]);
/// if let Some(result) = my_line.line_split_twice(0.2, 0.5) {
///     if let Some(second) = result.into_second() {
///         // got the 'second' part of the split line
///         // between the points 20% and 50% along its length
///     }
/// }
/// ```
#[rustfmt::skip]
#[derive(PartialEq, Debug)]
pub enum LineSplitResult<T> {
    First       (T   ),
    Second      (   T),
    FirstSecond (T, T),
}

#[rustfmt::skip]
impl<T> LineSplitResult<T>{
    /// Return only the first of two split line parts, if it exists.
    pub fn first(&self) -> Option<&T> {
        match self {
            Self::First      (x   ) => Some(x),
            Self::Second     (   _) => None,
            Self::FirstSecond(x, _) => Some(x),
        }
    }
    /// Return only the first of two split line parts, if it exists, consuming the result.
    pub fn into_first(self) -> Option<T> {
        match self {
            Self::First      (x   ) => Some(x),
            Self::Second     (   _) => None,
            Self::FirstSecond(x, _) => Some(x),
        }
    }
    /// Return only the second of two split line parts, if it exists.
    pub fn second(&self) -> Option<&T> {
        match self {
            Self::First      (_   ) => None,
            Self::Second     (   x) => Some(x),
            Self::FirstSecond(_, x) => Some(x),
        }
    }
    /// Return only the second of two split line parts, if it exists, consuming the result.
    pub fn into_second(self) -> Option<T> {
        match self {
            Self::First      (_   ) => None,
            Self::Second     (   x) => Some(x),
            Self::FirstSecond(_, x) => Some(x),
        }
    }
    /// Return all two parts of the split line, if they exist.
    /// 
    /// Instead of using this, consider using a match statement directly on the
    /// [LineSplitResult] type; the reason is that some combinations of this type
    /// (eg `(None, None)`) can never exist, but the compiler will still complain about missing arms
    /// in your match statement.
    pub fn as_tuple(&self) -> (Option<&T>, Option<&T>) {
        match self {
            Self::First      (a   ) => (Some(a), None   ),
            Self::Second     (   b) => (None   , Some(b)),
            Self::FirstSecond(a, b) => (Some(a), Some(b)),
        }
    }
    /// Return all two parts of the split line, if they exist, consuming the result.
    /// 
    /// Instead of using this, consider using a match statement directly on the
    /// [LineSplitResult] type; the reason is that some combinations of this type
    /// (eg `(None, None)`) can never exist, but the compiler will still complain about missing arms
    /// in your match statement.
    pub fn into_tuple(self) -> (Option<T>, Option<T>) {
        match self {
            Self::First      (a   ) => (Some(a), None   ),
            Self::Second     (   b) => (None   , Some(b)),
            Self::FirstSecond(a, b) => (Some(a), Some(b)),
        }
    }
}
