
mod vector_extensions;
mod slice_itertools;
mod line_intersection;
mod offset_segments_iterator;
mod offset_line_raw;
mod line_measured;

// Kept temporarily during transition to new approach
mod offset_curve_trait_old;
use offset_curve_trait_old::OffsetCurveOld;

mod offset_curve_trait;
pub use offset_curve_trait::OffsetCurve;
