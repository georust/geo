use super::{LineStringIterator, PointTrait};
use geo_types::{Coord, CoordNum, LineString};

/// A trait for accessing data from a generic LineString.
pub trait LineStringTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [PointTrait]
    type ItemType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The number of dimensions in this geometry
    fn dim(&self) -> usize;

    /// An iterator over the coords in this LineString
    fn coords(&self) -> LineStringIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        LineStringIterator::new(self, 0, self.num_coords())
    }

    /// The number of coords in this LineString
    fn num_coords(&self) -> usize;

    /// Access to a specified point in this LineString
    /// Will return None if the provided index is out of bounds
    #[inline]
    fn coord(&self, i: usize) -> Option<Self::ItemType<'_>> {
        if i >= self.num_coords() {
            None
        } else {
            unsafe { Some(self.coord_unchecked(i)) }
        }
    }

    /// Access to a specified point in this LineString
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn coord_unchecked(&self, i: usize) -> Self::ItemType<'_>;
}

impl<T: CoordNum> LineStringTrait for LineString<T> {
    type T = T;
    type ItemType<'a> = &'a Coord<Self::T> where Self: 'a;

    fn dim(&self) -> usize {
        2
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}

impl<'a, T: CoordNum> LineStringTrait for &'a LineString<T> {
    type T = T;
    type ItemType<'b> = &'a Coord<Self::T> where Self: 'b;

    fn dim(&self) -> usize {
        2
    }

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}
