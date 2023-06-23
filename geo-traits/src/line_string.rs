use geo_types::{Coord, CoordNum, LineString};

use super::CoordTrait;
use std::iter::Cloned;
use std::ops::SubAssign;
use std::slice::Iter;

pub trait LineStringTrait<'a>: Send + Sync {
    type T: CoordNum + Send + Sync + SubAssign;
    type ItemType: 'a + CoordTrait<T = Self::T>;
    type Iter: ExactSizeIterator<Item = Self::ItemType>;

    /// An iterator over the coords in this LineString
    fn coords(&'a self) -> Self::Iter;

    /// The number of coords in this LineString
    fn num_coords(&'a self) -> usize;

    /// Access to a specified point in this LineString
    /// Will return None if the provided index is out of bounds
    fn coord(&'a self, i: usize) -> Option<Self::ItemType>;
}

impl<'a, T: CoordNum + Send + Sync + SubAssign + 'a> LineStringTrait<'a> for LineString<T> {
    type T = T;
    type ItemType = Coord<T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn coords(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&'a self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}

impl<'a, T: CoordNum + Send + Sync + SubAssign + 'a> LineStringTrait<'a> for &LineString<T> {
    type T = T;
    type ItemType = Coord<T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn coords(&'a self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&'a self, i: usize) -> Option<Self::ItemType> {
        self.0.get(i).cloned()
    }
}
