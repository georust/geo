//! Monotone Chains are a way of partitioning the segments of a linestring to allow for fast searching of intersections. They have the following properties:
//!
//! 1. the segments within a monotone chain never intersect each other
//! 2. the envelope of any contiguous subset of the segments in a monotone chain is equal to the envelope of the endpoints of the subset.
//!
//! Property 1 means that there is no need to test pairs of segments from within the same monotone chain for intersection.  
//! Property 2 allows an efficient binary search to be used to find the intersection points of two monotone chains. For many types of real-world data, these properties eliminate a large number of segment comparisons, producing substantial speed gains.  
//!
//! This module provides geometries backed by monotone chains.
//!

// primitives
mod segment;
pub(crate) use segment::MonotoneChainSegment;
mod chain;
pub use chain::MonotoneChain;

// geometries
pub(crate) mod geometry;
pub use geometry::{
    MonotoneChainGeometry, MonotoneChainLineString, MonotoneChainMultiLineString,
    MonotoneChainMultiPolygon, MonotoneChainPolygon,
};

// iterators
mod chain_iter;
pub use chain_iter::MonotoneChains;

// debug utils
mod util;
