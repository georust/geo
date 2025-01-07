use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Rect};

use crate::{CoordTrait, Dimensions, UnimplementedCoord};

/// A trait for accessing data from a generic Rect.
///
/// A Rect is an _axis-aligned_ bounded 2D rectangle whose area is
/// defined by minimum and maximum [`Point`s][CoordTrait].
pub trait RectTrait {
    /// The coordinate type of this geometry
    type T;

    /// The type of each underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// The minimum coordinate of this Rect
    fn min(&self) -> Self::CoordType<'_>;

    /// The maximum coordinate of this Rect
    fn max(&self) -> Self::CoordType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> RectTrait for Rect<T> {
    type T = T;
    type CoordType<'b>
        = Coord<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn min(&self) -> Self::CoordType<'_> {
        Rect::min(*self)
    }

    fn max(&self) -> Self::CoordType<'_> {
        Rect::max(*self)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum + 'a> RectTrait for &'a Rect<T> {
    type T = T;
    type CoordType<'b>
        = Coord<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn min(&self) -> Self::CoordType<'_> {
        Rect::min(**self)
    }

    fn max(&self) -> Self::CoordType<'_> {
        Rect::max(**self)
    }
}

/// An empty struct that implements [RectTrait].
///
/// This can be used as the `RectType` of the `GeometryTrait` by implementations that don't
/// have a Rect concept
pub struct UnimplementedRect<T>(PhantomData<T>);

impl<T> RectTrait for UnimplementedRect<T> {
    type T = T;
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn min(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn max(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}
