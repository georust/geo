use crate::CoordFloat;

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