use std::marker::PhantomData;

use crate::iterator::MultiPointIterator;
use crate::{Dimensions, PointTrait, UnimplementedPoint};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, MultiPoint, Point};

/// A trait for accessing data from a generic MultiPoint.
///
/// A MultiPoint is a collection of [`Point`s][PointTrait].
///
/// Refer to [geo_types::MultiPoint] for information about semantics and validity.
pub trait MultiPointTrait: Sized {
    /// The coordinate type of this geometry
    type T;

    /// The type of each underlying Point, which implements [PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// An iterator over the points in this MultiPoint
    fn points(&self) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::PointType<'_>> {
        MultiPointIterator::new(self, 0, self.num_points())
    }

    /// The number of points in this MultiPoint
    fn num_points(&self) -> usize;

    /// Access to a specified point in this MultiPoint
    /// Will return None if the provided index is out of bounds
    fn point(&self, i: usize) -> Option<Self::PointType<'_>> {
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
    unsafe fn point_unchecked(&self, i: usize) -> Self::PointType<'_>;
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> MultiPointTrait for MultiPoint<T> {
    type T = T;
    type PointType<'a>
        = &'a Point<Self::T>
    where
        Self: 'a;

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

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> MultiPointTrait for &'a MultiPoint<T> {
    type T = T;
    type PointType<'b>
        = &'a Point<Self::T>
    where
        Self: 'b;

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

/// An empty struct that implements [MultiPointTrait].
///
/// This can be used as the `MultiPointType` of the `GeometryTrait` by implementations that don't
/// have a MultiPoint concept
pub struct UnimplementedMultiPoint<T>(PhantomData<T>);

impl<T> MultiPointTrait for UnimplementedMultiPoint<T> {
    type T = T;
    type PointType<'a>
        = UnimplementedPoint<Self::T>
    where
        Self: 'a;

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
