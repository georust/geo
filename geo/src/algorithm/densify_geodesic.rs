use num_traits::ToPrimitive;

use crate::{
    CoordFloat, CoordsIter, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect,
    Triangle,
};

use crate::{GeodesicIntermediate, GeodesicLength};

/// Returns a new geometry on a spheroid containing both existing and new interpolated coordinates with
/// a maximum distance of `max_distance` between them.
///
/// Note: `max_distance` must be greater than 0.
///
/// ## Units
///
/// `max_distance`: meters
///
/// # Examples
/// ```
/// use approx::assert_relative_eq;
///
/// use geo::{coord, GeodesicLength, Line, LineString};
/// use geo::DensifyGeodesic;
///
/// let line = Line::new(coord! {x: 10.0, y: 20.0}, coord! { x: 125.0, y: 25.00 });
/// // known output
/// let output: LineString = vec![[10.0, 20.0], [65.879360, 37.722253], [125.0, 25.00]].into();
/// // densify
/// let dense = line.densify_geodesic(5703861.471800622);
///
/// assert_relative_eq!(dense, output, epsilon = 1.0e-6);
///```
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord;

    #[test]
    fn test_polygon_densify() {
        let exterior: LineString = vec![
            [4.925, 45.804],
            [4.732, 45.941],
            [4.935, 46.513],
            [5.821, 46.103],
            [5.627, 45.611],
            [5.355, 45.883],
            [4.925, 45.804],
        ].into();

        let polygon = Polygon::new(exterior, vec![]);

        let output_exterior: LineString = vec![
            [4.925, 45.804],
            [4.732, 45.941],
            [4.832972865149862, 46.22705224065524],
            [4.935, 46.513],
            [5.379653814979939, 46.30886184400083],
            [5.821, 46.103],
            [5.723572275808633, 45.85704648840237],
            [5.627, 45.611],
            [5.355, 45.883],
            [4.925, 45.804],
        ].into();

        let dense = polygon.densify_geodesic(50000.0);
        assert_relative_eq!(dense.exterior(), &output_exterior, epsilon = 1.0e-6);
    }

    #[test]
    fn test_linestring_densify() {
        let linestring: LineString = vec![
            [-3.202, 55.9471],
            [-3.2012, 55.9476],
            [-3.1994, 55.9476],
            [-3.1977, 55.9481],
            [-3.196, 55.9483],
            [-3.1947, 55.9487],
            [-3.1944, 55.9488],
            [-3.1944, 55.949],
        ].into();

        let output: LineString = vec![
            [-3.202, 55.9471],
            [-3.2012, 55.9476],
            [-3.2002999999999995, 55.94760000327935],
            [-3.1994, 55.9476],
            [-3.1985500054877773, 55.94785000292509],
            [-3.1977, 55.9481],
            [-3.196, 55.9483],
            [-3.1947, 55.9487],
            [-3.1944, 55.9488],
            [-3.1944, 55.949],
        ].into();

        let dense = linestring.densify_geodesic(110.0);
        assert_relative_eq!(dense, output, epsilon = 1.0e-6);
    }

    #[test]
    fn test_line_densify() {
        let output: LineString = vec![[10.0, 20.0], [65.879360, 37.722253], [125.0, 25.00]].into();
        let line = Line::new(coord! {x: 10.0, y: 20.0}, coord! { x: 125.0, y: 25.00 });
        let dense = line.densify_geodesic(5703861.471800622);
        assert_relative_eq!(dense, output, epsilon = 1.0e-6);
    }

    #[test]
    fn test_empty_linestring() {
        let linestring: LineString<f64> = LineString::new(vec![]);
        let dense = linestring.densify_geodesic(10.0);
        assert_eq!(0, dense.coords_count());
    }
}
