pub mod error;
pub mod helper;
pub mod trait_def;
pub mod trait_impl;

pub use error::SpadeBoolopsError;
pub use trait_def::SpadeBoolops;

#[cfg(test)]
pub mod tests;
