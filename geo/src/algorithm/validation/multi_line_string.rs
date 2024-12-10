use super::{GeometryIndex, Validation};
use crate::algorithm::validation::line_string::InvalidLineString;
use crate::{GeoFloat, MultiLineString};

use std::fmt;

/// A [`MultiLineString`] is valid if each [`LineString`](crate::LineString) in it is valid.
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidMultiLineString {
    /// Which element is invalid, and what was invalid about it.
    InvalidLineString(GeometryIndex, InvalidLineString),
}

impl fmt::Display for InvalidMultiLineString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidMultiLineString::InvalidLineString(idx, err) => {
                write!(f, "line string at index {} is invalid: {}", idx.0, err)
            }
        }
    }
}

impl std::error::Error for InvalidMultiLineString {}

impl<F: GeoFloat> Validation for MultiLineString<F> {
    type Error = InvalidMultiLineString;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        for (i, line_string) in self.0.iter().enumerate() {
            line_string.visit_validation(Box::new(&mut |line_string_err| {
                let err =
                    InvalidMultiLineString::InvalidLineString(GeometryIndex(i), line_string_err);
                handle_validation_error(err)
            }))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{
        assert_valid, assert_validation_errors, InvalidLineString, InvalidMultiLineString,
    };
    use crate::wkt;

    #[test]
    fn test_multilinestring_valid() {
        let mls = wkt!(
            MULTILINESTRING(
                (0. 0.,1. 1.),
                (3. 1.,4. 1.)
            )
        );
        assert_valid!(&mls);
    }

    #[test]
    fn test_multilinestring_invalid_too_few_points_with_duplicate() {
        // The second LineString (at position 1) of this MultiLineString
        // is not valid because it has only one (deduplicated) point
        let mls = wkt!(
            MULTILINESTRING(
                (0. 0.,1. 1.),
                (0. 0.,0. 0.)
            )
        );
        assert_validation_errors!(
            &mls,
            vec![InvalidMultiLineString::InvalidLineString(
                GeometryIndex(1),
                InvalidLineString::TooFewPoints
            )]
        );
    }
}
