use crate::{CoordinateType, LineString};
use std::iter::FromIterator;

/// A collection of
/// [`LineString`s](line_string/struct.LineString.html). Can
/// be created from a `Vec` of `LineString`s, or from an
/// Iterator which yields `LineString`s. Iterating over this
/// objects, yields the component `LineString`s.
///
/// # Semantics
///
/// The _boundary_ of a `MultiLineString` is obtained by
/// applying the “mod 2” union rule: A `Point` is in the
/// boundary of a `MultiLineString` if it is in the
/// boundaries of an odd number of elements of the
/// `MultiLineString`.
///
/// The _interior_ of a `MultiLineString` is the union of
/// the interior, and boundary of the constituent
/// `LineString`s, _except_ for the boundary as defined
/// above. In other words, it is the set difference of the
/// boundary from the union of the interior and boundary of
/// the constituents.
///
/// A `MultiLineString` is _simple_ if and only if all of
/// its elements are simple and the only intersections
/// between any two elements occur at `Point`s that are on
/// the boundaries of both elements. A `MultiLineString` is
/// _closed_ if all of its elements are closed. The boundary
/// of a closed `MultiLineString` is always empty.
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
