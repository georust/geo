use super::Kernel;
use crate::CoordinateType;
use std::marker::PhantomData;

/// Simple kernel provides the direct implementation of the
/// predicates. These are meant to be used with exact
/// arithmetic signed tpyes (eg. i32, i64).
#[derive(Default)]
pub struct SimpleKernel<T>(PhantomData<T>);

impl<T: CoordinateType> Kernel for SimpleKernel<T> {
    type Scalar = T;
}
