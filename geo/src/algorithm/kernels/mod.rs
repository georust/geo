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

pub mod robust;
pub use self::robust::RobustKernel;

pub mod simple;
pub use self::simple::SimpleKernel;
