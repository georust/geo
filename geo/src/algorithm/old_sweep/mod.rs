// NOTE: this module is private and for the use of the monotone module only
// For all other purposes it is DEPRECATED in favour of new_sweep

#![allow(dead_code)]

mod point;
pub use point::SweepPoint;

mod events;
pub(crate) use events::{Event, EventType};

mod line_or_point;
pub use line_or_point::LineOrPoint;

mod cross;
pub use cross::Cross;

mod segment;
use segment::{Segment, SplitSegments};

mod active;
pub(super) use active::{Active, ActiveSet};

mod im_segment;
use im_segment::IMSegment;

mod vec_set;
pub(crate) use vec_set::VecSet;

mod proc;
use proc::Sweep;

mod iter;
