use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Rect};

use crate::{CoordTrait, UnimplementedCoord};

/// A trait for accessing data from a generic Rect.
///
/// A Rect is an _axis-aligned_ bounded 2D rectangle whose area is
/// defined by minimum and maximum [`Point`s][CoordTrait].
pub trait RectTrait: crate::GeometryTrait {
    /// The type of each underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = <Self as crate::GeometryTrait>::T>
    where
        Self: 'a;

    /// The minimum coordinate of this Rect
    fn min(&self) -> Self::CoordType<'_>;

    /// The maximum coordinate of this Rect
    fn max(&self) -> Self::CoordType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> RectTrait for Rect<T> {
    type CoordType<'b>
        = Coord<T>
    where
        Self: 'b;

    fn min(&self) -> Self::CoordType<'_> {
        Rect::min(*self)
    }

    fn max(&self) -> Self::CoordType<'_> {
        Rect::max(*self)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum + 'a> RectTrait for &'a Rect<T> {
    type CoordType<'b>
        = Coord<T>
    where
        Self: 'b;

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
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn min(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn max(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}
