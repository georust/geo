use num_traits::ToPrimitive;

use crate::{CoordFloat, Line, Point};

use crate::{GeodesicIntermediate, GeodesicLength};

/// Returns a new geometry on a spheroid containing both existing and new interpolated coordinates with
/// a maximum distance of `max_distance` between them.
///
/// Note: `max_distance` must be greater than 0.
///
/// ## Units
///
/// `max_distance`: meters
pub trait DensifyGeodesic<F: CoordFloat> {
    type Output;

    fn densify_geodesic(&self, max_distance: F) -> Self::Output;
}

// Helper for densification trait
fn densify_line(
    line: Line<f64>,
    container: &mut Vec<Point<f64>>,
    max_distance: f64,
) {
    assert!(max_distance > 0.0);

    container.push(line.start_point());

    let num_segments = (line.geodesic_length() / max_distance).ceil().to_u64().unwrap();
    // distance "unit" for this line segment
    let frac = 1.0 / num_segments as f64;

    let start = line.start;
    let end = line.end;

    for segment_idx in 1..num_segments {
        let ratio = frac * segment_idx as f64;
        let interpolated_point = Point::from(start).geodesic_intermediate(&Point::from(end), ratio);
        container.push(interpolated_point);
    }
}