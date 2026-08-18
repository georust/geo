use crate::Coord;
use crate::GeoFloat;

/// A line segment stored in an [`rstar::RTree`], carrying a caller-defined
/// payload that identifies the segment within its source collection.
///
/// The default payload is the geomgraph convention: `(edge_idx, segment_idx)`
/// into a graph's edge list.
#[derive(Debug, Clone)]
pub(crate) struct Segment<F: GeoFloat + rstar::RTreeNum, P = (usize, usize)> {
    pub payload: P,
    pub envelope: rstar::AABB<Coord<F>>,
}

impl<F, P> Segment<F, P>
where
    F: GeoFloat + rstar::RTreeNum,
{
    pub fn new(payload: P, p1: Coord<F>, p2: Coord<F>) -> Self {
        Self {
            payload,
            envelope: rstar::AABB::from_corners(p1, p2),
        }
    }
}

impl<F, P> rstar::RTreeObject for Segment<F, P>
where
    F: GeoFloat + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Coord<F>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}
