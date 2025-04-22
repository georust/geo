//! # WKB Module
//!
//! The WKB (Well-Known Binary) module provides functionality for working with OGC Well-Known Binary
//! format for geometric objects.

// Make common public so it can be used with crate::common paths

pub mod common;
pub mod error;
pub mod reader;
#[cfg(test)]
pub mod test;

pub use common::Endianness;
