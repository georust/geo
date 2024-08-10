use num_traits::ToPrimitive;

use crate::{CoordFloat, CoordsIter, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect, Triangle};

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

impl DensifyGeodesic<f64> for MultiPolygon<f64>
where
    Line<f64>: GeodesicLength<f64>,
    LineString<f64>: GeodesicLength<f64>,
{
    type Output = MultiPolygon<f64>;

    fn densify_geodesic(&self, max_distance: f64) -> Self::Output {
        MultiPolygon::new(
            self.iter().map(|polygon| polygon.densify_geodesic(max_distance)).collect(),
        )
    }
}

impl DensifyGeodesic<f64> for Polygon<f64>
where
    Line<f64>: GeodesicLength<f64>,
    LineString<f64>: GeodesicLength<f64>,
{
    type Output = Polygon<f64>;

    fn densify_geodesic(&self, max_distance: f64) -> Self::Output {
        let densified_exterior = self.exterior().densify_geodesic(max_distance);
        let densified_interiors = self.interiors().iter().map(|ring| ring.densify_geodesic(max_distance)).collect();
        Polygon::new(densified_exterior, densified_interiors)
    }
}

impl DensifyGeodesic<f64> for MultiLineString<f64>
where
    Line<f64>: GeodesicLength<f64>,
    LineString<f64>: GeodesicLength<f64>,
{
    type Output = MultiLineString<f64>;

    fn densify_geodesic(&self, max_distance: f64) -> Self::Output {
        MultiLineString::new(
            self.iter().map(|linestring| linestring.densify_geodesic(max_distance)).collect(),
        )
    }
}

impl DensifyGeodesic<f64> for LineString<f64>
where
    Line<f64>: GeodesicLength<f64>,
    LineString<f64>: GeodesicLength<f64>,
{
    type Output = LineString<f64>;

    fn densify_geodesic(&self, max_distance: f64) -> Self::Output {
        if self.coords_count() == 0 {
            return LineString::new(vec![]);
        }

        let mut new_line = vec![];
        self.lines().for_each(|line| densify_line(line, &mut new_line, max_distance));
        // we're done, push the last coordinate on to finish
        new_line.push(self.points().last().unwrap());
        LineString::from(new_line)
    }
}

impl DensifyGeodesic<f64> for Line<f64>
where
    Line<f64>: GeodesicLength<f64>,
    LineString<f64>: GeodesicLength<f64>,
{
    type Output = LineString<f64>;

    fn densify_geodesic(&self, max_distance: f64) -> Self::Output {
        let mut new_line = vec![];
        densify_line(*self, &mut new_line, max_distance);
        // we're done, push the last coordinate on to finish
        new_line.push(self.end_point());
        LineString::from(new_line)
    }
}

impl DensifyGeodesic<f64> for Triangle<f64>
where
    Line<f64>: GeodesicLength<f64>,
    LineString<f64>: GeodesicLength<f64>,
{
    type Output = Polygon<f64>;

    fn densify_geodesic(&self, max_distance: f64) -> Self::Output {
        self.to_polygon().densify_geodesic(max_distance)
    }
}

impl DensifyGeodesic<f64> for Rect<f64>
where
    Line<f64>: GeodesicLength<f64>,
    LineString<f64>: GeodesicLength<f64>,
{
    type Output = Polygon<f64>;

    fn densify_geodesic(&self, max_distance: f64) -> Self::Output {
        self.to_polygon().densify_geodesic(max_distance)
    }
}