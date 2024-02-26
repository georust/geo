use super::{GeometryCollectionIterator, GeometryTrait};
use geo_types::{CoordNum, Geometry, GeometryCollection};

/// A trait for accessing data from a generic GeometryCollection.
pub trait GeometryCollectionTrait: Sized {
    type T: CoordNum;
    type ItemType<'a>: 'a + GeometryTrait<T = Self::T>
    where
        Self: 'a;

    /// An iterator over the geometries in this GeometryCollection
    fn geometries(&self) -> GeometryCollectionIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        GeometryCollectionIterator::new(self, 0, self.num_geometries())
    }

    /// The number of geometries in this GeometryCollection
    fn num_geometries(&self) -> usize;

    /// Access to a specified geometry in this GeometryCollection
    /// Will return None if the provided index is out of bounds
    fn geometry(&self, i: usize) -> Option<Self::ItemType<'_>> {
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
    unsafe fn geometry_unchecked(&self, i: usize) -> Self::ItemType<'_>;
}

impl<T: CoordNum> GeometryCollectionTrait for GeometryCollection<T> {
    type T = T;
    type ItemType<'a> = &'a Geometry<Self::T>
    where
        Self: 'a;

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}

impl<'a, T: CoordNum> GeometryCollectionTrait for &'a GeometryCollection<T> {
    type T = T;
    type ItemType<'b> = &'a Geometry<Self::T> where
        Self: 'b;

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}
