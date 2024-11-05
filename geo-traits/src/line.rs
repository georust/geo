use std::marker::PhantomData;

use crate::{CoordTrait, Dimensions, UnimplementedCoord};
#[cfg(feature = "geo-types")]
use geo_types::{Coord, CoordNum, Line};

/// A trait for accessing data from a generic Line.
///
/// A Line is a line segment made up of exactly two [coordinates][CoordTrait].
///
/// Refer to [geo_types::Line] for information about semantics and validity.
pub trait LineTrait: Sized {
    /// The type of each underlying coordinate, which implements [CoordTrait]
    type CoordType<'a>: 'a + CoordTrait
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimensions;

    /// Access the start coordinate in this Line
    fn start(&self) -> Self::CoordType<'_>;

    /// Access the start coordinate in this Line
    fn end(&self) -> Self::CoordType<'_>;

    /// Access the two underlying coordinates
    fn coords(&self) -> [Self::CoordType<'_>; 2] {
        [self.start(), self.end()]
    }
}

#[cfg(feature = "geo-types")]
impl<T: CoordNum> LineTrait for Line<T> {
    type CoordType<'a>
        = &'a Coord<T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn start(&self) -> Self::CoordType<'_> {
        &self.start
    }

    fn end(&self) -> Self::CoordType<'_> {
        &self.end
    }
}

#[cfg(feature = "geo-types")]
impl<'a, T: CoordNum> LineTrait for &'a Line<T> {
    type CoordType<'b>
        = &'a Coord<T>
    where
        Self: 'b;

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn start(&self) -> Self::CoordType<'_> {
        &self.start
    }

    fn end(&self) -> Self::CoordType<'_> {
        &self.end
    }
}

/// An empty struct that implements [LineTrait].
///
/// This can be used as the `LineType` of the `GeometryTrait` by implementations that don't
/// have a Line concept
pub struct UnimplementedLine<T>(PhantomData<T>);

impl<T> LineTrait for UnimplementedLine<T> {
    type CoordType<'a>
        = UnimplementedCoord<T>
    where
        Self: 'a;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn start(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }

    fn end(&self) -> Self::CoordType<'_> {
        unimplemented!()
    }
}
