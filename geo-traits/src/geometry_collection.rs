use std::marker::PhantomData;

use crate::iterator::GeometryCollectionIterator;
use crate::{Dimensions, GeometryTrait, UnimplementedGeometry};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, Geometry, GeometryCollection};

/// A trait for accessing data from a generic GeometryCollection.
///
/// A GeometryCollection is a collection of [Geometry][GeometryTrait] types.
pub trait GeometryCollectionTrait: Sized {
    /// The coordinate type of this geometry
    type T;

    /// The type of each underlying geometry, which implements [GeometryTrait]
    type GeometryType<'a>: 'a + GeometryTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// An iterator over the geometries in this GeometryCollection
    fn geometries(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::GeometryType<'_>> {
        GeometryCollectionIterator::new(self, 0, self.num_geometries())
    }

    /// The number of geometries in this GeometryCollection
    fn num_geometries(&self) -> usize;

    /// Access to a specified geometry in this GeometryCollection
    /// Will return None if the provided index is out of bounds
    fn geometry(&self, i: usize) -> Option<Self::GeometryType<'_>> {
        if i >= self.num_geometries() {
            None
        } else {
            unsafe { Some(self.geometry_unchecked(i)) }
        }
    }

    /// Access to a specified geometry in this GeometryCollection
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> GeometryCollectionTrait for GeometryCollection<T> {
    type T = T;
    type GeometryType<'a>
        = &'a Geometry<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> GeometryCollectionTrait for &'a GeometryCollection<T> {
    type T = T;
    type GeometryType<'b>
        = &'a Geometry<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.0.get_unchecked(i)
    }
}

/// An empty struct that implements [GeometryCollectionTrait].
///
/// This can be used as the `GeometryCollectionType` of the `GeometryTrait` by implementations that
/// don't have a GeometryCollection concept
pub struct UnimplementedGeometryCollection<T>(PhantomData<T>);

impl<T> GeometryCollectionTrait for UnimplementedGeometryCollection<T> {
    type T = T;
    type GeometryType<'a>
        = UnimplementedGeometry<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_geometries(&self) -> usize {
        unimplemented!()
    }

    unsafe fn geometry_unchecked(&self, _i: usize) -> Self::GeometryType<'_> {
        unimplemented!()
    }
}
