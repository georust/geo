// Extend MultiLineStringTrait traits for the `geo-traits` crate

use geo_traits::to_geo::ToGeoCoord;
use geo_traits::{
    GeometryTrait, LineStringTrait, MultiLineStringTrait, UnimplementedMultiLineString,
};
use geo_types::{Coord, CoordNum, MultiLineString};

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

    /// True if the MultiLineString is empty or if all of its LineStrings are closed - see
    /// [`LineString::is_closed`].
    fn is_closed(&self) -> bool {
        // Note: Unlike JTS et al, we consider an empty MultiLineString as closed.
        self.line_strings_ext().all(|ls| ls.is_closed())
    }

    fn coord_iter(&self) -> impl Iterator<Item = Coord<<Self as GeometryTrait>::T>> {
        CoordIter::new(self)
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

struct CoordIter<'a, T: CoordNum, MLS: MultiLineStringTrait<T = T>> {
    multi_line_string: &'a MLS,
    current_line_string: Option<MLS::InnerLineStringType<'a>>,
    idx_line_string: usize,
    idx_coord: usize,
}

impl<'a, T: CoordNum, MLS: MultiLineStringTrait<T = T>> CoordIter<'a, T, MLS> {
    fn new(multi_line_string: &'a MLS) -> Self {
        let current_line_string = multi_line_string.line_string(0);
        Self {
            multi_line_string,
            current_line_string,
            idx_line_string: 0,
            idx_coord: 0,
        }
    }
}

impl<T: CoordNum, MLS: MultiLineStringTrait<T = T>> Iterator for CoordIter<'_, T, MLS> {
    type Item = Coord<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_line_string.as_ref()?;

        let ls = self.current_line_string.as_ref().unwrap();
        if self.idx_coord >= ls.num_coords() {
            // Load the next line string
            while self.idx_line_string < self.multi_line_string.num_line_strings() {
                self.idx_line_string += 1;
                self.idx_coord = 0;
                self.current_line_string = self.multi_line_string.line_string(self.idx_line_string);
                if self.current_line_string.is_some() {
                    break;
                }
            }
            self.next()
        } else {
            // Load the next coordinate
            let mut coord = None;
            while coord.is_none() && self.idx_coord < ls.num_coords() {
                coord = ls.coord(self.idx_coord);
                self.idx_coord += 1;
            }
            coord.map(|c| c.to_coord()).or_else(|| self.next())
        }
    }
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
