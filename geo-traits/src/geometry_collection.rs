use super::GeometryTrait;
use geo_types::{CoordNum, Geometry, GeometryCollection};
use std::iter::Cloned;
use std::slice::Iter;

pub trait GeometryCollectionTrait<'a> {
    type T: CoordNum;
    type ItemType: 'a + GeometryTrait<'a, T = Self::T>;
    type Iter: ExactSizeIterator<Item = Self::ItemType>;

    /// An iterator over the geometries in this GeometryCollection
    fn geometries(&'a self) -> Self::Iter;

    /// The number of geometries in this GeometryCollection
    fn num_geometries(&self) -> usize;

    /// Access to a specified geometry in this GeometryCollection
    /// Will return None if the provided index is out of bounds
    fn geometry(&self, i: usize) -> Option<Self::ItemType>;
}

impl<'a, T: CoordNum + 'a> GeometryCollectionTrait<'a> for GeometryCollection<T> {
    type T = T;
    type ItemType = Geometry<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn geometries(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    fn geometry(&self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}

impl<'a, T: CoordNum + 'a> GeometryCollectionTrait<'a> for &GeometryCollection<T> {
    type T = T;
    type ItemType = Geometry<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn geometries(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_geometries(&self) -> usize {
        self.0.len()
    }

    fn geometry(&self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}
