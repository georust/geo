use std::marker::PhantomData;

use crate::iterator::PolygonInteriorIterator;
use crate::line_string::UnimplementedLineString;
use crate::{Dimensions, LineStringTrait};
#[cfg(feature = "geo-types")]
use geo_types::{CoordNum, LineString, Polygon};

/// A trait for accessing data from a generic Polygon.
///
/// A `Polygon`’s outer boundary (_exterior ring_) is represented by a
/// [`LineString`][LineStringTrait]. It may contain zero or more holes (_interior rings_), also
/// represented by `LineString`s.
///
/// Refer to [geo_types::Polygon] for information about semantics and validity.
pub trait PolygonTrait: Sized {
    /// The coordinate type of this geometry
    type T;

    /// The type of each underlying ring, which implements [LineStringTrait]
    type RingType<'a>: 'a + LineStringTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

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
    type T = T;
    type RingType<'a> = &'a LineString<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

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
    type T = T;
    type RingType<'b> = &'a LineString<Self::T> where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

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
pub struct UnimplementedPolygon<T>(PhantomData<T>);

impl<T> PolygonTrait for UnimplementedPolygon<T> {
    type T = T;
    type RingType<'a> = UnimplementedLineString<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

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
