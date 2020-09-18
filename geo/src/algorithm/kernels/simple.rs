use super::Kernel;
use crate::CoordinateType;

/// Simple kernel provides the direct implementation of the
/// predicates. These are meant to be used with exact
/// arithmetic signed tpyes (eg. i32, i64).
#[derive(Default)]
pub struct SimpleKernel;

impl<T: CoordinateType> Kernel<T> for SimpleKernel {}
