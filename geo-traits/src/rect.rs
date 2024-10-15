use std::marker::PhantomData;

use geo_types::{Coord, CoordNum, Rect};

use crate::{Dimensions, PointTrait, UnimplementedPoint};

/// A trait for accessing data from a generic Rect.
///
/// A Rect is an _axis-aligned_ bounded 2D rectangle whose area is
/// defined by minimum and maximum [`Point`s][PointTrait].
pub trait RectTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// The minimum coordinate of this Rect
    fn min(&self) -> Self::PointType<'_>;

    /// The maximum coordinate of this Rect
    fn max(&self) -> Self::PointType<'_>;
}

impl<'a, T: CoordNum + 'a> RectTrait for Rect<T> {
    type T = T;
    type PointType<'b> = Coord<T> where Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn min(&self) -> Self::PointType<'_> {
        Rect::min(*self)
    }

    fn max(&self) -> Self::PointType<'_> {
        Rect::max(*self)
    }
}

impl<'a, T: CoordNum + 'a> RectTrait for &'a Rect<T> {
    type T = T;
    type PointType<'b> = Coord<T> where Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn min(&self) -> Self::PointType<'_> {
        Rect::min(**self)
    }

    fn max(&self) -> Self::PointType<'_> {
        Rect::max(**self)
    }
}

/// An empty struct that implements [RectTrait].
///
/// This can be used as the `RectType` of the `GeometryTrait` by implementations that don't
/// have a Rect concept
pub struct UnimplementedRect<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> RectTrait for UnimplementedRect<T> {
    type T = T;
    type PointType<'a> = UnimplementedPoint<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn min(&self) -> Self::PointType<'_> {
        unimplemented!()
    }

    fn max(&self) -> Self::PointType<'_> {
        unimplemented!()
    }
}
