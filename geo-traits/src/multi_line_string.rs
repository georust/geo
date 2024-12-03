use std::marker::PhantomData;

use crate::iterator::MultiLineStringIterator;
use crate::line_string::UnimplementedLineString;
use crate::{Dimensions, LineStringTrait};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, LineString, MultiLineString};

/// A trait for accessing data from a generic MultiLineString.
///
/// A MultiLineString is a collection of [`LineString`s][LineStringTrait].
///
/// Refer to [geo_types::MultiLineString] for information about semantics and validity.
pub trait MultiLineStringTrait: Sized {
    /// The type of each underlying LineString, which implements [LineStringTrait]
    type LineStringType<'a>: 'a + LineStringTrait
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// An iterator over the LineStrings in this MultiLineString
    fn line_strings(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::LineStringType<'_>> {
        MultiLineStringIterator::new(self, 0, self.num_line_strings())
    }

    /// The number of line_strings in this MultiLineString
    fn num_line_strings(&self) -> usize;

    /// Access to a specified line_string in this MultiLineString
    /// Will return None if the provided index is out of bounds
    fn line_string(&self, i: usize) -> Option<Self::LineStringType<'_>> {
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
    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> MultiLineStringTrait for MultiLineString<T> {
    type LineStringType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> MultiLineStringTrait for &'a MultiLineString<T> {
    type LineStringType<'b>
        = &'a LineString<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_line_strings(&self) -> usize {
        self.0.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::LineStringType<'_> {
        self.0.get_unchecked(i)
    }
}

/// An empty struct that implements [MultiLineStringTrait].
///
/// This can be used as the `MultiLineStringType` of the `GeometryTrait` by implementations that
/// don't have a MultiLineString concept
pub struct UnimplementedMultiLineString<T>(PhantomData<T>);

impl<T> MultiLineStringTrait for UnimplementedMultiLineString<T> {
    type LineStringType<'a>
        = UnimplementedLineString<T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_line_strings(&self) -> usize {
        unimplemented!()
    }

    unsafe fn line_string_unchecked(&self, _i: usize) -> Self::LineStringType<'_> {
        unimplemented!()
    }
}
