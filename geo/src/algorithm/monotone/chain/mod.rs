mod monotone_chain;
pub use monotone_chain::MonotoneChain;

mod monotone_segment;
pub(crate) use monotone_segment::MonotoneChainSegment;
use monotone_segment::MonotoneChainSegmentFactory;

mod chain_iter;
pub use chain_iter::MonotoneChainIter;

mod intersects;

mod util;
