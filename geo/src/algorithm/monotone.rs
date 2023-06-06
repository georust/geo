mod monopoly;
pub use monopoly::MonoPoly;

mod segment;
use segment::RcSegment;
pub(crate) use segment::Segment;

mod sweep;
pub(crate) use sweep::SimpleSweep;

mod builder;
pub use builder::monotone_subdivision;

#[cfg(test)]
mod tests;
