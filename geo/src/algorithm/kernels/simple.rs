use super::Kernel;
use crate::{Coordinate, CoordinateType};
use crate::algorithm::winding_order::WindingOrder;
use std::marker::PhantomData;

/// Simple kernel provides the direct implementation of the
/// predicates. These are meant to be used with exact
/// arithmetic signed tpyes (eg. i32, i64).
#[derive(Default)]
pub struct SimpleKernel<T>(PhantomData<T>);

impl<T: CoordinateType> Kernel for SimpleKernel<T> {
    type Scalar = T;

    fn orient2d(
        p: Coordinate<Self::Scalar>,
        q: Coordinate<Self::Scalar>,
        r: Coordinate<Self::Scalar>,
    ) -> Option<WindingOrder> {

        let res = (q.x - p.x) * (r.y - q.y) - (q.y - p.y) * (r.x - q.x);
        if res > T::zero() {
            Some(WindingOrder::CounterClockwise)
        } else if res < T::zero() {
            Some(WindingOrder::Clockwise)
        } else {
            None
        }
    }
}
