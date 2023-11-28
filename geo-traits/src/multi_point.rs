use super::point::PointTrait;
use geo_types::{CoordNum, MultiPoint, Point};
use std::slice::Iter;

pub trait MultiPointTrait {
    type T: CoordNum;
    type ItemType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;
    type Iter<'a>: ExactSizeIterator<Item = Self::ItemType<'a>>
    where
        Self: 'a;

    /// An iterator over the points in this MultiPoint
    fn points(&self) -> Self::Iter<'_>;

    /// The number of points in this MultiPoint
    fn num_points(&self) -> usize;

    /// Access to a specified point in this MultiPoint
    /// Will return None if the provided index is out of bounds
    fn point(&self, i: usize) -> Option<Self::ItemType<'_>>;
}

impl<T: CoordNum> MultiPointTrait for MultiPoint<T> {
    type T = T;
    type ItemType<'a> = &'a Point<Self::T> where Self: 'a;
    type Iter<'a> = Iter<'a, Point<Self::T>> where T: 'a;

    fn points(&self) -> Self::Iter<'_> {
        self.0.iter()
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    fn point(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.0.get(i)
    }
}

impl<'a, T: CoordNum> MultiPointTrait for &'a MultiPoint<T> {
    type T = T;
    type ItemType<'b> = &'a Point<Self::T> where Self: 'b;
    type Iter<'b> = Iter<'a, Point<Self::T>> where Self: 'b;

    fn points(&self) -> Self::Iter<'_> {
        self.0.iter()
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    fn point(&self, i: usize) -> Option<Self::ItemType<'_>> {
        self.0.get(i)
    }
}
