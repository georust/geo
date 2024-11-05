use std::marker::PhantomData;

use crate::iterator::MultiPolygonIterator;
use crate::polygon::UnimplementedPolygon;
use crate::{Dimensions, PolygonTrait};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, MultiPolygon, Polygon};

/// A trait for accessing data from a generic MultiPolygon.
///
/// Refer to [geo_types::MultiPolygon] for information about semantics and validity.
pub trait MultiPolygonTrait: Sized {
    /// The type of each underlying Polygon, which implements [PolygonTrait]
    type PolygonType<'a>: 'a + PolygonTrait
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// An iterator over the Polygons in this MultiPolygon
    fn polygons(
        &self,
    ) -> impl DoubleEndedIterator + ExactSizeIterator<Item = Self::PolygonType<'_>> {
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

#[cfg(feature = "geo-types")]
impl<T: CoordNum> MultiPolygonTrait for MultiPolygon<T> {
    type PolygonType<'a>
        = &'a Polygon<T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn num_polygons(&self) -> usize {
        self.0.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::PolygonType<'_> {
        self.0.get_unchecked(i)
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> MultiPolygonTrait for &'a MultiPolygon<T> {
    type PolygonType<'b>
        = &'a Polygon<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
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
pub struct UnimplementedMultiPolygon<T>(PhantomData<T>);

impl<T> MultiPolygonTrait for UnimplementedMultiPolygon<T> {
    type PolygonType<'a>
        = UnimplementedPolygon<T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn num_polygons(&self) -> usize {
        unimplemented!()
    }

    unsafe fn polygon_unchecked(&self, _i: usize) -> Self::PolygonType<'_> {
        unimplemented!()
    }
}
