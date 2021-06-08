use crate::{Coordinate, GeoFloat};

/// Represents a point on an edge which intersects with another edge.
///
/// The intersection may either be a single point, or a line segment (in which case this point is
/// the start of the line segment) The intersection point must be precise.
///
/// This is based on [JTS's EdgeIntersection as of 1.18.1](https://github.com/locationtech/jts/blob/jts-1.18.1/modules/core/src/main/java/org/locationtech/jts/geomgraph/EdgeIntersection.java)
#[derive(Debug)]
pub(crate) struct EdgeIntersection<F: GeoFloat> {
    coord: Coordinate<F>,
    segment_index: usize,
    dist: F,
}

impl<F: GeoFloat> EdgeIntersection<F> {
    pub fn new(coord: Coordinate<F>, segment_index: usize, dist: F) -> EdgeIntersection<F> {
        EdgeIntersection {
            coord,
            segment_index,
            dist,
        }
    }

    pub fn coordinate(&self) -> Coordinate<F> {
        self.coord
    }

    pub fn segment_index(&self) -> usize {
        self.segment_index
    }

    pub fn distance(&self) -> F {
        self.dist
    }
}

impl<F: GeoFloat> std::cmp::PartialEq for EdgeIntersection<F> {
    fn eq(&self, other: &EdgeIntersection<F>) -> bool {
        self.segment_index == other.segment_index && self.dist == other.dist
    }
}

impl<F: GeoFloat> std::cmp::Eq for EdgeIntersection<F> {}

impl<F: GeoFloat> std::cmp::PartialOrd for EdgeIntersection<F> {
    fn partial_cmp(&self, other: &EdgeIntersection<F>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<F: GeoFloat> std::cmp::Ord for EdgeIntersection<F> {
    fn cmp(&self, other: &EdgeIntersection<F>) -> std::cmp::Ordering {
        if self.segment_index < other.segment_index {
            return std::cmp::Ordering::Less;
        }
        if self.segment_index > other.segment_index {
            return std::cmp::Ordering::Greater;
        }
        if self.dist < other.dist {
            return std::cmp::Ordering::Less;
        }
        if self.dist > other.dist {
            return std::cmp::Ordering::Greater;
        }

        // BTreeMap requires nodes to be fully `Ord`, but we're comparing floats, so we require
        // non-NaN for valid results.
        debug_assert!(!self.dist.is_nan() && !other.dist.is_nan());

        std::cmp::Ordering::Equal
    }
}

impl<F: GeoFloat> EdgeIntersection<F> {}
