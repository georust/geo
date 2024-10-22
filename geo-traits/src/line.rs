use std::marker::PhantomData;

use crate::{Dimensions, PointTrait, UnimplementedPoint};
use geo_types::{Coord, CoordNum, Line};

/// A trait for accessing data from a generic Line.
///
/// A Line is a line segment made up of exactly two [points][PointTrait].
///
/// Refer to [geo_types::Line] for information about semantics and validity.
pub trait LineTrait: Sized {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [PointTrait]
    type PointType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// Access the start point in this Line
    fn start(&self) -> Self::PointType<'_>;

    /// Access the start point in this Line
    fn end(&self) -> Self::PointType<'_>;

    /// Access the two underlying points
    fn points(&self) -> [Self::PointType<'_>; 2] {
        [self.start(), self.end()]
    }
}

impl<T: CoordNum> LineTrait for Line<T> {
    type T = T;
    type PointType<'a> = &'a Coord<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn start(&self) -> Self::PointType<'_> {
        &self.start
    }

    fn end(&self) -> Self::PointType<'_> {
        &self.end
    }
}

impl<'a, T: CoordNum> LineTrait for &'a Line<T> {
    type T = T;
    type PointType<'b> = &'a Coord<Self::T> where Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn start(&self) -> Self::PointType<'_> {
        &self.start
    }

    fn end(&self) -> Self::PointType<'_> {
        &self.end
    }
}

/// An empty struct that implements [LineTrait].
///
/// This can be used as the `LineType` of the `GeometryTrait` by implementations that don't
/// have a Line concept
pub struct UnimplementedLine<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> LineTrait for UnimplementedLine<T> {
    type T = T;
    type PointType<'a> = UnimplementedPoint<Self::T> where Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn start(&self) -> Self::PointType<'_> {
        unimplemented!()
    }

    fn end(&self) -> Self::PointType<'_> {
        unimplemented!()
    }
}
