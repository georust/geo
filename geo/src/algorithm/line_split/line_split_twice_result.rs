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
            Self::Second          (   _   ) => None,
            Self::Third           (      x) => Some(x),
            Self::FirstSecond     (_, _   ) => None,
            Self::SecondThird     (   _, x) => Some(x),
            Self::FirstThird      (_,    x) => Some(x),
            Self::FirstSecondThird(_, _, x) => Some(x),
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
}