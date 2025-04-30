// Extend PointTrait traits for the `geo-traits` crate

use geo_traits::{CoordTrait, GeometryTrait, PointTrait, UnimplementedPoint};
use geo_types::{Coord, CoordNum, Point};

use crate::{CoordTraitExt, GeoTraitExtWithTypeTag, PointTag};

pub trait PointTraitExt: PointTrait + GeoTraitExtWithTypeTag<Tag = PointTag>
where
    <Self as GeometryTrait>::T: CoordNum,
{
    type CoordTypeExt<'a>: 'a + CoordTraitExt<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn coord_ext(&self) -> Option<Self::CoordTypeExt<'_>>;

    fn geo_point(&self) -> Option<Point<<Self as GeometryTrait>::T>> {
        self.coord_ext()
            .map(|coord| Point::new(coord.x(), coord.y()))
    }

    fn geo_coord(&self) -> Option<Coord<<Self as GeometryTrait>::T>> {
        self.coord_ext().map(|coord| coord.geo_coord())
    }
}

#[macro_export]
macro_rules! forward_point_trait_ext_funcs {
    () => {
        type CoordTypeExt<'__l_inner>
            = <Self as PointTrait>::CoordType<'__l_inner>
        where
            Self: '__l_inner;

        fn coord_ext(&self) -> Option<Self::CoordTypeExt<'_>> {
            <Self as PointTrait>::coord(self)
        }
    };
}

impl<T> PointTraitExt for Point<T>
where
    T: CoordNum,
{
    forward_point_trait_ext_funcs!();

    fn geo_point(&self) -> Option<Point<T>> {
        Some(*self)
    }

    fn geo_coord(&self) -> Option<Coord<T>> {
        Some(self.0)
    }
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for Point<T> {
    type Tag = PointTag;
}

impl<T> PointTraitExt for &Point<T>
where
    T: CoordNum,
{
    forward_point_trait_ext_funcs!();

    fn geo_point(&self) -> Option<Point<T>> {
        Some(**self)
    }

    fn geo_coord(&self) -> Option<Coord<T>> {
        Some(self.0)
    }
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &Point<T> {
    type Tag = PointTag;
}

impl<T> PointTraitExt for UnimplementedPoint<T>
where
    T: CoordNum,
{
    forward_point_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for UnimplementedPoint<T> {
    type Tag = PointTag;
}
