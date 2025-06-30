use crate::{CoordNum, LineString};

use alloc::vec;
use alloc::vec::Vec;
#[cfg(any(feature = "approx", test))]
use core::iter::FromIterator;
#[cfg(feature = "multithreading")]
use rayon::prelude::*;

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
#[derive(Eq, PartialEq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiLineString<T: CoordNum = f64>(pub Vec<LineString<T>>);

impl<T: CoordNum> MultiLineString<T> {
    /// Returns a MultiLineString with the given LineStrings as elements
    pub fn new(value: Vec<LineString<T>>) -> Self {
        Self(value)
    }

    /// Returns an empty MultiLineString
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// True if the MultiLineString is empty or if all of its LineStrings are closed - see
    /// [`LineString::is_closed`].
    ///
    /// # Examples
    ///
    /// ```
    /// use geo_types::{MultiLineString, LineString, line_string};
    ///
    /// let open_line_string: LineString<f32> = line_string![(x: 0., y: 0.), (x: 5., y: 0.)];
    /// assert!(!MultiLineString::new(vec![open_line_string.clone()]).is_closed());
    ///
    /// let closed_line_string: LineString<f32> = line_string![(x: 0., y: 0.), (x: 5., y: 0.), (x: 0., y: 0.)];
    /// assert!(MultiLineString::new(vec![closed_line_string.clone()]).is_closed());
    ///
    /// // MultiLineString is not closed if *any* of it's LineStrings are not closed
    /// assert!(!MultiLineString::new(vec![open_line_string, closed_line_string]).is_closed());
    ///
    /// // An empty MultiLineString is closed
    /// assert!(MultiLineString::<f32>::empty().is_closed());
    /// ```
    pub fn is_closed(&self) -> bool {
        // Note: Unlike JTS et al, we consider an empty MultiLineString as closed.
        self.iter().all(LineString::is_closed)
    }
}

impl<T: CoordNum, ILS: Into<LineString<T>>> From<ILS> for MultiLineString<T> {
    fn from(ls: ILS) -> Self {
        Self(vec![ls.into()])
    }
}

impl<T: CoordNum, ILS: Into<LineString<T>>> FromIterator<ILS> for MultiLineString<T> {
    fn from_iter<I: IntoIterator<Item = ILS>>(iter: I) -> Self {
        Self(iter.into_iter().map(|ls| ls.into()).collect())
    }
}

impl<T: CoordNum> IntoIterator for MultiLineString<T> {
    type Item = LineString<T>;
    type IntoIter = ::alloc::vec::IntoIter<LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a MultiLineString<T> {
    type Item = &'a LineString<T>;
    type IntoIter = ::alloc::slice::Iter<'a, LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a mut MultiLineString<T> {
    type Item = &'a mut LineString<T>;
    type IntoIter = ::alloc::slice::IterMut<'a, LineString<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter_mut()
    }
}

impl<T: CoordNum> MultiLineString<T> {
    pub fn iter(&self) -> impl Iterator<Item = &LineString<T>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut LineString<T>> {
        self.0.iter_mut()
    }
}

#[cfg(feature = "multithreading")]
impl<T: CoordNum + Send> IntoParallelIterator for MultiLineString<T> {
    type Item = LineString<T>;
    type Iter = rayon::vec::IntoIter<LineString<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Sync> IntoParallelIterator for &'a MultiLineString<T> {
    type Item = &'a LineString<T>;
    type Iter = rayon::slice::Iter<'a, LineString<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Send + Sync> IntoParallelIterator for &'a mut MultiLineString<T> {
    type Item = &'a mut LineString<T>;
    type Iter = rayon::slice::IterMut<'a, LineString<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter_mut()
    }
}

#[cfg(any(feature = "approx", test))]
mod approx_integration {
    use super::*;
    use approx::{AbsDiffEq, RelativeEq, UlpsEq};

