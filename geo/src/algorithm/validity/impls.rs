use super::*;

use geo_types::{MultiPolygon, Polygon};

use crate::GeoFloat;

/// Splits self intersecting polygons into individual valid polygons
///
/// Self intersecting here includes banana polygons
///
/// There are no real guarantees in terms of minimality we can give since the algorithms is a
/// heuristic. In practice it turned out to work quiet well though.
///
/// # Validity
///
/// The algo will return non-sense if the input is non-sense. This mainly means:
///
/// - a polygon including a weird, disconnected linestring
impl<F: GeoFloat> Validify for Polygon<F> {
    type ValidResult = MultiPolygon<F>;
    fn split_into_valid(&self) -> Self::ValidResult {
        let mp = MultiPolygon::new(vec![self.clone()]);
        split::split_invalid_multipolygon(&mp)
    }
}

impl<F: GeoFloat> Validify for MultiPolygon<F> {
    type ValidResult = MultiPolygon<F>;
    fn split_into_valid(&self) -> Self::ValidResult {
        split::split_invalid_multipolygon(self)
    }
}
