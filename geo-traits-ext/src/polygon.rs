// Extend PolygonTrait traits for the `geo-traits` crate

use geo_traits::{GeometryTrait, PolygonTrait, UnimplementedPolygon};
use geo_types::{CoordNum, Polygon};

use crate::{GeoTraitExtWithTypeTag, LineStringTraitExt, PolygonTag};

pub trait PolygonTraitExt: PolygonTrait + GeoTraitExtWithTypeTag<Tag = PolygonTag>
where
    <Self as GeometryTrait>::T: CoordNum,
{
    type RingTypeExt<'a>: 'a + LineStringTraitExt<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn exterior_ext(&self) -> Option<Self::RingTypeExt<'_>>;
    fn interiors_ext(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::RingTypeExt<'_>>;
    fn interior_ext(&self, i: usize) -> Option<Self::RingTypeExt<'_>>;

    /// Returns an interior ring by index without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `i` is a valid index less than the number of interior rings.
    /// Otherwise, this function may cause undefined behavior.
    unsafe fn interior_unchecked_ext(&self, i: usize) -> Self::RingTypeExt<'_>;
}

#[macro_export]
macro_rules! forward_polygon_trait_ext_funcs {
    () => {
        type RingTypeExt<'__l_inner>
            = <Self as PolygonTrait>::RingType<'__l_inner>
        where
            Self: '__l_inner;

        fn exterior_ext(&self) -> Option<Self::RingTypeExt<'_>> {
            <Self as PolygonTrait>::exterior(self)
        }

        fn interiors_ext(
            &self,
        ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::RingTypeExt<'_>> {
            <Self as PolygonTrait>::interiors(self)
        }

        fn interior_ext(&self, i: usize) -> Option<Self::RingTypeExt<'_>> {
            <Self as PolygonTrait>::interior(self, i)
        }

        unsafe fn interior_unchecked_ext(&self, i: usize) -> Self::RingTypeExt<'_> {
            <Self as PolygonTrait>::interior_unchecked(self, i)
        }
    };
}

impl<T> PolygonTraitExt for Polygon<T>
where
    T: CoordNum,
{
    forward_polygon_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for Polygon<T> {
    type Tag = PolygonTag;
}

impl<T> PolygonTraitExt for &Polygon<T>
where
    T: CoordNum,
{
    forward_polygon_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &Polygon<T> {
    type Tag = PolygonTag;
}

impl<T> PolygonTraitExt for UnimplementedPolygon<T>
where
    T: CoordNum,
{
    forward_polygon_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for UnimplementedPolygon<T> {
    type Tag = PolygonTag;
}
