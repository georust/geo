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

#[cfg(test)]
mod hegel_props {
    use super::RectOps;
    use crate::utils::hegel_gens::coords;
    use crate::{Intersects, Rect};

    fn rects(tc: &hegel::TestCase) -> Rect<f64> {
        Rect::new(tc.draw(coords(1e3)), tc.draw(coords(1e3)))
    }

    // "Calculate the smallest axis-aligned rectangle that contains both
    // rectangles": it contains both, and shrinking any side would drop a
    // corner of one of them.
    #[hegel::test]
    fn the_union_is_the_smallest_rect_containing_both(tc: hegel::TestCase) {
        let (a, b) = (rects(&tc), rects(&tc));
        let union = a.rect_union(b);
        assert_eq!(union.min().x, a.min().x.min(b.min().x));
        assert_eq!(union.min().y, a.min().y.min(b.min().y));
        assert_eq!(union.max().x, a.max().x.max(b.max().x));
        assert_eq!(union.max().y, a.max().y.max(b.max().y));
    }

    #[hegel::test]
    fn the_union_is_commutative_and_idempotent(tc: hegel::TestCase) {
        let (a, b) = (rects(&tc), rects(&tc));
        assert_eq!(a.rect_union(b), b.rect_union(a));
        assert_eq!(a.rect_union(a), a);
    }

    // "Returns `None` if the rectangles are disjoint" — and the impl gates on
    // `Intersects`, which for rectangles includes touching boundaries.
    #[hegel::test]
    fn the_intersection_is_some_exactly_when_the_rects_intersect(tc: hegel::TestCase) {
        let (a, b) = (rects(&tc), rects(&tc));
        assert_eq!(a.rect_intersection(b).is_some(), a.intersects(&b));
    }

    // "Calculate the axis-aligned rectangle contained in both rectangles":
    // whatever comes back is inside each of them, so unioning it back in
    // changes nothing.
    #[hegel::test]
    fn the_intersection_is_contained_in_both_rects(tc: hegel::TestCase) {
        let (a, b) = (rects(&tc), rects(&tc));
        let Some(intersection) = a.rect_intersection(b) else {
            return;
        };
        assert_eq!(intersection.rect_union(a), a);
        assert_eq!(intersection.rect_union(b), b);
    }

    #[hegel::test]
    fn a_rect_intersected_with_itself_is_itself(tc: hegel::TestCase) {
        let a = rects(&tc);
        assert_eq!(a.rect_intersection(a), Some(a));
    }

    // It is the *largest* such rectangle: each of its four sides is set by
    // whichever input constrains it.
    #[hegel::test]
    fn the_intersection_takes_the_tighter_bound_on_every_side(tc: hegel::TestCase) {
        let (a, b) = (rects(&tc), rects(&tc));
        let Some(intersection) = a.rect_intersection(b) else {
            return;
        };
        assert_eq!(intersection.min().x, a.min().x.max(b.min().x));
        assert_eq!(intersection.min().y, a.min().y.max(b.min().y));
        assert_eq!(intersection.max().x, a.max().x.min(b.max().x));
        assert_eq!(intersection.max().y, a.max().y.min(b.max().y));
    }
}
