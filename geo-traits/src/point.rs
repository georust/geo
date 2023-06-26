use geo_types::{Coord, CoordNum, Point};

pub trait PointTrait {
    type T: CoordNum;

    /// x component of this coord
    fn x(&self) -> Self::T;

    /// y component of this coord
    fn y(&self) -> Self::T;

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the coord.
    fn x_y(&self) -> (Self::T, Self::T);
}

impl<T: CoordNum> PointTrait for Point<T> {
    type T = T;

    fn x(&self) -> T {
        self.0.x
    }

    fn y(&self) -> T {
        self.0.y
    }

    fn x_y(&self) -> (T, T) {
        (self.0.x, self.0.y)
    }
}

impl<T: CoordNum> PointTrait for &Point<T> {
    type T = T;

    fn x(&self) -> T {
        self.0.x
    }

    fn y(&self) -> T {
        self.0.y
    }

    fn x_y(&self) -> (T, T) {
        (self.0.x, self.0.y)
    }
}

impl<T: CoordNum> PointTrait for Coord<T> {
    type T = T;

    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T: CoordNum> PointTrait for &Coord<T> {
    type T = T;

    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }

    fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }
}
