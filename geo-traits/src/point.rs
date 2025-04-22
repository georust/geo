use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Point};

use crate::{CoordTrait, GeometryTrait, UnimplementedCoord};

/// A trait for accessing data from a generic Point.
///
/// Refer to [geo_types::Point] for information about semantics and validity.
pub trait PointTrait: Sized + GeometryTrait {
    /// The coordinate type of this geometry
    /// The type of the underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    /// The location of this 0-dimensional geometry.
    ///
    /// According to Simple Features, a Point can have zero coordinates and be considered "empty".
    fn coord(&self) -> Option<Self::CoordType<'_>>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> PointTrait for Point<T> {
    type CoordType<'a>
        = &'a Coord<<Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(&self.0)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> PointTrait for &'a Point<T> {
    type CoordType<'b>
        = &'a Coord<<Self as GeometryTrait>::T>
    where
        Self: 'b;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(&self.0)
    }
}

/// An empty struct that implements [PointTrait].
///
/// This can be used as the `PointType` of the `GeometryTrait` by implementations that don't have a
/// Point concept
pub struct UnimplementedPoint<T>(PhantomData<T>);

impl<T> PointTrait for UnimplementedPoint<T> {
    type CoordType<'a>
        = UnimplementedCoord<<Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        unimplemented!()
    }
}
