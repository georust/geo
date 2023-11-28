use super::line_string::LineStringTrait;
use geo_types::{CoordNum, LineString, Polygon};
use std::slice::Iter;

pub trait PolygonTrait {
    type T: CoordNum;
    type ItemType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;
    type Iter<'a>: ExactSizeIterator<Item = Self::ItemType<'a>>
    where
        Self: 'a;

    /// The exterior ring of the polygon
    fn exterior(&self) -> Option<Self::ItemType<'_>>;

    /// An iterator of the interior rings of this Polygon
    fn interiors(&self) -> Self::Iter<'_>;

    /// The number of interior rings in this Polygon
    fn num_interiors(&self) -> usize;

    /// Access to a specified interior ring in this Polygon
    /// Will return None if the provided index is out of bounds
    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>>;
}

impl<T: CoordNum> PolygonTrait for Polygon<T> {
    type T = T;
    type ItemType<'a> = &'a LineString<Self::T> where Self: 'a;
    type Iter<'a> = Iter<'a, LineString<Self::T>> where T: 'a;

    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        // geo-types doesn't really have a way to describe an empty polygon
        Some(Polygon::exterior(self))
    }

    fn interiors(&self) -> Self::Iter<'_> {
        Polygon::interiors(self).iter()
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>> {
        Polygon::interiors(self).get(i)
    }
}

impl<'a, T: CoordNum> PolygonTrait for &'a Polygon<T> {
    type T = T;
    type ItemType<'b> = &'a LineString<Self::T> where
        Self: 'b;
    type Iter<'b> = Iter<'a, LineString<Self::T>>  where
        Self: 'b;

    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        Some(Polygon::exterior(self))
    }

    fn interiors(&self) -> Self::Iter<'_> {
        Polygon::interiors(self).iter()
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>> {
        Polygon::interiors(self).get(i)
    }
}
