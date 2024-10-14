use std::marker::PhantomData;

use geo_types::{Coord, CoordNum, Point};

use crate::Dimensions;

/// A trait for accessing data from a generic Point.
///
/// Refer to [geo_types::Point] for information about semantics and validity.
pub trait PointTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// May panic if n >= DIMENSION.
    /// See also [`nth()`](Self::nth).
    fn nth_unchecked(&self, n: usize) -> Self::T;

    /// Dimensions of the coordinate tuple
    fn dim(&self) -> Dimensions;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// Returns NaN if `n >= DIMENSION`.
    /// See also [`nth()`](Self::nth_unchecked).
    fn nth(&self, n: usize) -> Option<Self::T> {
        if n < self.dim().size() {
            Some(self.nth_unchecked(n))
        } else {
            None
        }
    }

    /// x component of this point.
    fn x(&self) -> Self::T;

    /// y component of this point.
    fn y(&self) -> Self::T;

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the point.
    fn x_y(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }
}

impl<T: CoordNum> PointTrait for Point<T> {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Point only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::XY
    }

    fn x(&self) -> Self::T {
        self.0.x
    }

    fn y(&self) -> Self::T {
        self.0.y
    }
}

impl<T: CoordNum> PointTrait for &Point<T> {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Point only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::XY
    }

    fn x(&self) -> Self::T {
        self.0.x
    }

    fn y(&self) -> Self::T {
        self.0.y
    }
}

impl<T: CoordNum> PointTrait for Coord<T> {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Point only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::XY
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: CoordNum> PointTrait for &Coord<T> {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Point only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::XY
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: CoordNum> PointTrait for (T, T) {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("(T, T) only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::XY
    }

    fn x(&self) -> Self::T {
        self.0
    }

    fn y(&self) -> Self::T {
        self.1
    }
}

/// An empty struct that implements [PointTrait].
///
/// This can be used as the `PointType` of the `GeometryTrait` by implementations that don't have a
/// Point concept
pub struct UnimplementedPoint<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> PointTrait for UnimplementedPoint<T> {
    type T = T;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn nth_unchecked(&self, _n: usize) -> Self::T {
        unimplemented!()
    }

    fn x(&self) -> Self::T {
        unimplemented!()
    }

    fn y(&self) -> Self::T {
        unimplemented!()
    }
}
