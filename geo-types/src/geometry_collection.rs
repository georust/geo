use crate::{CoordNum, Geometry, NoValue};

#[cfg(any(feature = "approx", test))]
use approx::{AbsDiffEq, RelativeEq};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

/// A generic collection of [Geometry] types with 3D space and measurement value support.
///
/// It can be created from a `Vec` of Geometries, or from an Iterator which yields Geometries.
///
/// Looping over this object yields its component **Geometry
/// enum members** (_not_ the underlying geometry
/// primitives), and it supports iteration and indexing as
/// well as the various
/// [`MapCoords`](algorithm/map_coords/index.html)
/// functions, which _are_ directly applied to the
/// underlying geometry primitives.
///
/// # Examples
/// ## Looping
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point! { x: 1.0, y: 1.0 };
/// let pe = Geometry::Point(p);
/// let gc = GeometryCollection::new(vec![pe]);
/// for geom in gc {
///     println!("{:?}", Point::try_from(geom).unwrap().x());
/// }
/// ```
/// ## Implements `iter()`
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point! { x: 1.0, y: 1.0 };
/// let pe = Geometry::Point(p);
/// let gc = GeometryCollection::new(vec![pe]);
/// gc.iter().for_each(|geom| println!("{:?}", geom));
/// ```
///
/// ## Mutable Iteration
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point! { x: 1.0, y: 1.0 };
/// let pe = Geometry::Point(p);
/// let mut gc = GeometryCollection::new(vec![pe]);
/// gc.iter_mut().for_each(|geom| {
///    if let Geometry::Point(p) = geom {
///        p.set_x(0.2);
///    }
/// });
/// let updated = gc[0].clone();
/// assert_eq!(Point::try_from(updated).unwrap().x(), 0.2);
/// ```
///
/// ## Indexing
///
/// ```
/// use std::convert::TryFrom;
/// use geo_types::{Point, point, Geometry, GeometryCollection};
/// let p = point! { x: 1.0, y: 1.0 };
/// let pe = Geometry::Point(p);
/// let gc = GeometryCollection::new(vec![pe]);
/// println!("{:?}", gc[0]);
/// ```
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GeometryCollection<T: CoordNum, Z: CoordNum = NoValue, M: CoordNum = NoValue>(
    pub Vec<Geometry<T, Z, M>>,
);

/// A geometry collection in 2D space + Measure value.
///
/// See [GeometryCollection]
pub type GeometryCollectionM<T> = GeometryCollection<T, NoValue, T>;

/// A geometry collection in 3D space.
///
/// See [GeometryCollection]
pub type GeometryCollection3D<T> = GeometryCollection<T, T, NoValue>;

/// A geometry collection in 3D space + Measure value.
///
/// See [GeometryCollection]
pub type GeometryCollection3DM<T> = GeometryCollection<T, T, T>;

// Implementing Default by hand because T does not have Default restriction
// todo: consider adding Default as a CoordNum requirement
impl<T: CoordNum, Z: CoordNum, M: CoordNum> Default for GeometryCollection<T, Z, M> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> GeometryCollection<T, Z, M> {
    /// Instantiate Self from the raw content value
    pub fn new(value: Vec<Geometry<T, Z, M>>) -> Self {
        Self(value)
    }

    /// DO NOT USE!
    /// This fn will be renamed to `new` in the upcoming version.
    /// This fn is not marked as deprecated because it would require extensive refactoring of the geo code.
    #[inline]
    pub fn new_from(value: Vec<Geometry<T, Z, M>>) -> Self {
        Self::new(value)
    }

