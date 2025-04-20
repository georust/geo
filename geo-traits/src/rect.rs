use std::marker::PhantomData;

#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Rect};

use crate::{CoordTrait, Dimensions, UnimplementedCoord};

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

impl<T> crate::GeometryTrait for UnimplementedRect<T> {
    type T = T;
    type PointType<'b>
        = crate::UnimplementedPoint<Self::T>
    where
        Self: 'b;
    type LineStringType<'b>
        = crate::UnimplementedLineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = crate::UnimplementedPolygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = crate::UnimplementedMultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = crate::UnimplementedMultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = crate::UnimplementedMultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = crate::UnimplementedGeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = Self
    where
        Self: 'b;
    type TriangleType<'b>
        = crate::UnimplementedTriangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = crate::UnimplementedLine<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn as_type(
        &self,
    ) -> crate::GeometryType<
        '_,
        Self::PointType<'_>,
        Self::LineStringType<'_>,
        Self::PolygonType<'_>,
        Self::MultiPointType<'_>,
        Self::MultiLineStringType<'_>,
        Self::MultiPolygonType<'_>,
        Self::GeometryCollectionType<'_>,
        Self::RectType<'_>,
        Self::TriangleType<'_>,
        Self::LineType<'_>,
    > {
        unimplemented!()
    }
}
