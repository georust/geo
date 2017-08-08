#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate num_traits;
extern crate spade;

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