    /// Number of geometries in this GeometryCollection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is this GeometryCollection empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Convert any Geometry (or anything that can be converted to a Geometry) into a
/// GeometryCollection
impl<T: CoordNum, Z: CoordNum, M: CoordNum, IG: Into<Geometry<T, Z, M>>> From<IG>
    for GeometryCollection<T, Z, M>
{
    fn from(x: IG) -> Self {
        Self(vec![x.into()])
    }
}

/// Collect Geometries (or what can be converted to a Geometry) into a GeometryCollection
impl<T: CoordNum, Z: CoordNum, M: CoordNum, IG: Into<Geometry<T, Z, M>>> FromIterator<IG>
    for GeometryCollection<T, Z, M>
{
    fn from_iter<I: IntoIterator<Item = IG>>(iter: I) -> Self {
        Self(iter.into_iter().map(|g| g.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> Index<usize> for GeometryCollection<T, Z, M> {
    type Output = Geometry<T, Z, M>;

    fn index(&self, index: usize) -> &Geometry<T, Z, M> {
        self.0.index(index)
    }
}

impl<T: CoordNum, Z: CoordNum, M: CoordNum> IndexMut<usize> for GeometryCollection<T, Z, M> {
    fn index_mut(&mut self, index: usize) -> &mut Geometry<T, Z, M> {
        self.0.index_mut(index)
    }
}

// structure helper for consuming iterator
#[derive(Debug)]
pub struct IntoIteratorHelper<T: CoordNum, Z: CoordNum, M: CoordNum> {
    iter: ::std::vec::IntoIter<Geometry<T, Z, M>>,
}

// implement the IntoIterator trait for a consuming iterator. Iteration will
// consume the GeometryCollection
impl<T: CoordNum, Z: CoordNum, M: CoordNum> IntoIterator for GeometryCollection<T, Z, M> {
    type Item = Geometry<T, Z, M>;
    type IntoIter = IntoIteratorHelper<T, Z, M>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IntoIteratorHelper {
            iter: self.0.into_iter(),
        }
    }
}

// implement Iterator trait for the helper struct, to be used by adapters
impl<T: CoordNum, Z: CoordNum, M: CoordNum> Iterator for IntoIteratorHelper<T, Z, M> {
    type Item = Geometry<T, Z, M>;

    // just return the reference
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// structure helper for non-consuming iterator
#[derive(Debug)]
pub struct IterHelper<'a, T: CoordNum, Z: CoordNum, M: CoordNum> {
    iter: ::std::slice::Iter<'a, Geometry<T, Z, M>>,
}

// implement the IntoIterator trait for a non-consuming iterator. Iteration will
// borrow the GeometryCollection
impl<'a, T: CoordNum, Z: CoordNum, M: CoordNum> IntoIterator for &'a GeometryCollection<T, Z, M> {
    type Item = &'a Geometry<T, Z, M>;
    type IntoIter = IterHelper<'a, T, Z, M>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IterHelper {
            iter: self.0.iter(),
        }
    }
}

// implement the Iterator trait for the helper struct, to be used by adapters
impl<'a, T: CoordNum, Z: 'a + CoordNum, M: 'a + CoordNum> Iterator for IterHelper<'a, T, Z, M> {
    type Item = &'a Geometry<T, Z, M>;

    // just return the str reference
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// structure helper for mutable non-consuming iterator
#[derive(Debug)]
pub struct IterMutHelper<'a, T: CoordNum, Z: CoordNum, M: CoordNum> {
    iter: ::std::slice::IterMut<'a, Geometry<T, Z, M>>,
}

// implement the IntoIterator trait for a mutable non-consuming iterator. Iteration will
// mutably borrow the GeometryCollection
impl<'a, T: CoordNum, Z: CoordNum, M: CoordNum> IntoIterator
    for &'a mut GeometryCollection<T, Z, M>
{
    type Item = &'a mut Geometry<T, Z, M>;
    type IntoIter = IterMutHelper<'a, T, Z, M>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IterMutHelper {
            iter: self.0.iter_mut(),
        }
    }
}

// implement the Iterator trait for the helper struct, to be used by adapters
impl<'a, T: CoordNum, Z: CoordNum, M: CoordNum> Iterator for IterMutHelper<'a, T, Z, M> {
    type Item = &'a mut Geometry<T, Z, M>;

    // just return the str reference
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T: CoordNum, Z: CoordNum, M: CoordNum> GeometryCollection<T, Z, M> {
    pub fn iter(&'a self) -> IterHelper<'a, T, Z, M> {
        self.into_iter()
    }

    pub fn iter_mut(&'a mut self) -> IterMutHelper<'a, T, Z, M> {
        self.into_iter()
    }
}

#[cfg(any(feature = "approx", test))]
impl<T> RelativeEq for GeometryCollection<T>
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
    /// use geo_types::{GeometryCollection, point};
    ///
    /// let a = GeometryCollection::new(vec![point![x: 1.0, y: 2.0].into()]);
    /// let b = GeometryCollection::new(vec![point![x: 1.0, y: 2.01].into()]);
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

#[cfg(any(feature = "approx", test))]
impl<T> AbsDiffEq for GeometryCollection<T>
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
    /// use geo_types::{GeometryCollection, point};
    ///
    /// let a = GeometryCollection::new(vec![point![x: 0.0, y: 0.0].into()]);
    /// let b = GeometryCollection::new(vec![point![x: 0.0, y: 0.1].into()]);
    ///
    /// approx::abs_diff_eq!(a, b, epsilon=0.1);
    /// approx::abs_diff_ne!(a, b, epsilon=0.001);
    /// ```
    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let mut mp_zipper = self.into_iter().zip(other.into_iter());
        mp_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
    }
}
