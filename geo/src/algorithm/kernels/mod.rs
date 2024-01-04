use num_traits::Zero;
use std::cmp::Ordering;

use crate::{coord, Coord, CoordNum};

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

/// Marker trait to assign Kernel for scalars
pub trait HasKernel: CoordNum {
    type Ker: Kernel<Self>;

    fn total_cmp(&self, other: &Self) -> Ordering;
}

// Helper macro to implement `HasKernel` for float types
macro_rules! has_robust_kernel {
    ($t:ident) => {
        impl $crate::HasKernel for $t {
            type Ker = RobustKernel;

            fn total_cmp(&self, other: &Self) -> Ordering {
                self.total_cmp(other)
            }
        }
    };
}

// Helper macro to implement `HasKernel` for integer types
macro_rules! has_simple_kernel {
    ($t:ident) => {
        impl $crate::HasKernel for $t {
            type Ker = SimpleKernel;

            fn total_cmp(&self, other: &Self) -> Ordering {
                self.cmp(other)
            }
        }
    };
}

pub mod robust;
pub use self::robust::RobustKernel;
has_robust_kernel!(f64);
has_robust_kernel!(f32);

pub mod simple;
pub use self::simple::SimpleKernel;

has_simple_kernel!(i64);
has_simple_kernel!(i32);
has_simple_kernel!(i16);
has_simple_kernel!(isize);

#[cfg(has_i128)]
has_simple_kernel!(i128, SimpleKernel);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_ord_float() {
        assert_eq!(HasKernel::total_cmp(&3.0f64, &2.0f64), Ordering::Greater);
        assert_eq!(HasKernel::total_cmp(&2.0f64, &2.0f64), Ordering::Equal);
        assert_eq!(HasKernel::total_cmp(&1.0f64, &2.0f64), Ordering::Less);
        assert_eq!(HasKernel::total_cmp(&1.0f64, &f64::NAN), Ordering::Less);
        assert_eq!(HasKernel::total_cmp(&f64::NAN, &f64::NAN), Ordering::Equal);
        assert_eq!(
            HasKernel::total_cmp(&f64::INFINITY, &f64::NAN),
            Ordering::Less
        );
    }

    #[test]
    fn total_ord_int() {
        assert_eq!(HasKernel::total_cmp(&3i32, &2i32), Ordering::Greater);
        assert_eq!(HasKernel::total_cmp(&2i32, &2i32), Ordering::Equal);
        assert_eq!(HasKernel::total_cmp(&1i32, &2i32), Ordering::Less);
    }
}
