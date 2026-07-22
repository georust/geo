use super::{Validation, utils};
use crate::{Coord, GeoFloat};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum InvalidCoord {
    /// A valid [`Coord`] must be finite.
    #[error("coordinate was non-finite")]
    NonFinite,
}

impl<F: GeoFloat> Validation for Coord<F> {
    type Error = InvalidCoord;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        if utils::check_coord_is_not_finite(self) {
            handle_validation_error(InvalidCoord::NonFinite)?;
        }
        Ok(())
    }
}
