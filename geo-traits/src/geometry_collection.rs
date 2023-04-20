use super::GeometryTrait;
use geo_types::{CoordNum, Geometry, GeometryCollection};
use std::iter::Cloned;
use std::slice::Iter;

pub trait GeometryCollectionTrait<'a>: Send + Sync {
    type ItemType: 'a + GeometryTrait<'a>;
    type Iter: Iterator<Item = Self::ItemType>;

    /// An iterator over the geometries in this GeometryCollection
    fn geometries(&'a self) -> Self::Iter;

    /// The number of geometries in this GeometryCollection
    fn num_geometries(&'a self) -> usize;

    /// Access to a specified geometry in this GeometryCollection
    /// Will return None if the provided index is out of bounds
    fn geometry(&'a self, i: usize) -> Option<Self::ItemType>;
}

impl<'a, T: CoordNum + Send + Sync + 'a> GeometryCollectionTrait<'a> for GeometryCollection<T> {
    type ItemType = Geometry<T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn geometries(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_geometries(&'a self) -> usize {
        self.0.len()
    }

    fn geometry(&'a self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}

impl<'a, T: CoordNum + Send + Sync + 'a> GeometryCollectionTrait<'a> for &GeometryCollection<T> {
    type ItemType = Geometry<T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn geometries(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_geometries(&'a self) -> usize {
        self.0.len()
    }

    fn geometry(&'a self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}
