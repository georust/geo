// Extend MultiPointTrait traits for the `geo-traits` crate

use geo_traits::{GeometryTrait, MultiPointTrait, UnimplementedMultiPoint};
use geo_types::{Coord, CoordNum, MultiPoint};

use crate::{CoordTraitExt, GeoTraitExtWithTypeTag, MultiPointTag, PointTraitExt};

pub trait MultiPointTraitExt:
    MultiPointTrait + GeoTraitExtWithTypeTag<Tag = MultiPointTag>
where
    <Self as GeometryTrait>::T: CoordNum,
{
    type PointTypeExt<'a>: 'a + PointTraitExt<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn point_ext(&self, i: usize) -> Option<Self::PointTypeExt<'_>>;

    /// Returns a point by index without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `i` is a valid index less than the number of points.
    /// Otherwise, this function may cause undefined behavior.
    unsafe fn point_unchecked_ext(&self, i: usize) -> Self::PointTypeExt<'_>;

    /// Returns a coordinate by index without bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `i` is a valid index less than the number of points.
    /// Otherwise, this function may cause undefined behavior.
    unsafe fn geo_coord_unchecked(&self, i: usize) -> Option<Coord<<Self as GeometryTrait>::T>> {
        let point = unsafe { self.point_unchecked_ext(i) };
        point.coord_ext().map(|c| c.geo_coord())
    }

    fn points_ext(&self) -> impl DoubleEndedIterator<Item = Self::PointTypeExt<'_>>;

    fn coord_iter(&self) -> impl DoubleEndedIterator<Item = Coord<<Self as GeometryTrait>::T>> {
        self.points_ext().flat_map(|p| p.geo_coord())
    }
}

#[macro_export]
macro_rules! forward_multi_point_trait_ext_funcs {
    () => {
        type PointTypeExt<'__l_inner>
            = <Self as MultiPointTrait>::InnerPointType<'__l_inner>
        where
            Self: '__l_inner;

        fn point_ext(&self, i: usize) -> Option<Self::PointTypeExt<'_>> {
            <Self as MultiPointTrait>::point(self, i)
        }

        unsafe fn point_unchecked_ext(&self, i: usize) -> Self::PointTypeExt<'_> {
            <Self as MultiPointTrait>::point_unchecked(self, i)
        }

        fn points_ext(&self) -> impl DoubleEndedIterator<Item = Self::PointTypeExt<'_>> {
            <Self as MultiPointTrait>::points(self)
        }
    };
}

impl<T> MultiPointTraitExt for MultiPoint<T>
where
    T: CoordNum,
{
    forward_multi_point_trait_ext_funcs!();

    unsafe fn geo_coord_unchecked(&self, i: usize) -> Option<Coord<T>> {
        Some(self.0.get_unchecked(i).0)
    }

    // Specialized implementation for geo_types::MultiPoint to reduce performance overhead
    fn coord_iter(&self) -> impl DoubleEndedIterator<Item = Coord<<Self as GeometryTrait>::T>> {
        self.0.iter().map(|p| p.0)
    }
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for MultiPoint<T> {
    type Tag = MultiPointTag;
}

impl<T> MultiPointTraitExt for &MultiPoint<T>
where
    T: CoordNum,
{
    forward_multi_point_trait_ext_funcs!();

    unsafe fn geo_coord_unchecked(&self, i: usize) -> Option<Coord<T>> {
        Some(self.0.get_unchecked(i).0)
    }

    // Specialized implementation for geo_types::MultiPoint to reduce performance overhead
    fn coord_iter(&self) -> impl DoubleEndedIterator<Item = Coord<<Self as GeometryTrait>::T>> {
        self.0.iter().map(|p| p.0)
    }
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &MultiPoint<T> {
    type Tag = MultiPointTag;
}

impl<T> MultiPointTraitExt for UnimplementedMultiPoint<T>
where
    T: CoordNum,
{
    forward_multi_point_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for UnimplementedMultiPoint<T> {
    type Tag = MultiPointTag;
}
