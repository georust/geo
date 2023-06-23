use geo_types::{Coord, CoordNum, Point};

pub trait CoordTrait: Send + Sync {
    type T: CoordNum + Send + Sync;

    /// x component of this coord
    fn x(&self) -> Self::T;

    /// y component of this coord
    fn y(&self) -> Self::T;

    /// Returns a tuple that contains the x/horizontal & y/vertical component of the coord.
    fn x_y(&self) -> (Self::T, Self::T) {
        (self.x(), self.y())
    }
}

impl<T: CoordNum + Send + Sync> CoordTrait for Point<T> {
    type T = T;

    fn x(&self) -> T {
        self.0.x
    }

    fn y(&self) -> T {
        self.0.y
    }
}

impl<T: CoordNum + Send + Sync> CoordTrait for &Point<T> {
    type T = T;

    fn x(&self) -> T {
        self.0.x
    }

    fn y(&self) -> T {
        self.0.y
    }
}

impl<T: CoordNum + Send + Sync> CoordTrait for Coord<T> {
    type T = T;

    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}

impl<T: CoordNum + Send + Sync> CoordTrait for &Coord<T> {
    type T = T;

    fn x(&self) -> T {
        self.x
    }

    fn y(&self) -> T {
        self.y
    }
}
