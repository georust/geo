use super::kernels::*;
use crate::coords_iter::CoordsIter;
use crate::utils::EitherIter;
use crate::{CoordNum, GeoFloat, GeoNum, LineString, Point};
use geo_types::{PointsIter, Triangle};
use std::iter::Rev;

/// Iterates through a list of `Point`s
#[allow(missing_debug_implementations)]
pub struct Points<'a, T>(pub(crate) EitherIter<PointsIter<'a, T>, Rev<PointsIter<'a, T>>>)
where
    T: CoordNum + 'a;

impl<T> Iterator for Points<'_, T>
where
    T: CoordNum,
{
    type Item = Point<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T> ExactSizeIterator for Points<'_, T>
where
    T: CoordNum,
{
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// How a linestring is wound, clockwise or counter-clockwise
#[derive(PartialEq, Clone, Debug, Eq, Copy)]
pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

impl WindingOrder {
    #[allow(dead_code)]
    pub(crate) fn inverse(&self) -> Self {
        match self {
            WindingOrder::Clockwise => WindingOrder::CounterClockwise,
            WindingOrder::CounterClockwise => WindingOrder::Clockwise,
        }
    }
}

/// Determine and operate on how a [`LineString`] is
/// wound. This functionality, and our implementation is
/// based on [CGAL's Polygon_2::orientation].
///
/// [CGAL's Polygon_2::orientation]: //doc.cgal.org/latest/Polygon/classCGAL_1_1Polygon__2.html#a4ce8b4b8395406243ac16c2a120ffc15
pub trait Winding {
    type Scalar: CoordNum;

    /// Return the winding order of this object if it
    /// contains at least three distinct coordinates, and
    /// `None` otherwise.
    fn winding_order(&self) -> Option<WindingOrder>;

    /// True iff this is wound clockwise
    fn is_cw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::Clockwise)
    }

    /// True iff this is wound counterclockwise
    fn is_ccw(&self) -> bool {
        self.winding_order() == Some(WindingOrder::CounterClockwise)
    }

    /// Iterate over the points in a clockwise order
    ///
    /// The object isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear clockwise
    fn points_cw(&self) -> Points<'_, Self::Scalar>;

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The object isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    fn points_ccw(&self) -> Points<'_, Self::Scalar>;

    /// Change this object's points so they are in clockwise winding order
    fn make_cw_winding(&mut self);

    /// Change this line's points so they are in counterclockwise winding order
    fn make_ccw_winding(&mut self);

    /// Return a clone of this object, but in the specified winding order
    fn clone_to_winding_order(&self, winding_order: WindingOrder) -> Self
    where
        Self: Sized + Clone,
    {
        let mut new: Self = self.clone();
        new.make_winding_order(winding_order);
        new
    }

    /// Change the winding order so that it is in this winding order
    fn make_winding_order(&mut self, winding_order: WindingOrder) {
        match winding_order {
            WindingOrder::Clockwise => self.make_cw_winding(),
            WindingOrder::CounterClockwise => self.make_ccw_winding(),
        }
    }
}

