use super::Kernel;
use crate::CoordNum;

/// Simple kernel provides the direct implementation of the
/// predicates. These are meant to be used with exact
/// arithmetic signed types (eg. i32, i64).
#[derive(Default, Debug)]
pub struct SimpleKernel;

impl<T: CoordNum> Kernel<T> for SimpleKernel {}
