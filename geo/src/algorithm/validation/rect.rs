use super::{CoordIndex, Validation, utils};
use crate::{GeoFloat, Rect};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InvalidRect {
    /// A valid [`Rect`] must have finite coordinates.
    /// Index `0` means the min coordinate, index `1` means the max coordinate.
    #[error("coordinate at rect's {} is non-finite", if .0.0 == 0 { "min" } else { "max" })]
    NonFiniteCoord(CoordIndex),
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
