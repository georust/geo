use geo_types::{Coord, CoordNum, LineString};

use super::CoordTrait;
use std::slice::Iter;

pub trait LineStringTrait {
    type T: CoordNum;
    type ItemType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;
    type Iter<'a>: ExactSizeIterator<Item = Self::ItemType<'a>>
    where
        Self: 'a;

    /// An iterator over the coords in this LineString
    fn coords(&self) -> Self::Iter<'_>;

    /// The number of coords in this LineString
    fn num_coords(&self) -> usize;

    /// Access to a specified point in this LineString
    /// Will return None if the provided index is out of bounds
    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>>;
}

impl<T: CoordNum> LineStringTrait for LineString<T> {
    type T = T;
    type ItemType<'a> = &'a Coord<Self::T> where Self: 'a;
    type Iter<'a> = Iter<'a, Coord<Self::T>> where T: 'a;

    fn coords(&self) -> Self::Iter<'_> {
        self.0.iter()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.0.get(i)
    }
}

impl<'a, T: CoordNum> LineStringTrait for &'a LineString<T> {
    type T = T;
    type ItemType<'b> = &'a Coord<Self::T> where Self: 'b;
    type Iter<'b> = Iter<'a, Coord<Self::T>> where Self: 'b;

    fn coords(&self) -> Self::Iter<'_> {
        self.0.iter()
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.0.get(i)
    }
}
