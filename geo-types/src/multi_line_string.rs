use crate::{CoordinateType, LineString};
use std::iter::FromIterator;

/// A collection of
/// [`LineString`s](line_string/struct.LineString.html). The
/// interior and the boundary are the union of the interior
/// or the boundary of the constituent line strings.
///
/// Can be created from a `Vec` of `LineString`s, or from an
/// Iterator which yields `LineString`s.
///
/// Iterating over this objects, yields the component
/// `LineString`s.
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
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
    type IntoIter = ::std::vec::IntoIter<LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
