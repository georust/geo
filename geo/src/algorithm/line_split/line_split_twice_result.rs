/// The result of splitting a line twice using
/// [LineSplit::line_split_twice()](crate::algorithm::LineSplit::line_split_twice) method.
/// It can contain between one and three [Line](crate::Line)s / [LineString](crate::LineString)s.
/// 
/// Note that it may not be desireable to use a `match` statement directly on this type if you only
/// ever want one part of the split. For this please see the helper functions;
/// [.first()](LineSplitTwiceResult#method.first),
/// [.second()](LineSplitTwiceResult#method.second),
/// [.third()](LineSplitTwiceResult#method.third),
/// [.into_first()](LineSplitTwiceResult#method.into_first),
/// [.into_second()](LineSplitTwiceResult#method.into_second), and
/// [.into_third()](LineSplitTwiceResult#method.into_third).
/// 
/// ```
/// // get the second part between splits;
/// let mid_part = my_line.line_split_twice(0.2, 0.5).unwrap().into_second().unwrap();
/// ```
/// 
/// To get more than one part, consider using consider using
/// [.into_tuple()](LineSplitTwiceResult#method.into_tuple):
/// 
/// ```
/// match my_line.line_split_twice(0.2, 0.5).unwrap().into_tuple() {
///     (Some(first), Some(second), _) => todo!(),
///     _ => None
/// }
/// ```
#[derive(PartialEq, Debug)]
#[rustfmt::skip]
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

    /// Return only the first of three split line parts, if it exists.
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
    /// Return only the first of three split line parts, if it exists, consuming the result
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
    /// Return only the second  of three split line parts, if it exists
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
    /// Return only the second  of three split line parts, if it exists, consuming the result
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
    /// Return only the third of three split line parts, if it exists
    pub fn third(&self) -> Option<&T> {
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
    /// Return only the third of three split line parts, if it exists, consuming the result
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
    /// Return all three parts of the split line, if they exist
    pub fn as_tuple(&self) -> (Option<&T>, Option<&T>, Option<&T>) {
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
    /// Return all three parts of the split line, if they exist, consuming the result
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
