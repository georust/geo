use std::marker::PhantomData;

use crate::geometry::{GeometryTrait, GeometryType};
use crate::iterator::PolygonInteriorIterator;
use crate::line_string::UnimplementedLineString;
use crate::{Dimensions, LineStringTrait};
#[cfg(feature = "geo-types")]
use crate::{
    UnimplementedGeometryCollection, UnimplementedLine, UnimplementedMultiLineString,
    UnimplementedMultiPoint, UnimplementedMultiPolygon, UnimplementedPoint, UnimplementedRect,
    UnimplementedTriangle,
};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, LineString, Polygon};

/// A trait for accessing data from a generic Polygon.
///
/// A `Polygon`’s outer boundary (_exterior ring_) is represented by a
/// [`LineString`][LineStringTrait]. It may contain zero or more holes (_interior rings_), also
/// represented by `LineString`s.
///
/// Refer to [geo_types::Polygon] for information about semantics and validity.
pub trait PolygonTrait: Sized + GeometryTrait {
    /// The coordinate type of this geometry
    /// The type of each underlying ring, which implements [LineStringTrait]
    type RingType<'a>: 'a + LineStringTrait<T = <Self as GeometryTrait>::T>
    where
        Self: 'a;

    /// The exterior ring of the polygon
    fn exterior(&self) -> Option<Self::RingType<'_>>;

    /// An iterator of the interior rings of this Polygon
    fn interiors(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::RingType<'_>> {
        PolygonInteriorIterator::new(self, 0, self.num_interiors())
    }

    /// The number of interior rings in this Polygon
    fn num_interiors(&self) -> usize;

    /// Access to a specified interior ring in this Polygon
    /// Will return None if the provided index is out of bounds
    fn interior(&self, i: usize) -> Option<Self::RingType<'_>> {
        if i >= self.num_interiors() {
            None
        } else {
            unsafe { Some(self.interior_unchecked(i)) }
        }
    }

    /// Access to a specified interior ring in this Polygon
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> PolygonTrait for Polygon<T> {
    type RingType<'a>
        = &'a LineString<<Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        let ext_ring = Polygon::exterior(self);
        if LineStringTrait::num_coords(&ext_ring) == 0 {
            None
        } else {
            Some(ext_ring)
        }
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        unsafe { Polygon::interiors(self).get_unchecked(i) }
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> PolygonTrait for &'a Polygon<T> {
    type RingType<'b>
        = &'a LineString<<Self as GeometryTrait>::T>
    where
        Self: 'b;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        let ext_ring = Polygon::exterior(self);
        if LineStringTrait::num_coords(&ext_ring) == 0 {
            None
        } else {
            Some(ext_ring)
        }
    }

    fn num_interiors(&self) -> usize {
        Polygon::interiors(self).len()
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        unsafe { Polygon::interiors(self).get_unchecked(i) }
    }
}

/// An empty struct that implements [PolygonTrait].
///
/// This can be used as the `PolygonType` of the `GeometryTrait` by implementations that don't have a
/// Polygon concept
#[derive(Clone)]
pub struct UnimplementedPolygon<T: Clone>(PhantomData<T>);

impl<T: Clone> PolygonTrait for UnimplementedPolygon<T> {
    type RingType<'a>
        = UnimplementedLineString<<Self as GeometryTrait>::T>
    where
        Self: 'a;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        unimplemented!()
    }

    fn num_interiors(&self) -> usize {
        unimplemented!()
    }

    unsafe fn interior_unchecked(&self, _i: usize) -> Self::RingType<'_> {
        unimplemented!()
    }
}

impl<T: Clone> GeometryTrait for UnimplementedPolygon<T> {
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
        Dimensions::Xy
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
        GeometryType::Polygon(self)
    }
}
