use super::{CoordIndex, Validation, utils};
use crate::{CoordFloat, Triangle};

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum InvalidTriangle {
    /// A valid [`Triangle`] must have finite coordinates.
    NonFiniteCoord(CoordIndex),
    /// A valid [`Triangle`] must have distinct points.
    IdenticalCoords(CoordIndex, CoordIndex),
    /// A valid [`Triangle`] must have non-collinear points.
    CollinearCoords,
}

impl fmt::Display for InvalidTriangle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidTriangle::NonFiniteCoord(idx) => {
                write!(f, "coordinate at index {} is non-finite", idx.0)
            }
            InvalidTriangle::IdenticalCoords(idx1, idx2) => {
                write!(
                    f,
                    "coordinates at indices {} and {} are identical",
                    idx1.0, idx2.0
                )
            }
            InvalidTriangle::CollinearCoords => write!(f, "triangle has collinear coordinates"),
        }
    }
}

impl std::error::Error for InvalidTriangle {}

impl<F: CoordFloat> Validation for Triangle<F> {
    type Error = InvalidTriangle;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        let v1 = self.v1();
        let v2 = self.v2();
        let v3 = self.v3();

        if utils::check_coord_is_not_finite(&v1) {
            handle_validation_error(InvalidTriangle::NonFiniteCoord(CoordIndex(0)))?;
        }
        if utils::check_coord_is_not_finite(&v2) {
            handle_validation_error(InvalidTriangle::NonFiniteCoord(CoordIndex(1)))?;
        }
        if utils::check_coord_is_not_finite(&v3) {
            handle_validation_error(InvalidTriangle::NonFiniteCoord(CoordIndex(2)))?;
        }

        // We wont check if the points are collinear if they are identical
        let mut identical = false;

        if v1 == v2 {
            handle_validation_error(InvalidTriangle::IdenticalCoords(
                CoordIndex(0),
                CoordIndex(1),
            ))?;
            identical = true;
        }
        if v1 == v3 {
            handle_validation_error(InvalidTriangle::IdenticalCoords(
                CoordIndex(0),
                CoordIndex(2),
            ))?;
            identical = true;
        }
        if v2 == v3 {
            handle_validation_error(InvalidTriangle::IdenticalCoords(
                CoordIndex(1),
                CoordIndex(2),
            ))?;
            identical = true;
        }

        if !identical && utils::robust_check_points_are_collinear::<F>(&v1, &v2, &v3) {
            handle_validation_error(InvalidTriangle::CollinearCoords)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};

    #[test]
    fn test_triangle_valid() {
        let t = Triangle::new((0., 0.).into(), (0., 1.).into(), (0.5, 2.).into());
        assert_valid!(t);
    }

    #[test]
    fn test_triangle_invalid_same_points() {
        let t = Triangle::new((0., 0.).into(), (0., 1.).into(), (0., 1.).into());
        assert_validation_errors!(
            t,
            vec![InvalidTriangle::IdenticalCoords(
                CoordIndex(1),
                CoordIndex(2)
            )]
        );
    }

    #[test]
    fn test_triangle_invalid_points_collinear() {
        let t = Triangle::new((0., 0.).into(), (1., 1.).into(), (2., 2.).into());
        assert_validation_errors!(t, vec![InvalidTriangle::CollinearCoords]);
    }
}
