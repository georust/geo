use std::marker::PhantomData;

use crate::{
    CoordTrait, Dimensions, GeometryTrait, GeometryType, UnimplementedCoord,
    UnimplementedGeometryCollection, UnimplementedLine, UnimplementedLineString,
    UnimplementedMultiLineString, UnimplementedMultiPoint, UnimplementedMultiPolygon,
    UnimplementedPoint, UnimplementedPolygon, UnimplementedRect,
};
#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Triangle};

/// A trait for accessing data from a generic Triangle.
///
/// A triangle is a bounded area whose three vertices are defined by [coordinates][CoordTrait].
///
/// Refer to [geo_types::Triangle] for information about semantics and validity.
pub trait TriangleTrait: Sized + GeometryTrait {
    /// The type of each underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// Access the first coordinate in this Triangle
    fn first(&self) -> Self::CoordType<'_>;

    /// Access the second coordinate in this Triangle
    fn second(&self) -> Self::CoordType<'_>;

    /// Access the third coordinate in this Triangle
    fn third(&self) -> Self::CoordType<'_>;

    /// Access the three underlying coordinates
    fn coords(&self) -> [Self::CoordType<'_>; 3] {
        [self.first(), self.second(), self.third()]
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> TriangleTrait for Triangle<T> {
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn first(&self) -> Self::CoordType<'_> {
        &self.0
    }

    fn second(&self) -> Self::CoordType<'_> {
        &self.1
    }

    fn third(&self) -> Self::CoordType<'_> {
        &self.2
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> TriangleTrait for &'a Triangle<T> {
    type CoordType<'b>
        = &'a Coord<Self::T>
    where
        Self: 'b;

    fn first(&self) -> Self::CoordType<'_> {
        &self.0
    }

    fn second(&self) -> Self::CoordType<'_> {
        &self.0
    }

    fn third(&self) -> Self::CoordType<'_> {
        &self.0
    }
}

/// An empty struct that implements [TriangleTrait].
///
/// This can be used as the `TriangleType` of the `GeometryTrait` by implementations that don't
/// have a Triangle concept
pub struct UnimplementedTriangle<T>(PhantomData<T>);

impl<T> TriangleTrait for UnimplementedTriangle<T> {
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn first(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn second(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn third(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}

impl<T> GeometryTrait for UnimplementedTriangle<T> {
    type T = T;
    type PointType<'b>
        = UnimplementedPoint<Self::T>
    where
        Self: 'b;
    type LineStringType<'b>
        = UnimplementedLineString<Self::T>
    where
        Self: 'b;
    type PolygonType<'b>
        = UnimplementedPolygon<Self::T>
    where
        Self: 'b;
    type MultiPointType<'b>
        = UnimplementedMultiPoint<Self::T>
    where
        Self: 'b;
    type MultiLineStringType<'b>
        = UnimplementedMultiLineString<Self::T>
    where
        Self: 'b;
    type MultiPolygonType<'b>
        = UnimplementedMultiPolygon<Self::T>
    where
        Self: 'b;
    type GeometryCollectionType<'b>
        = UnimplementedGeometryCollection<Self::T>
    where
        Self: 'b;
    type RectType<'b>
        = UnimplementedRect<Self::T>
    where
        Self: 'b;
    type TriangleType<'b>
        = UnimplementedTriangle<Self::T>
    where
        Self: 'b;
    type LineType<'b>
        = UnimplementedLine<Self::T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn as_type(
        &self,
    ) -> GeometryType<
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
        GeometryType::Triangle(self)
    }
}
