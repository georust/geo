use crate::{CoordinateType, LineString};
use core::iter::FromIterator;
use alloc::vec::Vec;

/// A collection of [`LineString`s](line_string/struct.LineString.html).
///
/// Can be created from a `Vec` of `LineString`s, or from an Iterator which yields `LineString`s.
///
/// Iterating over this objects, yields the component `LineString`s.
#[derive(PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiLineString<T>(pub Vec<LineString<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType, ILS: Into<LineString<T>>> From<ILS> for MultiLineString<T> {
    fn from(ls: ILS) -> Self {
        MultiLineString(vec![ls.into()])
    }
}

impl<T: CoordinateType, ILS: Into<LineString<T>>> FromIterator<ILS> for MultiLineString<T> {
    fn from_iter<I: IntoIterator<Item = ILS>>(iter: I) -> Self {
        MultiLineString(iter.into_iter().map(|ls| ls.into()).collect())
    }
}

impl<T: CoordinateType> IntoIterator for MultiLineString<T> {
    type Item = LineString<T>;
    type IntoIter = ::alloc::vec::IntoIter<LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
