
use crate::{Line, CoordNum};

// Note: Previously I had a struct called "OffsetLineRaw" which turned out to
// be a line and it's length. This new struct [LineMeasured] is basically the
// same, but
// - has wider use cases
// - has a name that communicate's it's content better

/// A struct containing a [Line] and it's precalculated length.
/// 
/// TODO: I always Assume that it is faster to store calculated lengths than
/// simply recalculate them every time they are needed. This is likely the case
/// if the length is recalculated many times for the same Line. But in practice
/// there may be no benefit if it is only used once or twice in the same
/// function, where the compiler might have made the same optimization on our
/// behalf
/// 
/// TODO: I would like to mark this type as immutable so that it can only be
/// instantiated using the full struct constructor and never mutated. Apparently
/// that isn't possible in rust without making all members private. This is sad
/// because we give up nice destructuring syntax if we make members private. For
/// the time being, I am leaving members public. I think ultimately this type
/// might go away in refactoring and integrating with other parts of the geo
/// crate.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(super)struct LineMeasured <T> where T:CoordNum {
    pub line:Line<T>,
    pub length:T
}