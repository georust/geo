use std::marker::PhantomData;

use crate::iterator::LineStringIterator;
use crate::{Dimensions, PointTrait, UnimplementedPoint};
use geo_types::{Coord, CoordNum, LineString};

/// A trait for accessing data from a generic LineString.
///
/// A LineString is an ordered collection of two or more [points][PointTrait], representing a path
/// between locations.
///
/// Refer to [geo_types::LineString] for information about semantics and validity.
pub trait LineStringTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// An iterator over the points in this LineString
    fn points(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::PointType<'_>> {
        LineStringIterator::new(self, 0, self.num_points())
    }

    /// The number of points in this LineString
    fn num_points(&self) -> usize;

    /// Access to a specified point in this LineString
    /// Will return None if the provided index is out of bounds
    #[inline]
    fn point(&self, i: usize) -> Option<Self::PointType<'_>> {
        if i >= self.num_points() {
            None
        } else {
            unsafe { Some(self.point_unchecked(i)) }
        }
    }

    /// Access to a specified point in this LineString
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_>;
}

impl<T: CoordNum> LineStringTrait for LineString<T> {
    type T = T;
    type PointType<'a> = &'a Coord<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.0.get_unchecked(i)
    }
}

impl<'a, T: CoordNum> LineStringTrait for &'a LineString<T> {
    type T = T;
    type PointType<'b> = &'a Coord<Self::T> where Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_points(&self) -> usize {
        self.0.len()
    }

    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_> {
        self.0.get_unchecked(i)
    }
}

/// An empty struct that implements [LineStringTrait].
///
/// This can be used as the `LineStringType` of the `GeometryTrait` by implementations that don't
/// have a LineString concept
pub struct UnimplementedLineString<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> LineStringTrait for UnimplementedLineString<T> {
    type T = T;
    type PointType<'a> = UnimplementedPoint<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_points(&self) -> usize {
        unimplemented!()
    }

    unsafe fn point_unchecked(&self, _i: usize) -> Self::PointType<'_> {
        unimplemented!()
    }
}
