use super::{GeometryIndex, InvalidPoint, Validation};
use crate::{GeoFloat, MultiPoint};

/// A [`MultiPoint`] is valid if each [`Point`](crate::Point) in it is valid.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InvalidMultiPoint {
    /// Which element is invalid, and what was invalid about it.
    #[error("point at index {} is invalid: {}", .0.0, .1)]
    InvalidPoint(GeometryIndex, #[source] InvalidPoint),
}

impl<F: GeoFloat> Validation for MultiPoint<F> {
    type Error = InvalidMultiPoint;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        for (i, point) in self.0.iter().enumerate() {
            point.visit_validation(Box::new(&mut |invalid_point| {
                let err = InvalidMultiPoint::InvalidPoint(GeometryIndex(i), invalid_point);
                handle_validation_error(err)
            }))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};
    use crate::{geometry::*, wkt};

    #[test]
    fn test_multipoint_valid() {
        let mp = wkt!(MULTIPOINT(0. 0.,1. 1.));
        assert_valid!(&mp);
    }

    #[test]
    fn test_multipoint_invalid() {
        let mp = MultiPoint(vec![
            Point::new(0., f64::INFINITY),
            Point::new(f64::NAN, 1.),
        ]);
        assert_validation_errors!(
            &mp,
            vec![
                InvalidMultiPoint::InvalidPoint(GeometryIndex(0), InvalidPoint::NonFiniteCoord),
                InvalidMultiPoint::InvalidPoint(GeometryIndex(1), InvalidPoint::NonFiniteCoord)
            ]
        );
    }
}
