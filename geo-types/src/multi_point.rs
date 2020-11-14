use crate::{CoordinateType, Point};
use std::iter::FromIterator;

/// A collection of [`Point`s](struct.Point.html). Can
/// be created from a `Vec` of `Point`s, or from an
/// Iterator which yields `Point`s. Iterating over this
/// object yields the component `Point`s.
///
/// # Semantics
///
/// The _interior_ and the _boundary_ are the union of the
/// interior and the boundary of the constituent points. In
/// particular, the boundary of a `MultiPoint` is always
/// empty.
///
/// # Examples
///
/// Iterating over a `MultiPoint` yields the `Point`s inside.
///
/// ```
/// use geo_types::{MultiPoint, Point};
/// let points: MultiPoint<_> = vec![(0., 0.), (1., 2.)].into();
/// for point in points {
///     println!("Point x = {}, y = {}", point.x(), point.y());
/// }
/// ```
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPoint<T>(pub Vec<Point<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType, IP: Into<Point<T>>> From<IP> for MultiPoint<T> {
    /// Convert a single `Point` (or something which can be converted to a `Point`) into a
    /// one-member `MultiPoint`
    fn from(x: IP) -> MultiPoint<T> {
        MultiPoint(vec![x.into()])
    }
}

impl<T: CoordinateType, IP: Into<Point<T>>> From<Vec<IP>> for MultiPoint<T> {
    /// Convert a `Vec` of `Points` (or `Vec` of things which can be converted to a `Point`) into a
    /// `MultiPoint`.
    fn from(v: Vec<IP>) -> MultiPoint<T> {
        MultiPoint(v.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordinateType, IP: Into<Point<T>>> FromIterator<IP> for MultiPoint<T> {
    /// Collect the results of a `Point` iterator into a `MultiPoint`
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        MultiPoint(iter.into_iter().map(|p| p.into()).collect())
    }
}

/// Iterate over the `Point`s in this `MultiPoint`.
impl<T: CoordinateType> IntoIterator for MultiPoint<T> {
    type Item = Point<T>;
    type IntoIter = ::std::vec::IntoIter<Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordinateType> IntoIterator for &'a MultiPoint<T> {
    type Item = &'a Point<T>;
    type IntoIter = ::std::slice::Iter<'a, Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, T: CoordinateType> IntoIterator for &'a mut MultiPoint<T> {
    type Item = &'a mut Point<T>;
    type IntoIter = ::std::slice::IterMut<'a, Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).iter_mut()
    }
}

impl<T: CoordinateType> MultiPoint<T> {
    pub fn iter(&self) -> impl Iterator<Item = &Point<T>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Point<T>> {
        self.0.iter_mut()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::point;

    #[test]
    fn test_iter() {
        let multi = MultiPoint(vec![point![x: 0, y: 0], point![x: 10, y: 10]]);

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &point![x: 0, y: 0]);
                first = false;
            } else {
                assert_eq!(p, &point![x: 10, y: 10]);
            }
        }

        // Do it again to prove that `multi` wasn't `moved`.
        first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &point![x: 0, y: 0]);
                first = false;
            } else {
                assert_eq!(p, &point![x: 10, y: 10]);
            }
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiPoint(vec![point![x: 0, y: 0], point![x: 10, y: 10]]);

        for point in &mut multi {
            point.0.x += 1;
            point.0.y += 1;
        }

        for point in multi.iter_mut() {
            point.0.x += 1;
            point.0.y += 1;
        }

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &point![x: 2, y: 2]);
                first = false;
            } else {
                assert_eq!(p, &point![x: 12, y: 12]);
            }
        }
    }
}
