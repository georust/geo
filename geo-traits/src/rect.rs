use geo_types::{Coord, CoordNum, Rect};

use crate::{Dimension, PointTrait};

/// A trait for accessing data from a generic Rect.
pub trait RectTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [PointTrait]
    type ItemType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The dimension of this geometry
    fn dim(&self) -> Dimension;

    /// The minimum coordinate of this Rect
    fn min(&self) -> Self::ItemType<'_>;

    /// The maximum coordinate of this Rect
    fn max(&self) -> Self::ItemType<'_>;
}

impl<'a, T: CoordNum + 'a> RectTrait for Rect<T> {
    type T = T;
    type ItemType<'b> = Coord<T> where Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn min(&self) -> Self::ItemType<'_> {
        Rect::min(*self)
    }

    fn max(&self) -> Self::ItemType<'_> {
        Rect::max(*self)
    }
}

impl<'a, T: CoordNum + 'a> RectTrait for &'a Rect<T> {
    type T = T;
    type ItemType<'b> = Coord<T> where Self: 'b;

    fn dim(&self) -> Dimension {
        Dimension::XY
    }

    fn min(&self) -> Self::ItemType<'_> {
        Rect::min(**self)
    }

    fn max(&self) -> Self::ItemType<'_> {
        Rect::max(**self)
    }
}
