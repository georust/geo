use super::{utils, CoordIndex, Validation};
use crate::{GeoFloat, Rect};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidRect {
    /// A valid [`Rect`] must have finite coordinates.
    /// Index `0` means the min coordinate, index `1` means the max coordinate.
    NonFiniteCoord(CoordIndex),
}

impl std::error::Error for InvalidRect {}

impl fmt::Display for InvalidRect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidRect::NonFiniteCoord(idx) => {
                let corner = if idx.0 == 0 { "min" } else { "max" };
                write!(f, "coordinate at rect's {corner} is non-finite")
            }
        }
    }
}
impl<F: GeoFloat> Validation for Rect<F> {
    type Error = InvalidRect;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        if utils::check_coord_is_not_finite(&self.min()) {
            handle_validation_error(InvalidRect::NonFiniteCoord(CoordIndex(0)))?;
        }
        if utils::check_coord_is_not_finite(&self.max()) {
            handle_validation_error(InvalidRect::NonFiniteCoord(CoordIndex(1)))?;
        }
        Ok(())
    }
}