impl<T, K> Winding for LineString<T>
where
    T: GeoNum<Ker = K>,
    K: Kernel<T>,
{
    type Scalar = T;

    fn winding_order(&self) -> Option<WindingOrder> {
        // If linestring has at most 3 coords, it is either
        // not closed, or is at most two distinct points.
        // Either way, the WindingOrder is unspecified.
        if self.coords_count() < 4 || !self.is_closed() {
            return None;
        }

        let increment = |x: &mut usize| {
            *x += 1;
            if *x >= self.coords_count() {
                *x = 0;
            }
        };

        let decrement = |x: &mut usize| {
            if *x == 0 {
                *x = self.coords_count() - 1;
            } else {
                *x -= 1;
            }
        };

        use crate::utils::least_index;
        let i = least_index(&self.0);

        let mut next = i;
        increment(&mut next);
        while self.0[next] == self.0[i] {
            if next == i {
                // We've looped too much. There aren't
                // enough unique coords to compute orientation.
                return None;
            }
            increment(&mut next);
        }

        let mut prev = i;
        decrement(&mut prev);
        while self.0[prev] == self.0[i] {
            // Note: we don't need to check if prev == i as
            // the previous loop succeeded, and so we have
            // at least two distinct elements in the list
            decrement(&mut prev);
        }

        match K::orient2d(self.0[prev], self.0[i], self.0[next]) {
            Orientation::CounterClockwise => Some(WindingOrder::CounterClockwise),
            Orientation::Clockwise => Some(WindingOrder::Clockwise),
            _ => None,
        }
    }

    /// Iterate over the points in a clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear clockwise
    fn points_cw(&self) -> Points<'_, Self::Scalar> {
        match self.winding_order() {
            Some(WindingOrder::CounterClockwise) => Points(EitherIter::B(self.points().rev())),
            _ => Points(EitherIter::A(self.points())),
        }
    }

    /// Iterate over the points in a counter-clockwise order
    ///
    /// The Linestring isn't changed, and the points are returned either in order, or in reverse
    /// order, so that the resultant order makes it appear counter-clockwise
    fn points_ccw(&self) -> Points<'_, Self::Scalar> {
        match self.winding_order() {
            Some(WindingOrder::Clockwise) => Points(EitherIter::B(self.points().rev())),
            _ => Points(EitherIter::A(self.points())),
        }
    }

    /// Change this line's points so they are in clockwise winding order
    fn make_cw_winding(&mut self) {
        if let Some(WindingOrder::CounterClockwise) = self.winding_order() {
            self.0.reverse();
        }
    }

    /// Change this line's points so they are in counterclockwise winding order
    fn make_ccw_winding(&mut self) {
        if let Some(WindingOrder::Clockwise) = self.winding_order() {
            self.0.reverse();
        }
    }
}

