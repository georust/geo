use std::marker::PhantomData;

use crate::{Dimensions, PointTrait, UnimplementedPoint};
use geo_types::{Coord, CoordNum, Triangle};

/// A trait for accessing data from a generic Triangle.
///
/// A triangle is a bounded area whose three vertices are defined by [points][PointTrait].
///
/// Refer to [geo_types::Triangle] for information about semantics and validity.
pub trait TriangleTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// Access the first point in this Triangle
    fn first(&self) -> Self::PointType<'_>;

    /// Access the second point in this Triangle
    fn second(&self) -> Self::PointType<'_>;

    /// Access the third point in this Triangle
    fn third(&self) -> Self::PointType<'_>;

    /// Access the three underlying points
    fn points(
        &self,
    ) -> (
        Self::PointType<'_>,
        Self::PointType<'_>,
        Self::PointType<'_>,
    ) {
        (self.first(), self.second(), self.third())
    }
}

impl<T: CoordNum> TriangleTrait for Triangle<T> {
    type T = T;
    type PointType<'a> = &'a Coord<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn first(&self) -> Self::PointType<'_> {
        &self.0
    }

    fn second(&self) -> Self::PointType<'_> {
        &self.0
    }

    fn third(&self) -> Self::PointType<'_> {
        &self.0
    }
}

impl<'a, T: CoordNum> TriangleTrait for &'a Triangle<T> {
    type T = T;
    type PointType<'b> = &'a Coord<Self::T> where Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn first(&self) -> Self::PointType<'_> {
        &self.0
    }

    fn second(&self) -> Self::PointType<'_> {
        &self.0
    }

    fn third(&self) -> Self::PointType<'_> {
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
    type PointType<'a> = UnimplementedPoint<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn first(&self) -> Self::PointType<'_> {
        unimplemented!()
    }

    fn second(&self) -> Self::PointType<'_> {
        unimplemented!()
    }

    fn third(&self) -> Self::PointType<'_> {
        unimplemented!()
    }
}
