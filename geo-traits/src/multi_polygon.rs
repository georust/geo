use std::marker::PhantomData;

use crate::iterator::MultiPolygonIterator;
use crate::polygon::UnimplementedPolygon;
use crate::{Dimension, PolygonTrait};
use geo_types::{CoordNum, MultiPolygon, Polygon};

/// A trait for accessing data from a generic MultiPolygon.
///
/// Refer to [geo_types::MultiPolygon] for information about semantics and validity.
pub trait MultiPolygonTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying Polygon, which implements [PolygonTrait]
    type PolygonType<'a>: 'a + PolygonTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimension;

    /// An iterator over the Polygons in this MultiPolygon
    fn polygons(&self) -> impl Iterator<Item = Self::PolygonType<'_>> {
        MultiPolygonIterator::new(self, 0, self.num_polygons())
    }

    /// The number of polygons in this MultiPolygon
    fn num_polygons(&self) -> usize;

    /// Access to a specified polygon in this MultiPolygon
    /// Will return None if the provided index is out of bounds
    fn polygon(&self, i: usize) -> Option<Self::PolygonType<'_>> {
        if i >= self.num_polygons() {
            None
        } else {
            unsafe { Some(self.polygon_unchecked(i)) }
        }
    }

    /// Access to a specified polygon in this MultiPolygon
    ///
    /// # Safety
    ///
    /// Accessing an index out of bounds is UB.
    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_>;
}

impl<T: CoordNum> MultiPolygonTrait for MultiPolygon<T> {
    type T = T;
    type PolygonType<'a> = &'a Polygon<Self::T> where Self: 'a;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.0.get_unchecked(i)
    }
}

impl<'a, T: CoordNum> MultiPolygonTrait for &'a MultiPolygon<T> {
    type T = T;
    type PolygonType<'b> = &'a Polygon<Self::T> where Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.0.get_unchecked(i)
    }
}

/// An empty struct that implements [MultiPolygonTrait].
///
/// This can be used as the `MultiPolygonType` of the `GeometryTrait` by implementations that don't
/// have a MultiPolygon concept
pub struct UnimplementedMultiPolygon<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> MultiPolygonTrait for UnimplementedMultiPolygon<T> {
    type T = T;
    type PolygonType<'a> = UnimplementedPolygon<Self::T> where Self: 'a;

    fn dim(&self) -> Dimension {
        unimplemented!()
    }

    fn num_polygons(&self) -> usize {
        unimplemented!()
    }

    unsafe fn polygon_unchecked(&self, _i: usize) -> Self::PolygonType<'_> {
        unimplemented!()
    }
}