// This function can probably be converted into a trait implementation with a small refactoring of
// the trait but this is not in scope of the PR it is added for.
/// special cased algorithm for finding the winding of a triangle
pub fn triangle_winding_order<T: GeoFloat>(tri: &Triangle<T>) -> Option<WindingOrder> {
    let [a, b, c] = tri.to_array();
    let ab = b - a;
    let ac = c - a;

    let cross_prod = ab.x * ac.y - ab.y * ac.x;

    match cross_prod.total_cmp(&T::zero()) {
        std::cmp::Ordering::Less => Some(WindingOrder::Clockwise),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(WindingOrder::CounterClockwise),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Point;

    #[test]
    fn robust_winding_float() {
        // 3 points forming a triangle
        let a = Point::new(0., 0.);
        let b = Point::new(2., 0.);
        let c = Point::new(1., 2.);

        // Verify open linestrings return None
        let mut ls = LineString::from(vec![a.0, b.0, c.0]);
        assert!(ls.winding_order().is_none());

        ls.0.push(ls.0[0]);
        assert_eq!(ls.winding_order(), Some(WindingOrder::CounterClockwise));

        ls.make_cw_winding();
        assert_eq!(ls.winding_order(), Some(WindingOrder::Clockwise));
    }

    #[test]
    fn robust_winding_integer() {
        // 3 points forming a triangle
        let a = Point::new(0i64, 0);
        let b = Point::new(2, 0);
        let c = Point::new(1, 2);

        // Verify open linestrings return None
        let mut ls = LineString::from(vec![a.0, b.0, c.0]);
        assert!(ls.winding_order().is_none());

        ls.0.push(ls.0[0]);
        assert!(ls.is_ccw());

        let ccw_ls: Vec<_> = ls.points_ccw().collect();

        ls.make_cw_winding();
        assert!(ls.is_cw());

        assert_eq!(&ls.points_ccw().collect::<Vec<_>>(), &ccw_ls,);
    }
}

#[cfg(test)]
mod hegel_props {
    use super::{Winding, WindingOrder, triangle_winding_order};
    use crate::utils::hegel_gens::{grid_coords, star_polygons};
    use crate::{LineString, Triangle, coord};

    // The generator walks its vertices at strictly increasing angles about a
    // centre, which is counter-clockwise.
    #[hegel::test]
    fn a_ring_traced_at_increasing_angles_is_counter_clockwise(tc: hegel::TestCase) {
        let ring = tc.draw(star_polygons()).exterior().clone();
        assert_eq!(ring.winding_order(), Some(WindingOrder::CounterClockwise));
    }

    #[hegel::test]
    fn reversing_a_ring_inverts_its_winding_order(tc: hegel::TestCase) {
        let ring = tc.draw(star_polygons()).exterior().clone();
        let reversed = LineString::new(ring.0.iter().rev().copied().collect());
        assert_eq!(
            reversed.winding_order(),
            ring.winding_order().map(|order| order.inverse())
        );
    }

    // `is_cw` and `is_ccw` are documented as "True iff this is wound
    // clockwise"/"counterclockwise", and `WindingOrder` has no third variant, so
    // at most one of them can hold.
    #[hegel::test]
    fn at_most_one_winding_predicate_holds(tc: hegel::TestCase) {
        let ring = tc.draw(star_polygons()).exterior().clone();
        assert_eq!(
            ring.is_cw(),
            ring.winding_order() == Some(WindingOrder::Clockwise)
        );
        assert_eq!(
            ring.is_ccw(),
            ring.winding_order() == Some(WindingOrder::CounterClockwise)
        );
        assert!(!(ring.is_cw() && ring.is_ccw()));
    }

    // `make_winding_order` changes "the winding order so that it is in this
    // winding order", and the only way to do that without moving vertices is to
    // leave the ring alone or reverse it wholesale.
    #[hegel::test]
    fn make_winding_order_either_keeps_or_reverses_the_coords(tc: hegel::TestCase) {
        let ring = tc.draw(star_polygons()).exterior().clone();
        let order = if tc.draw(hegel::generators::booleans()) {
            WindingOrder::Clockwise
        } else {
            WindingOrder::CounterClockwise
        };
        let mut rewound = ring.clone();
        rewound.make_winding_order(order);
        assert_eq!(rewound.winding_order(), Some(order));
        let reversed: Vec<_> = ring.0.iter().rev().copied().collect();
        assert!(rewound.0 == ring.0 || rewound.0 == reversed);
    }

    // `points_cw` and `points_ccw` return the points "either in order, or in
    // reverse order", so for a ring with a defined winding they are exact
    // reverses of one another.
    #[hegel::test]
    fn points_cw_is_the_reverse_of_points_ccw(tc: hegel::TestCase) {
        let ring = tc.draw(star_polygons()).exterior().clone();
        let cw: Vec<_> = ring.points_cw().collect();
        let ccw: Vec<_> = ring.points_ccw().collect();
        assert_eq!(cw, ccw.into_iter().rev().collect::<Vec<_>>());
    }

    // `triangle_winding_order` is a "special cased algorithm" using a plain
    // cross product, while `LineString::winding_order` goes through the kernel.
    // On the exact integer grid the cross product is exact, so the two must
    // agree. Collinear triangles are excluded: they disagree there, pinned by
    // `a_degenerate_triangle_has_no_winding_order` below.
    #[hegel::test]
    fn triangle_winding_order_agrees_with_its_rings_winding(tc: hegel::TestCase) {
        let triangle = Triangle::new(
            tc.draw(grid_coords()),
            tc.draw(grid_coords()),
            tc.draw(grid_coords()),
        );
        let [a, b, c] = triangle.to_array();
        tc.assume(
            robust::orient2d(
                robust::Coord { x: a.x, y: a.y },
                robust::Coord { x: b.x, y: b.y },
                robust::Coord { x: c.x, y: c.y },
            ) != 0.0,
        );
        assert_eq!(
            triangle_winding_order(&triangle),
            triangle.to_polygon().exterior().winding_order()
        );
    }

    // KNOWN FAILURE, #1608: `triangle_winding_order` reports a
    // triangle with two distinct vertices as clockwise, where the same
    // triangle's exterior ring reports no winding order.
    #[test]
    #[ignore = "#1608: triangle_winding_order reports a two-vertex triangle as clockwise"]
    fn a_degenerate_triangle_has_no_winding_order() {
        let triangle = Triangle::new(
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.0, y: -1.0 },
        );
        assert_eq!(triangle_winding_order(&triangle), None);
    }
}
