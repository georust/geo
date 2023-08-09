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

    pub fn as_tuple(&self) -> (Option<&T>, Option<&T>) {
        match self {
            Self::First      (a   ) => (Some(a), None   ),
            Self::Second     (   b) => (None   , Some(b)),
            Self::FirstSecond(a, b) => (Some(a), Some(b)),
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
