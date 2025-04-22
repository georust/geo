use super::{utils, Validation};
use crate::{GeoFloat, Point};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidPoint {
    /// A valid [`Point`] must be finite.
    NonFiniteCoord,
}

impl fmt::Display for InvalidPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidPoint::NonFiniteCoord => write!(f, "point has non-finite coordinates"),
        }
    }
}

impl std::error::Error for InvalidPoint {}

impl<F: GeoFloat> Validation for Point<F> {
    type Error = InvalidPoint;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        if utils::check_coord_is_not_finite(&self.0) {
            handle_validation_error(InvalidPoint::NonFiniteCoord)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};
    use crate::Point;

    #[test]
    fn test_point_valid() {
        let p = Point::new(0., 0.);
        assert_valid!(p);
    }

    #[test]
    fn test_point_validation_errors() {
        let p = Point::new(f64::NAN, f64::NAN);
        assert_validation_errors!(p, vec![InvalidPoint::NonFiniteCoord]);
    }

    #[test]
    fn test_point_check_validation() {
        let p = Point::new(f64::NAN, f64::NAN);

        let err = p.check_validation().unwrap_err();
        assert_eq!(err, InvalidPoint::NonFiniteCoord);
    }
}
