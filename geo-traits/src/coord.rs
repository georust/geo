use std::marker::PhantomData;

use geo_types::{Coord, CoordNum};

use crate::Dimensions;

/// A trait for accessing data from a generic Coord.
///
/// Refer to [geo_types::Coord] for information about semantics and validity.
pub trait CoordTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// Dimensions of the coordinate tuple
    fn dim(&self) -> Dimensions;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// Returns `None` if `n >= DIMENSION`.
    /// See also [`nth_or_panic()`](Self::nth_or_panic).
    fn nth(&self, n: usize) -> Option<Self::T> {
        if n < self.dim().size() {
            Some(self.nth_or_panic(n))
        } else {
            None
        }
    }

    /// x component of this coord.
    fn x(&self) -> Self::T;

    /// y component of this coord.
    fn y(&self) -> Self::T;

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the coord.
    fn x_y(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// May panic if n >= DIMENSION.
    /// See also [`nth()`](Self::nth).
    fn nth_or_panic(&self, n: usize) -> Self::T;
}

impl<T: CoordNum> CoordTrait for Coord<T> {
    type T = T;

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Coord only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: CoordNum> CoordTrait for &Coord<T> {
    type T = T;

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Coord only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: CoordNum> CoordTrait for (T, T) {
    type T = T;

    fn nth_or_panic(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("(T, T) only supports 2 dimensions"),
        }
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
    }

    fn x(&self) -> Self::T {
        self.0
    }

    fn y(&self) -> Self::T {
        self.1
    }
}

/// An empty struct that implements [CoordTrait].
///
/// This can be used as the `CoordType` of the `GeometryTrait` by implementations that don't have a
/// Coord concept
pub struct UnimplementedCoord<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> CoordTrait for UnimplementedCoord<T> {
    type T = T;

    fn dim(&self) -> Dimensions {
        unimplemented!()
    }

    fn nth_or_panic(&self, _n: usize) -> Self::T {
        unimplemented!()
    }

    fn x(&self) -> Self::T {
        unimplemented!()
    }

    fn y(&self) -> Self::T {
        unimplemented!()
    }
}
