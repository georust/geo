use crate::{Dimension, LineStringTrait, MultiLineStringIterator};
use geo_types::{CoordNum, LineString, MultiLineString};

/// A trait for accessing data from a generic MultiLineString.
pub trait MultiLineStringTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying LineString, which implements [LineStringTrait]
    type ItemType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimension;

    /// An iterator over the LineStrings in this MultiLineString
    fn line_strings(&self) -> MultiLineStringIterator<'_, Self::T, Self::ItemType<'_>, Self> {
        MultiLineStringIterator::new(self, 0, self.num_line_strings())
    }

    /// The number of line_strings in this MultiLineString
    fn num_line_strings(&self) -> usize;

    /// Access to a specified line_string in this MultiLineString
    /// Will return None if the provided index is out of bounds
    fn line_string(&self, i: usize) -> Option<Self::ItemType<'_>> {
        if i >= self.num_line_strings() {
            None
        } else {
            unsafe { Some(self.line_string_unchecked(i)) }
        }
    }

    /// Access to a specified line_string in this MultiLineString
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn line_string_unchecked(&self, i: usize) -> Self::ItemType<'_>;
}

impl<T: CoordNum> MultiLineStringTrait for MultiLineString<T> {
    type T = T;
    type ItemType<'a> = &'a LineString<Self::T> where Self: 'a;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}

impl<'a, T: CoordNum> MultiLineStringTrait for &'a MultiLineString<T> {
    type T = T;
    type ItemType<'b> = &'a LineString<Self::T> where Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::ItemType<'_> {
        self.0.get_unchecked(i)
    }
}
