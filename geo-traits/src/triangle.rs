use std::marker::PhantomData;

use crate::{CoordTrait, Dimensions, UnimplementedCoord};
use geo_types::{Coord, CoordNum, Triangle};

/// A trait for accessing data from a generic Triangle.
///
/// A triangle is a bounded area whose three vertices are defined by [coordinates][CoordTrait].
///
/// Refer to [geo_types::Triangle] for information about semantics and validity.
pub trait TriangleTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

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

impl<T: CoordNum> TriangleTrait for Triangle<T> {
    type T = T;
    type CoordType<'a> = &'a Coord<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

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

impl<'a, T: CoordNum> TriangleTrait for &'a Triangle<T> {
    type T = T;
    type CoordType<'b> = &'a Coord<Self::T> where Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

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
pub struct UnimplementedTriangle<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> TriangleTrait for UnimplementedTriangle<T> {
    type T = T;
    type CoordType<'a> = UnimplementedCoord<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

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
