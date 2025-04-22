// Extend GeometryCollectionTrait traits for the `geo-traits` crate

use geo_traits::{GeometryCollectionTrait, GeometryTrait, UnimplementedGeometryCollection};
use geo_types::{CoordNum, GeometryCollection};

use crate::{GeoTraitExtWithTypeTag, GeometryCollectionTag, GeometryTraitExt};

pub trait GeometryCollectionTraitExt:
    GeometryCollectionTrait + GeoTraitExtWithTypeTag<Tag = GeometryCollectionTag>
where
    <Self as GeometryTrait>::T: CoordNum,
{
    type GeometryTypeExt<'a>: 'a + GeometryTraitExt<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn geometry_ext(&self, i: usize) -> Option<Self::GeometryTypeExt<'_>>;

    /// Returns a geometry by index without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `i` is a valid index less than the number of geometries.
    /// Otherwise, this function may cause undefined behavior.
    unsafe fn geometry_unchecked_ext(&self, i: usize) -> Self::GeometryTypeExt<'_>;

    fn geometries_ext(&self) -> impl Iterator<Item = Self::GeometryTypeExt<'_>>;
}

#[macro_export]
macro_rules! forward_geometry_collection_trait_ext_funcs {
    () => {
        type GeometryTypeExt<'__gc_inner>
            = <Self as GeometryCollectionTrait>::GeometryType<'__gc_inner>
        where
            Self: '__gc_inner;

        fn geometry_ext(&self, i: usize) -> Option<Self::GeometryTypeExt<'_>> {
            <Self as GeometryCollectionTrait>::geometry(self, i)
        }

        unsafe fn geometry_unchecked_ext(&self, i: usize) -> Self::GeometryTypeExt<'_> {
            unsafe { <Self as GeometryCollectionTrait>::geometry_unchecked(self, i) }
        }

        fn geometries_ext(&self) -> impl Iterator<Item = Self::GeometryTypeExt<'_>> {
            <Self as GeometryCollectionTrait>::geometries(self)
        }
    };
}

impl<T> GeometryCollectionTraitExt for GeometryCollection<T>
where
    T: CoordNum,
{
    forward_geometry_collection_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for GeometryCollection<T> {
    type Tag = GeometryCollectionTag;
}

impl<T> GeometryCollectionTraitExt for &GeometryCollection<T>
where
    T: CoordNum,
{
    forward_geometry_collection_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &GeometryCollection<T> {
    type Tag = GeometryCollectionTag;
}

impl<T> GeometryCollectionTraitExt for UnimplementedGeometryCollection<T>
where
    T: CoordNum,
{
    forward_geometry_collection_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for UnimplementedGeometryCollection<T> {
    type Tag = GeometryCollectionTag;
}
