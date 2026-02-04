use crate::{CoordNum, Polygon};

use alloc::vec;
use alloc::vec::Vec;

use core::iter::FromIterator;
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;
#[cfg(feature = "multithreading")]
use rayon::prelude::*;

/// A collection of [`Polygon`s](struct.Polygon.html). Can
/// be created from a `Vec` of `Polygon`s, or from an
/// Iterator which yields `Polygon`s. Iterating over this
/// object yields the component `Polygon`s.
///
/// # Semantics
///
/// The _interior_ and the _boundary_ are the union of the
/// interior and the boundary of the constituent polygons.
///
/// # Validity
///
/// - The interiors of no two constituent polygons may intersect.
///
/// - The boundaries of two (distinct) constituent polygons may only intersect at finitely many points.
///
/// Refer to section 6.1.14 of the OGC-SFA for a formal
/// definition of validity. Note that the validity is not
/// enforced, but expected by the operations and
/// predicates that operate on it.
#[derive(Eq, PartialEq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPolygon<T: CoordNum = f64>(pub Vec<Polygon<T>>);

impl<T: CoordNum, IP: Into<Polygon<T>>> From<IP> for MultiPolygon<T> {
    fn from(x: IP) -> Self {
        Self(vec![x.into()])
    }
}

impl<T: CoordNum, IP: Into<Polygon<T>>> From<Vec<IP>> for MultiPolygon<T> {
    fn from(x: Vec<IP>) -> Self {
        Self(x.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordNum, IP: Into<Polygon<T>>> FromIterator<IP> for MultiPolygon<T> {
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        Self(iter.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordNum> IntoIterator for MultiPolygon<T> {
    type Item = Polygon<T>;
    type IntoIter = ::alloc::vec::IntoIter<Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a MultiPolygon<T> {
    type Item = &'a Polygon<T>;
    type IntoIter = ::alloc::slice::Iter<'a, Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a mut MultiPolygon<T> {
    type Item = &'a mut Polygon<T>;
    type IntoIter = ::alloc::slice::IterMut<'a, Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        (self.0).iter_mut()
    }
}

#[cfg(feature = "multithreading")]
impl<T: CoordNum + Send> IntoParallelIterator for MultiPolygon<T> {
    type Item = Polygon<T>;
    type Iter = rayon::vec::IntoIter<Polygon<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Sync> IntoParallelIterator for &'a MultiPolygon<T> {
    type Item = &'a Polygon<T>;
    type Iter = rayon::slice::Iter<'a, Polygon<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter()
    }
}

#[cfg(feature = "multithreading")]
impl<'a, T: CoordNum + Send + Sync> IntoParallelIterator for &'a mut MultiPolygon<T> {
    type Item = &'a mut Polygon<T>;
    type Iter = rayon::slice::IterMut<'a, Polygon<T>>;

    fn into_par_iter(self) -> Self::Iter {
        self.0.par_iter_mut()
    }
}

impl<T: CoordNum> MultiPolygon<T> {
    /// Returns a MultiPolygon with the given Polygons as elements
    pub fn new(value: Vec<Polygon<T>>) -> Self {
        Self(value)
    }

    /// Returns an empty MultiPolygon
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Polygon<T>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Polygon<T>> {
        self.0.iter_mut()
    }
}

impl<T: CoordNum, I: SliceIndex<[Polygon<T>]>> Index<I> for MultiPolygon<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &I::Output {
        self.0.index(index)
    }
}

impl<T: CoordNum, I: SliceIndex<[Polygon<T>]>> IndexMut<I> for MultiPolygon<T> {
    fn index_mut(&mut self, index: I) -> &mut I::Output {
        self.0.index_mut(index)
    }
}

#[cfg(any(feature = "approx", test))]
mod approx_integration {
    use super::*;
    use approx::{AbsDiffEq, RelativeEq, UlpsEq};

    impl<T> RelativeEq for MultiPolygon<T>
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
        /// use geo_types::{polygon, Polygon, MultiPolygon};
        ///
        /// let a_el: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)];
        /// let a = MultiPolygon::new(vec![a_el]);
        /// let b_el: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)];
        /// let b = MultiPolygon::new(vec![b_el]);
        ///
        /// approx::assert_relative_eq!(a, b, max_relative=0.1);
        /// approx::assert_relative_ne!(a, b, max_relative=0.001);
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

    impl<T> AbsDiffEq for MultiPolygon<T>
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
        /// use geo_types::{polygon, Polygon, MultiPolygon};
        ///
        /// let a_el: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7., y: 9.), (x: 0., y: 0.)];
        /// let a = MultiPolygon::new(vec![a_el]);
        /// let b_el: Polygon<f32> = polygon![(x: 0., y: 0.), (x: 5., y: 0.), (x: 7.01, y: 9.), (x: 0., y: 0.)];
        /// let b = MultiPolygon::new(vec![b_el]);
        ///
        /// approx::abs_diff_eq!(a, b, epsilon=0.1);
        /// approx::abs_diff_ne!(a, b, epsilon=0.001);
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

    impl<T> UlpsEq for MultiPolygon<T>
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
            let mut mp_zipper = self.into_iter().zip(other);
            mp_zipper.all(|(lhs, rhs)| lhs.ulps_eq(rhs, epsilon, max_ulps))
        }
    }
}

#[cfg(any(
    feature = "rstar_0_8",
    feature = "rstar_0_9",
    feature = "rstar_0_10",
    feature = "rstar_0_11",
    feature = "rstar_0_12"
))]
macro_rules! impl_rstar_multi_polygon {
    ($rstar:ident) => {
        impl<T> $rstar::RTreeObject for MultiPolygon<T>
        where
            T: ::num_traits::Float + ::$rstar::RTreeNum,
        {
            type Envelope = ::$rstar::AABB<$crate::Point<T>>;
            fn envelope(&self) -> Self::Envelope {
                use ::$rstar::Envelope;
                self.iter()
                    .map(|p| p.envelope())
                    .fold(::$rstar::AABB::new_empty(), |a, b| a.merged(&b))
            }
        }
    };
}
#[cfg(feature = "rstar_0_8")]
impl_rstar_multi_polygon!(rstar_0_8);
#[cfg(feature = "rstar_0_9")]
impl_rstar_multi_polygon!(rstar_0_9);
#[cfg(feature = "rstar_0_10")]
impl_rstar_multi_polygon!(rstar_0_10);
#[cfg(feature = "rstar_0_11")]
impl_rstar_multi_polygon!(rstar_0_11);
#[cfg(feature = "rstar_0_12")]
impl_rstar_multi_polygon!(rstar_0_12);