    impl<T> RelativeEq for MultiLineString<T>
    where
        T: CoordNum + RelativeEq<Epsilon = T>,
    {
        #[inline]
        fn default_max_relative() -> Self::Epsilon {
            T::default_max_relative()
        }

        /// Equality assertion within a relative limit.
        ///
        /// # Examples
        ///
        /// ```
        /// use geo_types::{MultiLineString, line_string};
        ///
        /// let a = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10., y: 10.)]]);
        /// let b = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10.01, y: 10.)]]);
        ///
        /// approx::assert_relative_eq!(a, b, max_relative=0.1);
        /// approx::assert_relative_ne!(a, b, max_relative=0.0001);
        /// ```
        #[inline]
        fn relative_eq(
            &self,
            other: &Self,
            epsilon: Self::Epsilon,
            max_relative: Self::Epsilon,
        ) -> bool {
            if self.0.len() != other.0.len() {
                return false;
            }

            let mut mp_zipper = self.iter().zip(other.iter());
            mp_zipper.all(|(lhs, rhs)| lhs.relative_eq(rhs, epsilon, max_relative))
        }
    }

    impl<T> AbsDiffEq for MultiLineString<T>
    where
        T: CoordNum + AbsDiffEq<Epsilon = T>,
    {
        type Epsilon = T;

        #[inline]
        fn default_epsilon() -> Self::Epsilon {
            T::default_epsilon()
        }

        /// Equality assertion with an absolute limit.
        ///
        /// # Examples
        ///
        /// ```
        /// use geo_types::{MultiLineString, line_string};
        ///
        /// let a = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10., y: 10.)]]);
        /// let b = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10.01, y: 10.)]]);
        ///
        /// approx::abs_diff_eq!(a, b, epsilon=0.1);
        /// approx::abs_diff_ne!(a, b, epsilon=0.001);
        /// ```
        #[inline]
        fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
            if self.0.len() != other.0.len() {
                return false;
            }

            self.into_iter()
                .zip(other)
                .all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
        }
    }

    impl<T> UlpsEq for MultiLineString<T>
    where
        T: CoordNum + UlpsEq<Epsilon = T>,
    {
        fn default_max_ulps() -> u32 {
            T::default_max_ulps()
        }

        fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
            if self.0.len() != other.0.len() {
                return false;
            }
            self.into_iter()
                .zip(other)
                .all(|(lhs, rhs)| lhs.ulps_eq(rhs, epsilon, max_ulps))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{line_string, wkt};

    #[cfg(feature = "multithreading")]
    #[test]
    fn test_multithreading_linestring() {
        let multi: MultiLineString<i32> = wkt! {
            MULTILINESTRING((0 0,2 0,1 2,0 0), (10 10,12 10,11 12,10 10))
        };
        let mut multimut: MultiLineString<i32> = wkt! {
            MULTILINESTRING((0 0,2 0,1 2,0 0), (10 10,12 10,11 12,10 10))
        };
        multi.par_iter().for_each(|_p| ());
        multimut.par_iter_mut().for_each(|_p| ());
        let _ = &multi.into_par_iter().for_each(|_p| ());
        let _ = &mut multimut.par_iter_mut().for_each(|_p| ());
    }

    #[test]
    fn test_iter() {
        let multi: MultiLineString<i32> = wkt! {
            MULTILINESTRING((0 0,2 0,1 2,0 0), (10 10,12 10,11 12,10 10))
        };

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &wkt! { LINESTRING(0 0,2 0,1 2,0 0) });
                first = false;
            } else {
                assert_eq!(p, &wkt! { LINESTRING(10 10,12 10,11 12,10 10) });
            }
        }

        // Do it again to prove that `multi` wasn't `moved`.
        first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &wkt! { LINESTRING(0 0,2 0,1 2,0 0) });
                first = false;
            } else {
                assert_eq!(p, &wkt! { LINESTRING(10 10,12 10,11 12,10 10) });
            }
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiLineString::new(vec![
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

    #[test]
    fn empty() {
        let empty = MultiLineString::<f64>::empty();
        let empty_2 = wkt! { MULTILINESTRING EMPTY };
        assert_eq!(empty, empty_2);
    }
}
