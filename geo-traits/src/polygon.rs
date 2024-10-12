use crate::{Dimension, LineStringTrait, PolygonInteriorIterator};
use geo_types::{CoordNum, LineString, Polygon};

/// A trait for accessing data from a generic Polygon.
pub trait PolygonTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying ring, which implements [LineStringTrait]
    type ItemType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimension;

    /// The exterior ring of the polygon
    fn exterior(&self) -> Option<Self::ItemType<'_>>;

    /// An iterator of the interior rings of this Polygon
    fn interiors(&self) -> impl Iterator<Item = Self::ItemType<'_>> {
        PolygonInteriorIterator::new(self, 0, self.num_interiors())
    }

    /// The number of interior rings in this Polygon
    fn num_interiors(&self) -> usize;

    /// Access to a specified interior ring in this Polygon
    /// Will return None if the provided index is out of bounds
    fn interior(&self, i: usize) -> Option<Self::ItemType<'_>> {
        if i >= self.num_interiors() {
            None
        } else {
            unsafe { Some(self.interior_unchecked(i)) }
        }
    }

    /// Access to a specified interior ring in this Polygon
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn interior_unchecked(&self, i: usize) -> Self::ItemType<'_>;
}

impl<T: CoordNum> PolygonTrait for Polygon<T> {
    type T = T;
    type ItemType<'a> = &'a LineString<Self::T> where Self: 'a;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        // geo-types doesn't really have a way to describe an empty polygon
        Some(Polygon::exterior(self))
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        unsafe { Polygon::interiors(self).get_unchecked(i) }
    }
}

impl<'a, T: CoordNum> PolygonTrait for &'a Polygon<T> {
    type T = T;
    type ItemType<'b> = &'a LineString<Self::T> where
        Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn exterior(&self) -> Option<Self::ItemType<'_>> {
        // geo-types doesn't really have a way to describe an empty polygon
        Some(Polygon::exterior(self))
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        unsafe { Polygon::interiors(self).get_unchecked(i) }
    }
}