#[cfg(test)]
mod test {
    use super::*;
    use crate::{polygon, wkt};

    #[test]
    fn test_iter() {
        let multi = MultiPolygon::new(vec![
            polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }

        // Do it again to prove that `multi` wasn't `moved`.
        first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }
    }

    #[cfg(feature = "multithreading")]
    #[test]
    fn test_par_iter() {
        let multi = MultiPolygon::new(vec![
            polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);
        let mut multimut = MultiPolygon::new(vec![
            polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);
        multi.par_iter().for_each(|_p| ());
        let _ = &multimut.par_iter_mut().for_each(|_p| ());
        let _ = &multi.into_par_iter().for_each(|_p| ());
        let _ = &mut multimut.par_iter_mut().for_each(|_p| ());
    }
    #[test]
    fn test_iter_mut() {
        let mut multi = MultiPolygon::new(vec![
            polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        for poly in &mut multi {
            poly.exterior_mut(|exterior| {
                for coord in exterior {
                    coord.x += 1;
                    coord.y += 1;
                }
            });
        }

        for poly in multi.iter_mut() {
            poly.exterior_mut(|exterior| {
                for coord in exterior {
                    coord.x += 1;
                    coord.y += 1;
                }
            });
        }

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &polygon![(x: 2, y: 2), (x: 4, y: 2), (x: 3, y: 4), (x:2, y:2)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &polygon![(x: 12, y: 12), (x: 14, y: 12), (x: 13, y: 14), (x:12, y:12)]
                );
            }
        }
    }

    #[test]
    fn empty() {
        let empty = MultiPolygon::<f64>::empty();
        let empty_2 = wkt! { MULTIPOLYGON EMPTY };
        assert_eq!(empty, empty_2);
    }

    #[test]
    fn test_indexing() {
        let mut mp = wkt! { MULTIPOLYGON(((0. 0., 1. 0., 1. 1., 0. 0.)), ((2. 2., 3. 2., 3. 3., 2. 2.)), ((4. 4., 5. 4., 5. 5., 4. 4.))) };

        // Index
        assert_eq!(mp[0], wkt! { POLYGON((0. 0., 1. 0., 1. 1., 0. 0.)) });
        assert_eq!(mp[1], wkt! { POLYGON((2. 2., 3. 2., 3. 3., 2. 2.)) });

        // IndexMut
        mp[1] = wkt! { POLYGON((100. 100., 101. 100., 101. 101., 100. 100.)) };
        assert_eq!(
            mp[1],
            wkt! { POLYGON((100. 100., 101. 100., 101. 101., 100. 100.)) }
        );

        // Range
        assert_eq!(
            mp[0..2],
            [
                wkt! { POLYGON((0. 0., 1. 0., 1. 1., 0. 0.)) },
                wkt! { POLYGON((100. 100., 101. 100., 101. 101., 100. 100.)) }
            ]
        );
    }

    #[test]
    #[should_panic]
    fn test_indexing_out_of_bounds() {
        let mp =
            wkt! { MULTIPOLYGON(((0. 0., 1. 0., 1. 1., 0. 0.)), ((2. 2., 3. 2., 3. 3., 2. 2.))) };
        let _ = mp[2];
    }
}
