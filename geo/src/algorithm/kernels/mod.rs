use crate::{Coordinate, CoordinateType};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Orientation {
    CounterClockwise,
    Clockwise,
    Collinear,
}

/// Kernel trait to provide predicates to operate on
/// different scalar types.
pub trait Kernel {
    type Scalar: CoordinateType;

    /// Gives the orientation of 3 2-dimensional points:
    /// ccw, cw or collinear (None)
    fn orient2d(
        p: Coordinate<Self::Scalar>,
        q: Coordinate<Self::Scalar>,
        r: Coordinate<Self::Scalar>,
    ) -> Orientation {
        let res = (q.x - p.x) * (r.y - q.y) - (q.y - p.y) * (r.x - q.x);
        use num_traits::Zero;
        if res > Zero::zero() {
            Orientation::CounterClockwise
        } else if res < Zero::zero() {
            Orientation::Clockwise
        } else {
            Orientation::Collinear
        }
    }

    fn square_euclidean_distance(
        p: Coordinate<Self::Scalar>,
        q: Coordinate<Self::Scalar>,
    ) -> Self::Scalar {
        (p.x - q.x) * (p.x - q.x) + (p.y - q.y) * (p.y - q.y)
    }
}

/// Marker trait to assign Kernel for scalars
pub trait HasKernel: CoordinateType {
    type Ker: Kernel<Scalar = Self>;
}

// Helper macro to implement `HasKernel` on a a scalar type
// `T` (first arg.) by assigning the second arg. It expects
// the second arg. to be a type that takes one generic
// parameter that is `T`.
#[macro_use]
macro_rules! has_kernel {
    ($t:ident, $k:ident) => {
        impl $crate::algorithm::kernels::HasKernel for $t {
            type Ker = $k<$t>;
        }
    };
}

pub mod robust;
pub use self::robust::RobustKernel;
has_kernel!(f64, RobustKernel);
has_kernel!(f32, RobustKernel);

pub mod simple;
pub use self::simple::SimpleKernel;

has_kernel!(i64, SimpleKernel);
has_kernel!(i32, SimpleKernel);
has_kernel!(i16, SimpleKernel);
has_kernel!(isize, SimpleKernel);

#[cfg(has_i128)]
has_kernel!(i128, SimpleKernel);
