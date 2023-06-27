use super::line_string::LineStringTrait;
use geo_types::{CoordNum, LineString, Polygon};
use std::iter::Cloned;
use std::slice::Iter;

pub trait PolygonTrait<'a> {
    type T: CoordNum;
    type ItemType: 'a + LineStringTrait<'a, T = Self::T>;
    type Iter: ExactSizeIterator<Item = Self::ItemType>;

    /// The exterior ring of the polygon
    fn exterior(&self) -> Self::ItemType;

    /// An iterator of the interior rings of this Polygon
    fn interiors(&'a self) -> Self::Iter;

    /// The number of interior rings in this Polygon
    fn num_interiors(&self) -> usize;

    /// Access to a specified interior ring in this Polygon
    /// Will return None if the provided index is out of bounds
    fn interior(&self, i: usize) -> Option<Self::ItemType>;
}

impl<'a, T: CoordNum + 'a> PolygonTrait<'a> for Polygon<T> {
    type T = T;
    type ItemType = LineString<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn exterior(&self) -> Self::ItemType {
        Polygon::exterior(self).clone()
    }

    fn interiors(&'a self) -> Self::Iter {
        Polygon::interiors(self).iter().cloned()
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType> {
        Polygon::interiors(self).get(i).cloned()
    }
}

impl<'a, T: CoordNum + 'a> PolygonTrait<'a> for &Polygon<T> {
    type T = T;
    type ItemType = LineString<Self::T>;
    type Iter = Cloned<Iter<'a, Self::ItemType>>;

    fn exterior(&self) -> Self::ItemType {
        Polygon::exterior(self).clone()
    }

    fn interiors(&'a self) -> Self::Iter {
        Polygon::interiors(self).iter().cloned()
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType> {
        Polygon::interiors(self).get(i).cloned()
    }
}
