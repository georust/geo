//! Extend CoordTrait traits for the `geo-traits` crate

use geo_traits::{CoordTrait, UnimplementedCoord};
use geo_types::{Coord, CoordNum};

use crate::{CoordTag, GeoTraitExtWithTypeTag};

pub trait CoordTraitExt: CoordTrait + GeoTraitExtWithTypeTag<Tag = CoordTag>
where
    <Self as CoordTrait>::T: CoordNum,
{
    fn geo_coord(&self) -> Coord<Self::T> {
        Coord {
            x: self.x(),
            y: self.y(),
        }
    }
}

impl<T> CoordTraitExt for Coord<T>
where
    T: CoordNum,
{
    fn geo_coord(&self) -> Coord<T> {
        *self
    }
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for Coord<T> {
    type Tag = CoordTag;
}

impl<T> CoordTraitExt for &Coord<T>
where
    T: CoordNum,
{
    fn geo_coord(&self) -> Coord<T> {
        **self
    }
}

impl<T: CoordNum> GeoTraitExtWithTypeTag for &Coord<T> {
    type Tag = CoordTag;
}

impl<T> CoordTraitExt for UnimplementedCoord<T> where T: CoordNum {}

impl<T: CoordNum> GeoTraitExtWithTypeTag for UnimplementedCoord<T> {
    type Tag = CoordTag;
}
