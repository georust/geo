#![allow(unused)]

mod point;
use point::SweepPoint;

mod events;
use events::{Event, EventType};

mod line_or_point;
use line_or_point::LineOrPoint;

mod cross;
use cross::Cross;

mod segment;
use segment::{Segment, SplitSegments};

mod active;
use active::{Active, ActiveSet};

mod im_segment;
use im_segment::IMSegment;

mod proc;
use proc::Sweep;

mod iter;
pub(crate) use iter::*;
