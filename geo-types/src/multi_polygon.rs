use crate::{CoordNum, Polygon};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};
use std::iter::FromIterator;

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
/// - The boundaries of two (distinct) constituent polygons
/// may only intersect at finitely many points.
///
/// Refer to section 6.1.14 of the OGC-SFA for a formal
/// definition of validity. Note that the validity is not
/// enforced, but expected by the operations and
/// predicates that operate on it.
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPolygon<T: CoordNum>(
    #[deprecated(
        since = "0.7.5",
        note = "Direct field access is deprecated - use `multi_poly.polygons()` or `multi_poly.polygons_mut()` for field access and `MultiPolygon::new(polygons_vec)` for construction"
    )]
    pub Vec<Polygon<T>>,
);

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
    type IntoIter = ::std::vec::IntoIter<Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a MultiPolygon<T> {
    type Item = &'a Polygon<T>;
    type IntoIter = ::std::slice::Iter<'a, Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.polygons().iter()
    }
}

impl<'a, T: CoordNum> IntoIterator for &'a mut MultiPolygon<T> {
    type Item = &'a mut Polygon<T>;
    type IntoIter = ::std::slice::IterMut<'a, Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.polygons_mut().iter_mut()
    }
}

impl<T: CoordNum> MultiPolygon<T> {
    /// Create a `MultiPolygon` with `polygons` as its members.
    #[inline]
    pub fn new(polygons: Vec<Polygon<T>>) -> Self {
        Self(polygons)
    }

    /// Get this collection's constituent [`Polygon`]s
    #[inline]
    pub fn polygons(&self) -> &[Polygon<T>] {
        #[allow(deprecated)]
        &self.0
    }

    /// Mutably borrow this collection's constituent [`Polygon`]s.
    #[inline]
    pub fn polygons_mut(&mut self) -> &mut [Polygon<T>] {
        #[allow(deprecated)]
        &mut self.0
    }

    /// Push a `polygon` onto the end of this collection.
    #[inline]
    pub fn push(&mut self, polygon: Polygon<T>) {
        #[allow(deprecated)]
        self.0.push(polygon)
    }

    /// Consume this [`MultiPolygon`] to get ownership of its constituent [`Polygon`]s.
    #[inline]
    pub fn into_inner(self) -> Vec<Polygon<T>> {
        #[allow(deprecated)]
        self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &Polygon<T>> {
        self.polygons().iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Polygon<T>> {
        self.polygons_mut().iter_mut()
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for MultiPolygon<T>
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
        if self.polygons().len() != other.polygons().len() {
            return false;
        }

        let mut mp_zipper = self.iter().zip(other.iter());
        mp_zipper.all(|(lhs, rhs)| lhs.relative_eq(rhs, epsilon, max_relative))
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for MultiPolygon<T>
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
        if self.polygons().len() != other.polygons().len() {
            return false;
        }

        let mut mp_zipper = self.into_iter().zip(other.into_iter());
        mp_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::polygon;

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

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiPolygon::new(vec![
            polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        for poly in &mut multi {
            poly.exterior_mut(|exterior| {
                for coord in exterior {
                    *coord.x_mut() += 1;
                    *coord.y_mut() += 1;
                }
            });
        }

        for poly in multi.iter_mut() {
            poly.exterior_mut(|exterior| {
                for coord in exterior {
                    *coord.x_mut() += 1;
                    *coord.y_mut() += 1;
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
}
