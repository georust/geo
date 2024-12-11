//! Provides a way to check the validity of geometries, based on the [OGC Simple Feature Access - Part 1: Common Architecture standard].
//!
//! [OGC Simple Feature Access - Part 1: Common Architecture standard]: https://www.ogc.org/standards/sfa
mod coord;
mod geometry;
mod geometry_collection;
mod line;
mod line_string;
mod multi_line_string;
mod multi_point;
mod multi_polygon;
mod point;
mod polygon;
mod rect;
#[cfg(test)]
mod tests;
mod triangle;
mod utils;

pub use geometry::InvalidGeometry;
pub use geometry_collection::InvalidGeometryCollection;
pub use line::InvalidLine;
pub use line_string::InvalidLineString;
pub use multi_line_string::InvalidMultiLineString;
pub use multi_point::InvalidMultiPoint;
pub use multi_polygon::InvalidMultiPolygon;
pub use point::InvalidPoint;
pub use polygon::InvalidPolygon;
pub use rect::InvalidRect;
pub use triangle::InvalidTriangle;

use std::boxed::Box;
use std::fmt;

/// A trait to check if a geometry is valid and report the reason(s) of invalidity.
///
/// ```
/// use geo::algorithm::Validation;
/// use geo::wkt;
///
/// let valid_polygon = wkt!(POLYGON((0. 0., 1. 1., 1. 0., 0. 0.)));
/// assert!(valid_polygon.is_valid());
///
/// let invalid_polygon = wkt!(POLYGON((0. 0., 1. 1.),(3. 3., 3. 4.,4. 4.)));
/// assert!(!invalid_polygon.is_valid());
///
/// // Get the first validation error, as a `Result`
/// let validation_error = invalid_polygon.check_validation().unwrap_err();
/// use geo::algorithm::validation::{InvalidPolygon, RingRole};
/// assert_eq!(validation_error, InvalidPolygon::TooFewPointsInRing(RingRole::Exterior));
///
/// // Get a human readable error
/// let text = validation_error.to_string();
/// assert_eq!(text, "exterior ring must have at least 3 distinct points");
//
/// // Get all validation errors
/// let all_validation_errors = invalid_polygon.validation_errors();
/// assert_eq!(all_validation_errors.len(), 2);
/// assert_eq!(all_validation_errors[0].to_string(), "exterior ring must have at least 3 distinct points");
/// assert_eq!(all_validation_errors[1].to_string(), "interior ring at index 0 is not contained within the polygon's exterior");
/// ```
pub trait Validation {
    type Error: std::error::Error;

    /// Check if the geometry is valid.
    fn is_valid(&self) -> bool {
        self.check_validation().is_ok()
    }

    /// Return the reason(s) of invalidity of the geometry.
    ///
    /// Though we try to return *all* problems with a geometry, it's possible that previous errors
    /// will obscure subsequent errors. For example, a MultiPolygon requires all its elements to be
    /// valid and non-overlapping. If one of the individual polygons is invalid, we can't guarantee
    /// the correctness of their "overlap" check which assumes valid input. Therefore, you should
    /// re-validate after attempting to correct any validation errors.
    fn validation_errors(&self) -> Vec<Self::Error> {
        let mut validation_errors = Vec::new();

        self.visit_validation(Box::new(|problem| {
            validation_errors.push(problem);
            Ok::<(), Self::Error>(())
        }))
        .expect("no errors are returned");

        validation_errors
    }

    /// Return the first reason of invalidity of the geometry.
    fn check_validation(&self) -> Result<(), Self::Error> {
        self.visit_validation(Box::new(Err))
    }

    /// Visit the validation of the geometry.
    ///
    /// The closure `handle_validation_error` is called for each validation error.
    fn visit_validation<T>(
        &self,
        handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T>;
}

/// The role of a ring in a [`Polygon`](crate::Polygon).
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RingRole {
    Exterior,
    Interior(usize),
}

impl fmt::Display for RingRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RingRole::Exterior => write!(f, "exterior ring"),
            RingRole::Interior(idx) => write!(f, "interior ring at index {}", idx),
        }
    }
}

/// The position of the problem in a multi-geometry, starting at 0.
#[derive(Debug, PartialEq, Clone)]
pub struct GeometryIndex(pub usize);

/// The index of the coordinate in the geometry
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CoordIndex(pub usize);

#[cfg(test)]
pub(crate) use test_macros::*;

#[cfg(test)]
mod test_macros {
    macro_rules! assert_valid {
        ($to_validate:expr) => {
            assert!(
                $to_validate.is_valid(),
                "Validation errors: {:?}",
                $to_validate.validation_errors()
            );
        };
    }
    pub(crate) use assert_valid;

    macro_rules! assert_validation_errors {
        ($to_validate:expr, $errors:expr) => {
            assert!(!$to_validate.is_valid());
            assert!(
                !$errors.is_empty(),
                "Use `assert_valid!` instead to verify there are no errors."
            );
            assert_eq!($errors, $to_validate.validation_errors());
        };
    }
    pub(crate) use assert_validation_errors;
}
