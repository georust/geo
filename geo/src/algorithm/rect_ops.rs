use crate::utils::{partial_max, partial_min};
use crate::{CoordNum, Intersects, Rect, coord};

/// Union and intersection of axis-aligned rectangles.
pub trait RectOps<T: CoordNum> {
    /// Calculate the smallest axis-aligned rectangle that contains both rectangles.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{coord, Rect, RectOps};
    ///
    /// let a = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 2., y: 2. });
    /// let b = Rect::new(coord! { x: 1., y: 1. }, coord! { x: 3., y: 3. });
    ///
    /// assert_eq!(
    ///     a.rect_union(b),
    ///     Rect::new(coord! { x: 0., y: 0. }, coord! { x: 3., y: 3. }),
    /// );
    /// ```
    #[must_use]
    fn rect_union(&self, other: Rect<T>) -> Rect<T>;

    /// Calculate the axis-aligned rectangle contained in both rectangles, if
    /// there is any.
    ///
    /// Returns `None` if the rectangles are disjoint.
    ///
    /// # Examples
    ///
    /// ```
    /// use geo::{coord, Rect, RectOps};
    ///
    /// let a = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 2., y: 2. });
    /// let b = Rect::new(coord! { x: 1., y: 1. }, coord! { x: 3., y: 3. });
    ///
    /// assert_eq!(
    ///     a.rect_intersection(b),
    ///     Some(Rect::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. })),
    /// );
    ///
    /// let c = Rect::new(coord! { x: 5., y: 5. }, coord! { x: 6., y: 6. });
    /// assert_eq!(a.rect_intersection(c), None);
    /// ```
    fn rect_intersection(&self, other: Rect<T>) -> Option<Rect<T>>;
}

impl<T: CoordNum> RectOps<T> for Rect<T> {
    fn rect_union(&self, other: Rect<T>) -> Rect<T> {
        Rect::new(
            coord! {
                x: partial_min(self.min().x, other.min().x),
                y: partial_min(self.min().y, other.min().y),
            },
            coord! {
                x: partial_max(self.max().x, other.max().x),
                y: partial_max(self.max().y, other.max().y),
            },
        )
    }

    fn rect_intersection(&self, other: Rect<T>) -> Option<Rect<T>> {
        if !self.intersects(&other) {
            return None;
        }

        Some(Rect::new(
            coord! {
                x: partial_max(self.min().x, other.min().x),
                y: partial_max(self.min().y, other.min().y),
            },
            coord! {
                x: partial_min(self.max().x, other.max().x),
                y: partial_min(self.max().y, other.max().y),
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rect_union() {
        let a = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 2., y: 2. });
        let b = Rect::new(coord! { x: 1., y: 1. }, coord! { x: 3., y: 3. });

        assert_eq!(
            a.rect_union(b),
            Rect::new(coord! { x: 0., y: 0. }, coord! { x: 3., y: 3. }),
        );
    }

    #[test]
    fn rect_intersection() {
        let a = Rect::new(coord! { x: 0., y: 0. }, coord! { x: 2., y: 2. });
        let b = Rect::new(coord! { x: 1., y: 1. }, coord! { x: 3., y: 3. });
        let c = Rect::new(coord! { x: 5., y: 5. }, coord! { x: 6., y: 6. });

        assert_eq!(
            a.rect_intersection(b),
            Some(Rect::new(coord! { x: 1., y: 1. }, coord! { x: 2., y: 2. })),
        );
        assert_eq!(a.rect_intersection(c), None);
    }
}
