// Extend MultiLineStringTrait traits for the `geo-traits` crate

use geo_traits::{GeometryTrait, MultiLineStringTrait, UnimplementedMultiLineString};
use geo_types::{CoordNum, MultiLineString};

use crate::{GeoTraitExtWithTypeTag, LineStringTraitExt, MultiLineStringTag};

pub trait MultiLineStringTraitExt:
    MultiLineStringTrait + GeoTraitExtWithTypeTag<Tag = MultiLineStringTag>
where
    <Self as GeometryTrait>::T: CoordNum,
{
    type LineStringTypeExt<'a>: 'a + LineStringTraitExt<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn line_string_ext(&self, i: usize) -> Option<Self::LineStringTypeExt<'_>>;

    /// Returns a line string by index without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `i` is a valid index less than the number of line strings.
    /// Otherwise, this function may cause undefined behavior.
    unsafe fn line_string_unchecked_ext(&self, i: usize) -> Self::LineStringTypeExt<'_>;

    fn line_strings_ext(&self) -> impl Iterator<Item = Self::LineStringTypeExt<'_>>;

    /// True if the MultiLineString is empty or if all of its LineStrings are closed
    fn is_closed(&self) -> bool {
        // Note: Unlike JTS et al, we consider an empty MultiLineString as closed.
        self.line_strings_ext().all(|ls| ls.is_closed())
    }
}

#[macro_export]
macro_rules! forward_multi_line_string_trait_ext_funcs {
    () => {
        type LineStringTypeExt<'__l_inner>
            = <Self as MultiLineStringTrait>::InnerLineStringType<'__l_inner>
        where
            Self: '__l_inner;

        fn line_string_ext(&self, i: usize) -> Option<Self::LineStringTypeExt<'_>> {
            <Self as MultiLineStringTrait>::line_string(self, i)
        }

        unsafe fn line_string_unchecked_ext(&self, i: usize) -> Self::LineStringTypeExt<'_> {
            <Self as MultiLineStringTrait>::line_string_unchecked(self, i)
        }

        fn line_strings_ext(&self) -> impl Iterator<Item = Self::LineStringTypeExt<'_>> {
            <Self as MultiLineStringTrait>::line_strings(self)
        }
    };
}

impl<T> MultiLineStringTraitExt for MultiLineString<T>
where
    T: CoordNum,
{
    forward_multi_line_string_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for MultiLineString<T> {
    type Tag = MultiLineStringTag;
}

impl<T> MultiLineStringTraitExt for &MultiLineString<T>
where
    T: CoordNum,
{
    forward_multi_line_string_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &MultiLineString<T> {
    type Tag = MultiLineStringTag;
}

impl<T> MultiLineStringTraitExt for UnimplementedMultiLineString<T>
where
    T: CoordNum,
{
    forward_multi_line_string_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for UnimplementedMultiLineString<T> {
    type Tag = MultiLineStringTag;
}
