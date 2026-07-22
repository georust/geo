use super::{CoordIndex, Validation, utils};
use crate::{GeoFloat, HasDimensions, LineString};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InvalidLineString {
    /// A valid [`LineString`] must have at least 2 distinct points to be valid - it must have a non-zero length.
    #[error("line string must have at least 2 distinct points")]
    TooFewPoints,
    /// A valid [`LineString`] must have finite coordinates.
    #[error("coordinate at index {} is non-finite", .0.0)]
    NonFiniteCoord(CoordIndex),
}

impl<F: GeoFloat> Validation for LineString<F> {
    type Error = InvalidLineString;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        if self.is_empty() {
            return Ok(());
        }

        // Perform the various checks
        if utils::check_too_few_points(self, false) {
            handle_validation_error(InvalidLineString::TooFewPoints)?;
        }

        for (coord_idx, coord) in self.0.iter().enumerate() {
            if utils::check_coord_is_not_finite(coord) {
                let err = InvalidLineString::NonFiniteCoord(CoordIndex(coord_idx));
                handle_validation_error(err)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};
    use crate::wkt;

    #[test]
    fn test_linestring_valid() {
        let ls = wkt!(LINESTRING(0. 0., 1. 1.));
        assert_valid!(&ls);
    }

    #[test]
    fn test_linestring_valid_empty() {
        let ls: LineString = wkt!(LINESTRING EMPTY);
        assert_valid!(&ls);
    }

    #[test]
    fn test_linestring_invalid_too_few_points_without_duplicate() {
        let ls = wkt!(LINESTRING(0. 0.));
        assert_validation_errors!(&ls, vec![InvalidLineString::TooFewPoints]);
    }

    #[test]
    fn test_linestring_invalid_too_few_points_with_duplicate() {
        let ls = wkt!(LINESTRING(0. 0.,0. 0.));
        assert_validation_errors!(&ls, vec![InvalidLineString::TooFewPoints]);
    }
}
