use std::marker::PhantomData;

use crate::iterator::MultiPointIterator;
use crate::{
    Dimensions, GeometryTrait, GeometryType, PointTrait, UnimplementedGeometryCollection,
    UnimplementedLine, UnimplementedLineString, UnimplementedMultiLineString,
    UnimplementedMultiPolygon, UnimplementedPoint, UnimplementedPolygon, UnimplementedRect,
    UnimplementedTriangle,
};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, MultiPoint, Point};

/// A trait for accessing data from a generic MultiPoint.
///
/// A MultiPoint is a collection of [`Point`s][PointTrait].
///
/// Refer to [geo_types::MultiPoint] for information about semantics and validity.
pub trait MultiPointTrait: Sized + GeometryTrait {
    /// The type of each underlying Point, which implements [PointTrait]
    type InnerPointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// An iterator over the points in this MultiPoint
    fn points(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::InnerPointType<'_>> {
        MultiPointIterator::new(self, 0, self.num_points())
    }

    /// The number of points in this MultiPoint
    fn num_points(&self) -> usize;

    /// Access to a specified point in this MultiPoint
    /// Will return None if the provided index is out of bounds
    fn point(&self, i: usize) -> Option<Self::InnerPointType<'_>> {
        if i >= self.num_points() {
            None
        } else {
            unsafe { Some(self.point_unchecked(i)) }
        }
    }

    /// Access to a specified point in this MultiPoint
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn point_unchecked(&self, i: usize) -> Self::InnerPointType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> MultiPointTrait for MultiPoint<T> {
    type InnerPointType<'a>
        = &'a Point<Self::T>
    where
        Self: 'a;

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::InnerPointType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> MultiPointTrait for &'a MultiPoint<T> {
    type InnerPointType<'b>
        = &'a Point<Self::T>
    where
        Self: 'b;

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::InnerPointType<'_> {
        self.0.get_unchecked(i)
    }
}

/// An empty struct that implements [MultiPointTrait].
///
/// This can be used as the `MultiPointType` of the `GeometryTrait` by implementations that don't
/// have a MultiPoint concept
pub struct UnimplementedMultiPoint<T>(PhantomData<T>);

impl<T> MultiPointTrait for UnimplementedMultiPoint<T> {
    type InnerPointType<'a>
        = UnimplementedPoint<Self::T>
    where
        Self: 'a;

    fn num_points(&self) -> usize {
        unimplemented!()
    }

    unsafe fn point_unchecked(&self, _i: usize) -> Self::InnerPointType<'_> {
        unimplemented!()
    }
}

impl<T> GeometryTrait for UnimplementedMultiPoint<T> {
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
        GeometryType::MultiPoint(self)
    }
}
