use crate::Coordinate;
use crate::GeoFloat;

#[derive(Debug)]
pub(crate) struct Segment<F: GeoFloat + rstar::RTreeNum> {
    pub edge_idx: usize,
    pub segment_idx: usize,
    pub envelope: rstar::AABB<Coordinate<F>>,
}

impl<F> Segment<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    pub fn new(edge_idx: usize, segment_idx: usize, p1: Coordinate<F>, p2: Coordinate<F>) -> Self {
        use crate::rstar::RTreeObject;
        Self {
            edge_idx,
            segment_idx,
            envelope: rstar::AABB::from_corners(p1, p2),
        }
    }
}

impl<'a, F> rstar::RTreeObject for Segment<F>
where
    F: GeoFloat + rstar::RTreeNum,
{
    type Envelope = rstar::AABB<Coordinate<F>>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}
