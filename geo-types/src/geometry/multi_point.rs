use crate::{CoordNum, Point};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};

use alloc::vec;
use alloc::vec::Vec;
use core::iter::FromIterator;
#[cfg(feature = "multithreading")]
use rayon::prelude::*;

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
pub struct MultiPoint<T: CoordNum = f64>(pub Vec<Point<T>>);

impl<T: CoordNum, IP: Into<Point<T>>> From<IP> for MultiPoint<T> {
    /// Convert a single `Point` (or something which can be converted to a
    /// `Point`) into a one-member `MultiPoint`
    fn from(x: IP) -> Self {
        Self(vec![x.into()])
    }
}

impl<T: CoordNum, IP: Into<Point<T>>> From<Vec<IP>> for MultiPoint<T> {
    /// Convert a `Vec` of `Points` (or `Vec` of things which can be converted
    /// to a `Point`) into a `MultiPoint`.
    fn from(v: Vec<IP>) -> Self {
        Self(v.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordNum, IP: Into<Point<T>>> FromIterator<IP> for MultiPoint<T> {
    /// Collect the results of a `Point` iterator into a `MultiPoint`
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        Self(iter.into_iter().map(|p| p.into()).collect())
    }
}

/// Iterate over the `Point`s in this `MultiPoint`.
impl<T: CoordNum> IntoIterator for MultiPoint<T> {
    type Item = Point<T>;
    type IntoIter = ::alloc::vec::IntoIter<Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a MultiPoint<T> {
    type Item = &'a Point<T>;
    type IntoIter = ::alloc::slice::Iter<'a, Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a mut MultiPoint<T> {
    type Item = &'a mut Point<T>;
    type IntoIter = ::alloc::slice::IterMut<'a, Point<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter_mut()
    }
}

#[cfg(feature = "multithreading")]
impl<T: CoordNum + Send> IntoParallelIterator for MultiPoint<T> {
    type Item = Point<T>;
    type Iter = rayon::vec::IntoIter<Point<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Sync> IntoParallelIterator for &'a MultiPoint<T> {
    type Item = &'a Point<T>;
    type Iter = rayon::slice::Iter<'a, Point<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Send + Sync> IntoParallelIterator for &'a mut MultiPoint<T> {
    type Item = &'a mut Point<T>;
    type Iter = rayon::slice::IterMut<'a, Point<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter_mut()
    }
}

impl<T: CoordNum> MultiPoint<T> {
    pub fn new(value: Vec<Point<T>>) -> Self {
        Self(value)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Point<T>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Point<T>> {
        self.0.iter_mut()
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for MultiPoint<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
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
    /// use geo_types::MultiPoint;
    /// use geo_types::point;
    ///
    /// let a = MultiPoint::new(vec![point![x: 0., y: 0.], point![x: 10., y: 10.]]);
    /// let b = MultiPoint::new(vec![point![x: 0., y: 0.], point![x: 10.001, y: 10.]]);
    ///
    /// approx::assert_relative_eq!(a, b, max_relative=0.1)
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

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for MultiPoint<T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
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
    /// use geo_types::MultiPoint;
    /// use geo_types::point;
    ///
    /// let a = MultiPoint::new(vec![point![x: 0., y: 0.], point![x: 10., y: 10.]]);
    /// let b = MultiPoint::new(vec![point![x: 0., y: 0.], point![x: 10.001, y: 10.]]);
    ///
    /// approx::abs_diff_eq!(a, b, epsilon=0.1);
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let mut mp_zipper = self.into_iter().zip(other);
        mp_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{point, wkt};

    #[test]
    fn test_iter() {
        let multi = wkt! { MULTIPOINT(0 0,10 10) };

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
        let mut multi = wkt! { MULTIPOINT(0 0,10 10) };

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

    #[test]
    fn test_relative_eq() {
        let delta = 1e-6;

        let multi = wkt! { MULTIPOINT(0. 0.,10. 10.) };

        let mut multi_x = multi.clone();
        *multi_x.0[0].x_mut() += delta;

        assert!(multi.relative_eq(&multi_x, 1e-2, 1e-2));
        assert!(multi.relative_ne(&multi_x, 1e-12, 1e-12));

        let mut multi_y = multi.clone();
        *multi_y.0[0].y_mut() += delta;
        assert!(multi.relative_eq(&multi_y, 1e-2, 1e-2));
        assert!(multi.relative_ne(&multi_y, 1e-12, 1e-12));

        // Under-sized but otherwise equal.
        let multi_undersized = wkt! { MULTIPOINT(0. 0.) };
        assert!(multi.relative_ne(&multi_undersized, 1., 1.));

        // Over-sized but otherwise equal.
        let multi_oversized = wkt! { MULTIPOINT(0. 0.,10. 10.,10. 100.) };
        assert!(multi.relative_ne(&multi_oversized, 1., 1.));
    }

    #[test]
    fn test_abs_diff_eq() {
        let delta = 1e-6;

        let multi = wkt! { MULTIPOINT(0. 0.,10. 10.) };

        let mut multi_x = multi.clone();
        *multi_x.0[0].x_mut() += delta;
        assert!(multi.abs_diff_eq(&multi_x, 1e-2));
        assert!(multi.abs_diff_ne(&multi_x, 1e-12));

        let mut multi_y = multi.clone();
        *multi_y.0[0].y_mut() += delta;
        assert!(multi.abs_diff_eq(&multi_y, 1e-2));
        assert!(multi.abs_diff_ne(&multi_y, 1e-12));

        // Under-sized but otherwise equal.
        let multi_undersized = wkt! { MULTIPOINT(0. 0.) };
        assert!(multi.abs_diff_ne(&multi_undersized, 1.));

        // Over-sized but otherwise equal.
        let multi_oversized = wkt! { MULTIPOINT(0. 0.,10. 10.,10. 100.) };
        assert!(multi.abs_diff_ne(&multi_oversized, 1.));
    }
}
