use super::{utils, Validation};
use crate::{Coord, GeoFloat};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidCoord {
    /// A valid [`Coord`] must be finite.
    NonFinite,
}

impl fmt::Display for InvalidCoord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidCoord::NonFinite => write!(f, "coordinite was non-finite"),
        }
    }
}

impl std::error::Error for InvalidCoord {}

impl<F: GeoFloat> Validation for Coord<F> {
    type Error = InvalidCoord;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        if utils::check_coord_is_not_finite(self) {
            handle_validation_error(InvalidCoord::NonFinite)?;
        }
        Ok(())
    }
}
