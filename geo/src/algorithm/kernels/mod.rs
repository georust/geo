use crate::{CoordinateType, Coordinate};
use super::winding_order::WindingOrder;

pub trait Kernel {
    type Scalar: CoordinateType;

    fn orient2d(
        p: Coordinate<Self::Scalar>,
        q: Coordinate<Self::Scalar>,
        r: Coordinate<Self::Scalar>,
    ) -> Option<WindingOrder>;
}

/// Marker trait
pub trait HasKernel: CoordinateType {
    type Ker: Kernel<Scalar = Self>;
}

pub mod robust;
pub use self::robust::RobustKernel;

pub mod simple;
pub use self::simple::SimpleKernel;

impl HasKernel for f64 {
    type Ker = RobustKernel<f64>;
}

impl HasKernel for f32 {
    type Ker = RobustKernel<f32>;
}

impl HasKernel for i64 {
    type Ker = SimpleKernel<i64>;
}

impl HasKernel for i32 {
    type Ker = SimpleKernel<i32>;
}
