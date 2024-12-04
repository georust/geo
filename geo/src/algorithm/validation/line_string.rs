use super::{utils, CoordIndex, Validation};
use crate::{GeoFloat, HasDimensions, LineString};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidLineString {
    /// A valid [`LineString`] must have at least 2 distinct points to be valid - it must have a non-zero length.
    TooFewPoints,
    /// A valid [`LineString`] must have finite coordinates.
    NonFiniteCoord(CoordIndex),
}

impl fmt::Display for InvalidLineString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidLineString::TooFewPoints => {
                write!(f, "line string must have at least 2 distinct points")
            }
            InvalidLineString::NonFiniteCoord(idx) => {
                write!(f, "coordinate at index {} is non-finite", idx.0)
            }
        }
    }
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
    use geos::Geom;

    #[test]
    fn test_linestring_valid() {
        let ls = wkt!(LINESTRING(0. 0., 1. 1.));
        assert_valid!(ls);

        // Test that the linestring has the same validity status than its GEOS equivalent
        let linestring_geos: geos::Geometry = (&ls).try_into().unwrap();
        assert_eq!(ls.is_valid(), linestring_geos.is_valid());
    }

    #[test]
    fn test_linestring_valid_empty() {
        let ls = wkt!(LINESTRING EMPTY);
        assert_valid!(ls);

        let linestring_geos: geos::Geometry = (&ls).try_into().unwrap();
        assert_eq!(ls.is_valid(), linestring_geos.is_valid());
    }

    #[test]
    fn test_linestring_invalid_too_few_points_without_duplicate() {
        let ls = wkt!(LINESTRING(0. 0.));
        assert_validation_errors!(ls, vec![InvalidLineString::TooFewPoints]);

        // Creating this linestring with geos fails (as soon as its creation is attempted)
        let linestring_geos: geos::GResult<geos::Geometry> = ls.try_into();
        assert!(linestring_geos.is_err());
    }

    #[test]
    fn test_linestring_invalid_too_few_points_with_duplicate() {
        let ls = wkt!(LINESTRING(0. 0.,0. 0.));
        assert_validation_errors!(ls, vec![InvalidLineString::TooFewPoints]);

        // Test that the linestring has the same validity status than its GEOS equivalent
        let linestring_geos: geos::Geometry = (&ls).try_into().unwrap();
        assert_eq!(ls.is_valid(), linestring_geos.is_valid());
    }
}
