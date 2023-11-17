use super::CoordTrait;
use geo_types::{Coord, CoordNum, Rect};

pub trait RectTrait {
    type T: CoordNum;
    type ItemType<'a>: 'a + CoordTrait<T = Self::T>
    where
        Self: 'a;

    fn lower(&self) -> Self::ItemType<'_>;

    fn upper(&self) -> Self::ItemType<'_>;

    /// Returns the width of the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.width(), 10.);
    /// ```
    fn width(&self) -> Self::T {
        self.upper().x() - self.lower().x()
    }

    /// Returns the height of the `Rect`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use geo_types::{coord, Rect};
    ///
    /// let rect = Rect::new(
    ///     coord! { x: 5., y: 5. },
    ///     coord! { x: 15., y: 15. },
    /// );
    ///
    /// assert_eq!(rect.height(), 10.);
    /// ```
    fn height(&self) -> Self::T {
        self.upper().y() - self.lower().y()
    }

}

impl<'a, T: CoordNum + 'a> RectTrait for Rect<T> {
    type T = T;
    type ItemType<'b> = Coord<T> where Self: 'b;

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

    fn lower(&self) -> Self::ItemType<'_> {
        self.min()
    }

    fn upper(&self) -> Self::ItemType<'_> {
        self.max()
    }
}
