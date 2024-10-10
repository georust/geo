use geo_types::{Coord, CoordNum, Point};

/// A trait for accessing data from a generic Point.
pub trait PointTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// May panic if n >= DIMENSION.
    /// See also [`nth()`](Self::nth).
    fn nth_unchecked(&self, n: usize) -> Self::T;

    /// Native dimension of the coordinate tuple
    fn dim(&self) -> usize;

    /// Access the n'th (0-based) element of the CoordinateTuple.
    /// Returns NaN if `n >= DIMENSION`.
    /// See also [`nth()`](Self::nth_unchecked).
    fn nth(&self, n: usize) -> Option<Self::T> {
        if n < self.dim() {
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
            _ => panic!(),
        }
    }

    fn dim(&self) -> usize {
        2
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
            _ => panic!(),
        }
    }

    fn dim(&self) -> usize {
        2
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
            _ => panic!(),
        }
    }

    fn dim(&self) -> usize {
        2
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
            _ => panic!(),
        }
    }

    fn dim(&self) -> usize {
        2
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
            _ => panic!(),
        }
    }

    fn dim(&self) -> usize {
        2
    }

    fn x(&self) -> Self::T {
        self.0
    }

    fn y(&self) -> Self::T {
        self.1
    }
}

impl<T: CoordNum, const D: usize> PointTrait for [T; D] {
    type T = T;

    fn nth_unchecked(&self, n: usize) -> Self::T {
        self[n]
    }

    fn dim(&self) -> usize {
        D
    }

    fn x(&self) -> Self::T {
        self[0]
    }

    fn y(&self) -> Self::T {
        self[1]
    }
}
