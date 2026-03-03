mod edge_set_intersector;
mod rstar_edge_set_intersector;
mod segment;
mod segment_intersector;
mod simple_edge_set_intersector;
#[deprecated(
    since = "0.31.1",
    note = "PreparedGeometry has moved to geo::indexed::PreparedGeometry"
)]
pub use crate::indexed::PreparedGeometry;
pub(crate) use edge_set_intersector::EdgeSetIntersector;

#[deprecated(
    since = "0.31.1",
    note = "prepare_geometry has moved to geo::indexed::prepared_geometry::prepare_geometry"
)]
pub(crate) use crate::indexed::prepared_geometry::prepare_geometry;
pub(crate) use rstar_edge_set_intersector::RStarEdgeSetIntersector;
pub(crate) use segment::Segment;
pub(crate) use segment_intersector::SegmentIntersector;
pub(crate) use simple_edge_set_intersector::SimpleEdgeSetIntersector;
