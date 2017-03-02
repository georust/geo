extern crate cgmath;
extern crate num_traits;

pub use traits::ToGeo;
pub use types::*;
pub use algorithm::*;

mod traits;
mod types;
/// This module includes all the functions of geometric calculations
pub mod algorithm;

#[cfg(test)]
#[macro_use]
extern crate approx;
