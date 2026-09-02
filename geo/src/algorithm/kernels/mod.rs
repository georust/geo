use num_traits::Zero;
use std::cmp::Ordering;

use crate::{Coord, CoordNum, coord};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Orientation {
    CounterClockwise,
    Clockwise,
    Collinear,
}

impl Orientation {
    /// Helper to convert orientation-2d into an ordering.
    #[inline]
    pub(crate) fn as_ordering(&self) -> Ordering {
        match self {
            Orientation::CounterClockwise => Ordering::Less,
            Orientation::Clockwise => Ordering::Greater,
            Orientation::Collinear => Ordering::Equal,
        }
    }
}

/// Kernel trait to provide predicates to operate on
/// different scalar types.
pub trait Kernel<T: CoordNum> {
    /// Gives the orientation of 3 2-dimensional points:
    /// ccw, cw or collinear (None)
    fn orient2d(p: Coord<T>, q: Coord<T>, r: Coord<T>) -> Orientation {
        let res = (q.x - p.x) * (r.y - q.y) - (q.y - p.y) * (r.x - q.x);
        if res > Zero::zero() {
            Orientation::CounterClockwise
        } else if res < Zero::zero() {
            Orientation::Clockwise
        } else {
            Orientation::Collinear
        }
    }

    fn square_euclidean_distance(p: Coord<T>, q: Coord<T>) -> T {
        (p.x - q.x) * (p.x - q.x) + (p.y - q.y) * (p.y - q.y)
    }

    /// Compute the sign of the dot product of `u` and `v` using
    /// robust predicates. The output is `CounterClockwise` if
    /// the sign is positive, `Clockwise` if negative, and
    /// `Collinear` if zero.
    fn dot_product_sign(u: Coord<T>, v: Coord<T>) -> Orientation {
        let zero = Coord::zero();
        let vdash = coord! {
            x: T::zero() - v.y,
            y: v.x,
        };
        Self::orient2d(zero, u, vdash)
    }
}

pub mod robust;
pub use self::robust::RobustKernel;

pub mod simple;
pub use self::simple::SimpleKernel;

#[cfg(test)]
mod hegel_props {
    use super::{Kernel, Orientation};
    use crate::Coord;
    use crate::kernels::{RobustKernel, SimpleKernel};
    use crate::utils::pbt_gens::{coords, grid_coords};

    fn inverse(orientation: Orientation) -> Orientation {
        match orientation {
            Orientation::CounterClockwise => Orientation::Clockwise,
            Orientation::Clockwise => Orientation::CounterClockwise,
            Orientation::Collinear => Orientation::Collinear,
        }
    }

    fn triple(tc: &hegel::TestCase) -> (Coord<f64>, Coord<f64>, Coord<f64>) {
        (
            tc.draw(coords(1e6)),
            tc.draw(coords(1e6)),
            tc.draw(coords(1e6)),
        )
    }

    // "Gives the orientation of 3 2-dimensional points": the sign of a
    // determinant, which is unchanged by a cyclic rotation of its rows.
    #[hegel::test]
    fn orient2d_is_invariant_under_cyclic_rotation(tc: hegel::TestCase) {
        let (p, q, r) = triple(&tc);
        assert_eq!(
            RobustKernel::orient2d(p, q, r),
            RobustKernel::orient2d(q, r, p)
        );
        assert_eq!(
            RobustKernel::orient2d(p, q, r),
            RobustKernel::orient2d(r, p, q)
        );
    }

    // Swapping two of the three points swaps two rows of the determinant, which
    // flips its sign.
    #[hegel::test]
    fn swapping_two_points_inverts_orient2d(tc: hegel::TestCase) {
        let (p, q, r) = triple(&tc);
        assert_eq!(
            RobustKernel::orient2d(p, r, q),
            inverse(RobustKernel::orient2d(p, q, r))
        );
    }

    #[hegel::test]
    fn a_repeated_point_is_collinear(tc: hegel::TestCase) {
        let (p, _, r) = triple(&tc);
        assert_eq!(RobustKernel::orient2d(p, p, r), Orientation::Collinear);
        assert_eq!(RobustKernel::orient2d(p, r, r), Orientation::Collinear);
    }

    // Orientation depends only on the shape, not on where it sits, so
    // translating all three points leaves it alone. Coordinates come from the
    // integer grid so the translated points are exact.
    #[hegel::test]
    fn orient2d_is_invariant_under_translation(tc: hegel::TestCase) {
        let (p, q, r) = (
            tc.draw(grid_coords()),
            tc.draw(grid_coords()),
            tc.draw(grid_coords()),
        );
        let offset = tc.draw(grid_coords());
        assert_eq!(
            RobustKernel::orient2d(p, q, r),
            RobustKernel::orient2d(p + offset, q + offset, r + offset)
        );
    }

    // `SimpleKernel` "provides the direct implementation of the predicates.
    // These are meant to be used with exact arithmetic signed types", and
    // `RobustKernel` promises robust floating point predicates. On the integer
    // grid the naive determinant is exact, so the two must agree — which makes
    // the naive kernel an independent oracle for the robust one.
    #[hegel::test]
    fn the_robust_kernel_agrees_with_the_simple_one_on_integer_coords(tc: hegel::TestCase) {
        let (p, q, r) = (
            tc.draw(grid_coords()),
            tc.draw(grid_coords()),
            tc.draw(grid_coords()),
        );
        assert_eq!(
            RobustKernel::orient2d(p, q, r),
            SimpleKernel::orient2d(p, q, r)
        );
    }

    // "Compute the sign of the dot product of `u` and `v` ... The output is
    // `CounterClockwise` if the sign is positive, `Clockwise` if negative, and
    // `Collinear` if zero." On the integer grid the dot product is exact.
    #[hegel::test]
    fn dot_product_sign_reports_the_sign_of_the_dot_product(tc: hegel::TestCase) {
        let u = tc.draw(grid_coords());
        let v = tc.draw(grid_coords());
        let dot = u.x * v.x + u.y * v.y;
        let expected = if dot > 0.0 {
            Orientation::CounterClockwise
        } else if dot < 0.0 {
            Orientation::Clockwise
        } else {
            Orientation::Collinear
        };
        assert_eq!(RobustKernel::dot_product_sign(u, v), expected);
    }
}
