use geo_types::{Coord, CoordNum, Point};

/// A trait for accessing data from a generic Coord.
pub trait CoordTrait {
    type T: CoordNum;

    /// x component of this coord
    fn x(&self) -> Self::T;

    /// y component of this coord
    fn y(&self) -> Self::T;

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the coord.
    fn x_y(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }
}

impl<T: CoordNum> CoordTrait for Point<T> {
    type T = T;

    fn x(&self) -> Self::T {
        self.0.x
    }

    fn y(&self) -> Self::T {
        self.0.y
    }
}

impl<T: CoordNum> CoordTrait for &Point<T> {
    type T = T;

    fn x(&self) -> Self::T {
        self.0.x
    }

    fn y(&self) -> Self::T {
        self.0.y
    }
}

impl<T: CoordNum> CoordTrait for Coord<T> {
    type T = T;

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: CoordNum> CoordTrait for &Coord<T> {
    type T = T;

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: CoordNum> CoordTrait for (T, T) {
    type T = T;

    fn x(&self) -> Self::T {
        self.0
    }

    fn y(&self) -> Self::T {
        self.1
    }
}
