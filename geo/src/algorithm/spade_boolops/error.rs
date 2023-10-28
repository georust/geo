use crate::stitch::LineStitchingError;
use crate::triangulate_spade::TriangulationError;
use geo_types::MultiPolygon;

#[derive(Debug)]
pub enum SpadeBoolopsError {
    TriangulationError(TriangulationError),
    StitchError(LineStitchingError),
}

impl std::fmt::Display for SpadeBoolopsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SpadeBoolopsError {}

pub type SpadeBoolopsResult<T> = Result<MultiPolygon<T>, SpadeBoolopsError>;
