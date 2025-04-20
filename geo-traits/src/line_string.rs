use std::marker::PhantomData;

use crate::iterator::LineStringIterator;
use crate::{
    CoordTrait, Dimensions, GeometryTrait, GeometryType, UnimplementedCoord,
    UnimplementedGeometryCollection, UnimplementedLine, UnimplementedMultiLineString,
    UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPoint, UnimplementedPolygon,
    UnimplementedRect, UnimplementedTriangle,
};
#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, LineString};

/// A trait for accessing data from a generic LineString.
///
/// A LineString is an ordered collection of two or more [points][CoordTrait], representing a path
/// between locations.
///
/// Refer to [geo_types::LineString] for information about semantics and validity.
pub trait LineStringTrait: Sized + GeometryTrait {
    /// The type of each underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// An iterator over the coordinates in this LineString
    fn coords(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::CoordType<'_>> {
        LineStringIterator::new(self, 0, self.num_coords())
    }

    /// The number of coordinates in this LineString
    fn num_coords(&self) -> usize;

    /// Access to a specified coordinate in this LineString
    /// Will return None if the provided index is out of bounds
    #[inline]
    fn coord(&self, i: usize) -> Option<Self::CoordType<'_>> {
        if i >= self.num_coords() {
            None
        } else {
            unsafe { Some(self.coord_unchecked(i)) }
        }
    }

    /// Access to a specified coordinate in this LineString
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> LineStringTrait for LineString<T> {
    type CoordType<'a>
        = &'a Coord<Self::T>
    where
        Self: 'a;

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> LineStringTrait for &'a LineString<T> {
    type CoordType<'b>
        = &'a Coord<Self::T>
    where
        Self: 'b;

    fn num_coords(&self) -> usize {
        self.0.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.0.get_unchecked(i)
    }
}

/// An empty struct that implements [LineStringTrait].
///
/// This can be used as the `LineStringType` of the `GeometryTrait` by implementations that don't
/// have a LineString concept
#[derive(Clone)]
pub struct UnimplementedLineString<T: Clone>(PhantomData<T>);

impl<T: Clone> LineStringTrait for UnimplementedLineString<T> {
    type CoordType<'a>
        = UnimplementedCoord<Self::T>
    where
        Self: 'a;

    fn num_coords(&self) -> usize {
        unimplemented!()
    }

    unsafe fn coord_unchecked(&self, _i: usize) -> Self::CoordType<'_> {
        unimplemented!()
    }
}

impl<T: Clone> GeometryTrait for UnimplementedLineString<T> {
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
        GeometryType::LineString(self)
    }
}
