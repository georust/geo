//! Defines [`WKBError`], representing all errors returned by this crate.

use std::borrow::Cow;
use std::fmt::Debug;
use thiserror::Error;

/// Enum with all errors in this crate.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum WKBError {
    /// Incorrect type was passed to an operation.
    #[error("Incorrect type passed to operation: {0}")]
    IncorrectType(Cow<'static, str>),

    /// Returned when functionality is not yet available.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),

    /// General error.
    #[error("General error: {0}")]
    General(String),

    /// [std::io::Error]
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

/// Crate-specific result type.
pub type WKBResult<T> = std::result::Result<T, WKBError>;
