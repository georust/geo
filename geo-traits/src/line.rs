use std::marker::PhantomData;

use crate::{
    CoordTrait, Dimensions, GeometryTrait, GeometryType, UnimplementedCoord,
    UnimplementedGeometryCollection, UnimplementedLineString, UnimplementedMultiLineString,
    UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPoint, UnimplementedPolygon,
    UnimplementedRect, UnimplementedTriangle,
};
#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Line};

/// A trait for accessing data from a generic Line.
///
/// A Line is a line segment made up of exactly two [coordinates][CoordTrait].
///
/// Refer to [geo_types::Line] for information about semantics and validity.
pub trait LineTrait: Sized + GeometryTrait {
    /// The type of each underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// Access the start coordinate in this Line
    fn start(&self) -> Self::CoordType<'_>;

    /// Access the start coordinate in this Line
    fn end(&self) -> Self::CoordType<'_>;

    /// Access the two underlying coordinates
    fn coords(&self) -> [Self::CoordType<'_>; 2] {
        [self.start(), self.end()]
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> LineTrait for Line<T> {
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn start(&self) -> Self::CoordType<'_> {
        &self.start
    }

    fn end(&self) -> Self::CoordType<'_> {
        &self.end
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> LineTrait for &'a Line<T> {
    type CoordType<'b>
        = &'a Coord<Self::T>
    where
        Self: 'b;

    fn start(&self) -> Self::CoordType<'_> {
        &self.start
    }

    fn end(&self) -> Self::CoordType<'_> {
        &self.end
    }
}

/// An empty struct that implements [LineTrait].
///
/// This can be used as the `LineType` of the `GeometryTrait` by implementations that don't
/// have a Line concept
#[derive(Clone)]
pub struct UnimplementedLine<T>(PhantomData<T>);

impl<T: Clone> LineTrait for UnimplementedLine<T> {
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn start(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn end(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}

impl<T: Clone> GeometryTrait for UnimplementedLine<T> {
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
        GeometryType::Line(self)
    }
}
