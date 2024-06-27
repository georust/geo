use geo_types::{Coord, CoordNum, Point};

/// A trait for accessing data from a generic Point.
pub trait PointTrait {
    type T: CoordNum;

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

    fn x(&self) -> Self::T {
        self.0.x
    }

    fn y(&self) -> Self::T {
        self.0.y
    }
}

impl<T: CoordNum> PointTrait for &Point<T> {
    type T = T;

    fn x(&self) -> Self::T {
        self.0.x
    }

    fn y(&self) -> Self::T {
        self.0.y
    }
}

impl<T: CoordNum> PointTrait for Coord<T> {
    type T = T;

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}

impl<T: CoordNum> PointTrait for &Coord<T> {
    type T = T;

    fn x(&self) -> Self::T {
        self.x
    }

    fn y(&self) -> Self::T {
        self.y
    }
}
