use crate::{CoordinateType, LineString};
use std::iter::FromIterator;

/// A collection of
/// [`LineString`s](line_string/struct.LineString.html). Can
/// be created from a `Vec` of `LineString`s or from an
/// Iterator which yields `LineString`s. Iterating over this
/// object yields the component `LineString`s.
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

impl<T: CoordinateType> MultiLineString<T> {
    /// True if the MultiLineString is empty or if all of its LineStrings are closed - see
    /// [`LineString::is_closed`].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{MultiLineString, LineString, line_string};
    ///
    /// let open_line_string: LineString<f32> = line_string![(x: 0., y: 0.), (x: 5., y: 0.)];
    /// assert!(!MultiLineString(vec![open_line_string.clone()]).is_closed());
    ///
    /// let closed_line_string: LineString<f32> = line_string![(x: 0., y: 0.), (x: 5., y: 0.), (x: 0., y: 0.)];
    /// assert!(MultiLineString(vec![closed_line_string.clone()]).is_closed());
    ///
    /// // MultiLineString is not closed if *any* of it's LineStrings are not closed
    /// assert!(!MultiLineString(vec![open_line_string, closed_line_string]).is_closed());
    ///
    /// // An empty MultiLineString is closed
    /// assert!(MultiLineString::<f32>(vec![]).is_closed());
    /// ```
    pub fn is_closed(&self) -> bool {
        // Note: Unlike JTS et al, we consider an empty MultiLineString as closed.
        self.iter().all(LineString::is_closed)
    }
}

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

impl<'a, T: CoordinateType> IntoIterator for &'a MultiLineString<T> {
    type Item = &'a LineString<T>;
    type IntoIter = ::std::slice::Iter<'a, LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, T: CoordinateType> IntoIterator for &'a mut MultiLineString<T> {
    type Item = &'a mut LineString<T>;
    type IntoIter = ::std::slice::IterMut<'a, LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).iter_mut()
    }
}

impl<T: CoordinateType> MultiLineString<T> {
    pub fn iter(&self) -> impl Iterator<Item = &LineString<T>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut LineString<T>> {
        self.0.iter_mut()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::line_string;

    #[test]
    fn test_iter() {
        let multi = MultiLineString(vec![
            line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }

        // Do it again to prove that `multi` wasn't `moved`.
        first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiLineString(vec![
            line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        for line_string in &mut multi {
            for coord in line_string {
                coord.x += 1;
                coord.y += 1;
            }
        }

        for line_string in multi.iter_mut() {
            for coord in line_string {
                coord.x += 1;
                coord.y += 1;
            }
        }

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &line_string![(x: 2, y: 2), (x: 4, y: 2), (x: 3, y: 4), (x:2, y:2)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &line_string![(x: 12, y: 12), (x: 14, y: 12), (x: 13, y: 14), (x:12, y:12)]
                );
            }
        }
    }
}
