use super::{Distance, InterpolatePoint, Length};
use crate::{
    CoordFloat, CoordsIter, HaversineIntermediate, HaversineLength, Line, LineString,
    MultiLineString, MultiPolygon, Point, Polygon, Rect, Triangle,
};
use num_traits::FromPrimitive;

pub trait Densify<F: CoordFloat> {
    type Output;
    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>;
}

// TODO: We have an opportunity for further cleanup in the codebase.
// I think this implementation could be *the* generic implementation for *all* the intermediate_fill implementations.
fn densify_line<F, MetricSpace>(
    line: &Line<F>,
    container: &mut Vec<Point<F>>,
    max_segment_length: F,
) where
    F: CoordFloat + FromPrimitive,
    MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
{
    assert!(max_segment_length > F::zero());

    container.push(line.start_point());

    let num_segments = (line.length::<MetricSpace>() / max_segment_length)
        .ceil()
        .to_u64()
        .expect("unreasonable number of segments");

    // distance "unit" for this line segment
    let frac = F::one() / F::from(num_segments).unwrap();

    for segment_idx in 1..num_segments {
        let ratio = frac * F::from(segment_idx).unwrap();
        let start = line.start;
        let end = line.end;

        let interpolated_point =
            MetricSpace::point_at_ratio_between(start.into(), end.into(), ratio);
        container.push(interpolated_point);
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Line<F> {
    type Output = LineString<F>;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        let mut new_line = vec![];
        densify_line::<F, MetricSpace>(self, &mut new_line, max_segment_length);
        LineString::from(new_line)
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for LineString<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> LineString<F>
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        if self.coords_count() == 0 {
            return LineString::new(vec![]);
        }

        let mut new_line = vec![];
        self.lines().for_each(|line| {
            densify_line::<F, MetricSpace>(&line, &mut new_line, max_segment_length)
        });

        // we're done, push the last coordinate on to finish
        new_line.push(self.points().last().unwrap());
        LineString::from(new_line)
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for MultiLineString<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        MultiLineString::new(
            self.iter()
                .map(|line_string| line_string.densify::<MetricSpace>(max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Polygon<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        Polygon::new(
            self.exterior().densify::<MetricSpace>(max_segment_length),
            self.interiors()
                .iter()
                .map(|interior| interior.densify::<MetricSpace>(max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for MultiPolygon<F> {
    type Output = Self;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        MultiPolygon::new(
            self.iter()
                .map(|polygon| polygon.densify::<MetricSpace>(max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Rect<F> {
    type Output = Polygon<F>;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        self.to_polygon().densify::<MetricSpace>(max_segment_length)
    }
}

impl<F: CoordFloat + FromPrimitive> Densify<F> for Triangle<F> {
    type Output = Polygon<F>;

    fn densify<MetricSpace>(&self, max_segment_length: F) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        self.to_polygon().densify::<MetricSpace>(max_segment_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, polygon, Geodesic, Haversine, Rhumb};

    #[test]
    fn densify_line() {
        // London to Paris
        let line = Line::new(
            coord!(x: -0.1278f64, y: 51.5074),
            coord!(x: 2.3522, y: 48.8566),
        );

        let densified_line = line.densify::<Geodesic>(100_000.0); // max segment length 100km
        assert!(densified_line.coords_count() > 2);

        let densified_rhumb = line.densify::<Rhumb>(100_000.0);
        assert!(densified_rhumb.coords_count() > 2);

        let densified_haversine = line.densify::<Haversine>(100_000.0);
        assert!(densified_haversine.coords_count() > 2);
    }

    #[test]
    fn densify_line_string() {
        let line_string = LineString::new(vec![
            coord!(x: -58.3816f64, y: -34.6037), // Buenos Aires, Argentina
            coord!(x: -77.0428, y: -12.0464),    // Lima, Peru
            coord!(x: -47.9292, y: -15.7801),    // Brasília, Brazil
        ]);

        let densified_ls = line_string.densify::<Geodesic>(500_000.0); // 500 km max segment length
        assert!(densified_ls.coords_count() > line_string.coords_count());

        let densified_rhumb_ls = line_string.densify::<Rhumb>(500_000.0);
        assert!(densified_rhumb_ls.coords_count() > line_string.coords_count());

        let densified_haversine_ls = line_string.densify::<Haversine>(500_000.0);
        assert!(densified_haversine_ls.coords_count() > line_string.coords_count());
    }

    #[test]
    fn densify_polygon() {
        let polygon = polygon![
            (x: -58.3816f64, y: -34.6037), // Buenos Aires
            (x: -77.0428, y: -12.0464),    // Lima
            (x: -47.9292, y: -15.7801),    // Brasília
        ];

        let densified_polygon = polygon.densify::<Geodesic>(500_000.0); // 500 km max segment length
        assert!(densified_polygon.exterior().coords_count() > polygon.exterior().coords_count());
    }
}
