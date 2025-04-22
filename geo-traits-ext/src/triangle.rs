// Extend TriangleTrait traits for the `geo-traits` crate

use geo_traits::to_geo::ToGeoCoord;
use geo_traits::{GeometryTrait, TriangleTrait, UnimplementedTriangle};
use geo_types::{polygon, Coord, CoordNum, Line, Polygon, Triangle};

use crate::{CoordTraitExt, GeoTraitExtWithTypeTag, TriangleTag};

pub trait TriangleTraitExt: TriangleTrait + GeoTraitExtWithTypeTag<Tag = TriangleTag>
where
    <Self as GeometryTrait>::T: CoordNum,
{
    type CoordTypeExt<'a>: 'a + CoordTraitExt<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn first_ext(&self) -> Self::CoordTypeExt<'_>;
    fn second_ext(&self) -> Self::CoordTypeExt<'_>;
    fn third_ext(&self) -> Self::CoordTypeExt<'_>;
    fn coords_ext(&self) -> [Self::CoordTypeExt<'_>; 3];

    fn to_array(&self) -> [Self::CoordType<'_>; 3] {
        [self.first(), self.second(), self.third()]
    }

    fn to_lines(&self) -> [Line<<Self as GeometryTrait>::T>; 3] {
        [
            Line::new(self.first().to_coord(), self.second().to_coord()),
            Line::new(self.second().to_coord(), self.third().to_coord()),
            Line::new(self.third().to_coord(), self.first().to_coord()),
        ]
    }

    fn to_polygon(&self) -> Polygon<<Self as GeometryTrait>::T> {
        polygon![
            self.first().to_coord(),
            self.second().to_coord(),
            self.third().to_coord(),
            self.first().to_coord(),
        ]
    }

    fn coord_iter(&self) -> impl Iterator<Item = Coord<<Self as GeometryTrait>::T>> {
        [self.first(), self.second(), self.third()]
            .into_iter()
            .map(|c| c.to_coord())
    }
}

#[macro_export]
macro_rules! forward_triangle_trait_ext_funcs {
    () => {
        type CoordTypeExt<'__l_inner>
            = <Self as TriangleTrait>::CoordType<'__l_inner>
        where
            Self: '__l_inner;

        fn first_ext(&self) -> Self::CoordTypeExt<'_> {
            <Self as TriangleTrait>::first(self)
        }

        fn second_ext(&self) -> Self::CoordTypeExt<'_> {
            <Self as TriangleTrait>::second(self)
        }

        fn third_ext(&self) -> Self::CoordTypeExt<'_> {
            <Self as TriangleTrait>::third(self)
        }

        fn coords_ext(&self) -> [Self::CoordTypeExt<'_>; 3] {
            [self.first_ext(), self.second_ext(), self.third_ext()]
        }
    };
}

impl<T> TriangleTraitExt for Triangle<T>
where
    T: CoordNum,
{
    forward_triangle_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for Triangle<T> {
    type Tag = TriangleTag;
}

impl<T> TriangleTraitExt for &Triangle<T>
where
    T: CoordNum,
{
    forward_triangle_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &Triangle<T> {
    type Tag = TriangleTag;
}

impl<T> TriangleTraitExt for UnimplementedTriangle<T>
where
    T: CoordNum,
{
    forward_triangle_trait_ext_funcs!();
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for UnimplementedTriangle<T> {
    type Tag = TriangleTag;
}
