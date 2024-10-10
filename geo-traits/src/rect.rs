use geo_types::{Coord, CoordNum, Rect};

use super::PointTrait;

/// A trait for accessing data from a generic Rect.
pub trait RectTrait {
    /// The coordinate type of this geometry
    type T: CoordNum;

    /// The type of each underlying coordinate, which implements [PointTrait]
    type ItemType<'a>: 'a + PointTrait<T = Self::T>
    where
        Self: 'a;

    /// The number of dimensions in this geometry
    fn dim(&self) -> usize;

    /// The lower coordinate of this Rect
    fn lower(&self) -> Self::ItemType<'_>;

    /// The upper coordinate of this Rect
    fn upper(&self) -> Self::ItemType<'_>;
}

impl<'a, T: CoordNum + 'a> RectTrait for Rect<T> {
    type T = T;
    type ItemType<'b> = Coord<T> where Self: 'b;

    fn dim(&self) -> usize {
        2
    }

    fn lower(&self) -> Self::ItemType<'_> {
        self.min()
    }

    fn upper(&self) -> Self::ItemType<'_> {
        self.max()
    }
}

impl<'a, T: CoordNum + 'a> RectTrait for &'a Rect<T> {
    type T = T;
    type ItemType<'b> = Coord<T> where Self: 'b;

    fn dim(&self) -> usize {
        2
    }

    fn lower(&self) -> Self::ItemType<'_> {
        self.min()
    }

    fn upper(&self) -> Self::ItemType<'_> {
        self.max()
    }
}
