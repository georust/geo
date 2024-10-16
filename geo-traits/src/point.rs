use std::marker::PhantomData;

use geo_types::{Coord, CoordNum, Point};

use crate::Dimensions;

/// A trait for accessing data from a generic Point.
///
/// Refer to [geo_types::Point] for information about semantics and validity.
pub trait PointTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// Dimensions of the coordinate tuple
    fn dim(&self) -> Dimensions;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// Returns NaN if `n >= DIMENSION`.
    /// See also [`nth_unchecked()`](Self::nth_unchecked).
    fn nth(&self, n: usize) -> Option<Self::T> {
        if n < self.dim().size() {
            Some(self.nth_unchecked(n))
        } else {
            None
        }
    }

    /// Whether this point is `empty` or not.
    ///
    /// According to Simple Features, a Point can have zero coordinates and be considered `empty`.
    ///
    /// If `is_empty` returns `true`, then the values of `x()`, `y()`, `nth()` and `nth_unchecked`
    /// have no semantic meaning.
    ///
    /// Only a top-level geometry can be empty. That is, when this point is contained within
    /// another geometry, such as a [`LineStringTrait`][geo_traits::LineStringTrait], those points
    /// can never be empty, and a consumer does not need to check this method.
    fn is_empty(&self) -> bool;

    /// x component of this point.
    fn x(&self) -> Self::T;

    /// y component of this point.
    fn y(&self) -> Self::T;

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the point.
    fn x_y(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// May panic if n >= DIMENSION.
    /// See also [`nth()`](Self::nth).
    fn nth_unchecked(&self, n: usize) -> Self::T;
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

    fn is_empty(&self) -> bool {
        false
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
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

    fn is_empty(&self) -> bool {
        false
    }

    fn dim(&self) -> Dimensions {
        Dimensions::Xy
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

    fn is_empty(&self) -> bool {
        false
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

impl<T: CoordNum> PointTrait for &Coord<T> {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("Point only supports 2 dimensions"),
        }
    }

    fn is_empty(&self) -> bool {
        false
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

impl<T: CoordNum> PointTrait for (T, T) {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        match n {
            0 => self.x(),
            1 => self.y(),
            _ => panic!("(T, T) only supports 2 dimensions"),
        }
    }

    fn is_empty(&self) -> bool {
        false
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

/// An empty struct that implements [PointTrait].
///
/// This can be used as the `PointType` of the `GeometryTrait` by implementations that don't have a
/// Point concept
pub struct UnimplementedPoint<T: CoordNum>(PhantomData<T>);

impl<T: CoordNum> PointTrait for UnimplementedPoint<T> {
    type T = T;

    fn is_empty(&self) -> bool {
        unimplemented!()
    }

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
