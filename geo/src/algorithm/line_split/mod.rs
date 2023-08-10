/// LineSplit Trait
mod line_split_trait;
pub use line_split_trait::LineSplit;

/// Implementations for LineSplit Trait for Line
mod line_split_trait_impl_for_line;

/// Implementations for LineSplit Trait for LineString
mod line_split_trait_impl_for_linestring;

/// Result types for LineSplit::line_split
mod line_split_result;
pub use line_split_result::LineSplitResult;

/// Result types for LineSplit::line_split_twice
mod line_split_twice_result;
pub use line_split_twice_result::LineSplitTwiceResult;

/// Helper function to measure the total length
/// of a LineString at the same time as the length of each segment
mod measure_line_string;
use measure_line_string::{measure_line_string, LineStringMeasurements};
