use super::{utils, CoordIndex, Validation};
use crate::{GeoFloat, Line};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidLine {
    /// A valid [`Line`] must have at least 2 distinct points to be valid - it must have a non-zero length.
    IdenticalCoords,

    /// A valid [`Line`] must have finite coordinates.
    NonFiniteCoord(CoordIndex),
}

impl fmt::Display for InvalidLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidLine::IdenticalCoords => write!(f, "line has identical coordinates"),
            InvalidLine::NonFiniteCoord(idx) => write!(
                f,
                "{} coordinate is non-finite",
                if idx.0 == 0 { "start" } else { "end" }
            ),
        }
    }
}

impl std::error::Error for InvalidLine {}

impl<F: GeoFloat> Validation for Line<F> {
    type Error = InvalidLine;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        if utils::check_coord_is_not_finite(&self.start) {
            handle_validation_error(InvalidLine::NonFiniteCoord(CoordIndex(0)))?;
        }
        if utils::check_coord_is_not_finite(&self.end) {
            handle_validation_error(InvalidLine::NonFiniteCoord(CoordIndex(1)))?;
        }
        if self.start == self.end {
            handle_validation_error(InvalidLine::IdenticalCoords)?
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};

    #[test]
    fn test_line_valid() {
        let l = Line::new((0., 0.), (1., 1.));
        assert_valid!(l);
    }

    #[test]
    fn test_line_invalid_not_finite_coords() {
        let l = Line::new((0., 0.), (f64::NEG_INFINITY, 0.));
        assert_validation_errors!(l, vec![InvalidLine::NonFiniteCoord(CoordIndex(1))]);
    }

    #[test]
    fn test_line_invalid_same_points() {
        let l = Line::new((0., 0.), (0., 0.));
        assert_validation_errors!(l, vec![InvalidLine::IdenticalCoords]);
    }
}
