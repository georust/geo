use std::marker::PhantomData;

use geo_types::{Coord, CoordNum, Point};

use crate::{CoordTrait, Dimensions, UnimplementedCoord};

/// A trait for accessing data from a generic Point.
///
/// Refer to [geo_types::Point] for information about semantics and validity.
pub trait PointTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of the underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// Dimensions of the coordinate tuple
    fn dim(&self) -> Dimensions;

    /// Whether this point is `empty` or not.
    ///
    /// According to Simple Features, a Point can have zero coordinates and be considered `empty`.
    ///
    /// If `is_empty` returns `true`, then the values of `x()`, `y()`, `nth()` and `nth_unchecked`
    /// have no semantic meaning.
    ///
    /// Only a top-level geometry can be empty. That is, when this point is contained within
    /// another geometry, such as a [`LineStringTrait`][crate::LineStringTrait], those points
    /// can never be empty, and a consumer does not need to check this method.
    fn coord(&self) -> Option<Self::CoordType<'_>>;
}

impl<T: CoordNum> PointTrait for Point<T> {
    type T = T;
    type CoordType<'a> = &'a Coord<Self::T> where Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(&self.0)
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }
}

impl<T: CoordNum> PointTrait for &Point<T> {
    type T = T;
    type CoordType<'a> = &'a Coord<Self::T> where Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        Some(&self.0)
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }
}

/// An empty struct that implements [PointTrait].
///
/// This can be used as the `PointType` of the `GeometryTrait` by implementations that don't have a
/// Point concept
pub struct UnimplementedPoint<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> PointTrait for UnimplementedPoint<T> {
    type T = T;
    type CoordType<'a> = UnimplementedCoord<Self::T> where Self: 'a;

    fn coord(&self) -> Option<Self::CoordType<'_>> {
        unimplemented!()
    }

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }
}
