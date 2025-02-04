use super::{Distance, InterpolatePoint};
use crate::{
    CoordFloat, CoordsIter, Line, LineString, MultiLineString, MultiPolygon, Point, Polygon, Rect,
    Triangle,
};
use num_traits::FromPrimitive;

/// Creates a copy of the geometry with additional points inserted as necessary to ensure there
/// is never more than `max_segment_length` between points.
///
/// ## Units
/// - `max_segment_length` units depend on the implementing [metric space]. It must be greater than 0.
///
/// # Examples
///
/// ```
/// # use approx::assert_relative_eq;
/// use geo::{wkt, Densify};
/// use geo::line_measures::Euclidean;
///
/// let line_string = wkt!(LINESTRING(0.0 0.0,0.0 6.0,1.0 7.0));
///
/// // For Euclidean calculations, the unit of distance is the same as the units
/// // of your coordinates.
/// let max_dist = 2.0;
/// let densified = Euclidean.densify(&line_string, max_dist);
/// let expected_output = wkt!(LINESTRING(
///     0.0 0.0,
///     0.0 2.0,
///     0.0 4.0,
///     0.0 6.0,
///     1.0 7.0
/// ));
/// assert_relative_eq!(densified, expected_output);
///```
///
/// For lng/lat geometries, consider using a different [metric space] like [`Haversine`](crate::Haversine) or [`Geodesic`](crate::Geodesic).
///
///```
/// # use approx::assert_relative_eq;
/// use geo::{wkt, Densify};
/// use geo::line_measures::Haversine;
/// let line_string = wkt!(LINESTRING(0.0 0.0,0.0 6.0,1.0 7.0));
///
/// // For Haversine, the unit of distance is in meters
/// let max_dist = 200_000.0;
/// let densified = Haversine.densify(&line_string, max_dist);
/// // Haversine interprets coordinate points as lng/lat
/// let expected_output = wkt!(LINESTRING(
///     0.0 0.0,
///     0.0 1.5,
///     0.0 3.0,
///     0.0 4.5,
///     0.0 6.0,
///     1.0 7.0
/// ));
/// assert_relative_eq!(densified, expected_output, epsilon = 1e-14);
/// ```
/// [metric space]: crate::line_measures::metric_spaces
pub trait Densify<F: CoordFloat> {
    fn densify<D: Densifiable<F>>(&self, geometry: &D, max_segment_length: F) -> D::Output;
}

impl<F, MetricSpace> Densify<F> for MetricSpace
where
    F: CoordFloat,
    MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
{
    fn densify<D: Densifiable<F>>(&self, geometry: &D, max_segment_length: F) -> D::Output {
        geometry.densify(self, max_segment_length)
    }
}

/// Creates a copy of the geometry with additional points inserted as necessary to ensure there
/// is never more than `max_segment_length` between points.
///
/// It's typically more convenient to use the [`Densify`] trait instead of this trait directly.
///
/// ## Units
/// - `max_segment_length` units depend on the implementing [metric space]. It must be greater than 0.
///
/// # Examples
/// ```
/// # use approx::assert_relative_eq;
/// use geo::wkt;
/// use geo::line_measures::{Euclidean, Densifiable};
///
/// let line_string = wkt!(LINESTRING(0.0 0.0,0.0 6.0,1.0 7.0));
///
/// // For Euclidean calculations, the unit of distance is the same as the units
/// // of your coordinates.
/// let max_dist = 2.0;
/// let densified = line_string.densify(&Euclidean, max_dist);
/// let expected_output = wkt!(LINESTRING(
///     0.0 0.0,
///     0.0 2.0,
///     0.0 4.0,
///     0.0 6.0,
///     1.0 7.0
/// ));
/// assert_relative_eq!(densified, expected_output);
///```
///
/// For lng/lat geometries, consider using a different [metric space] like [`Haversine`](crate::Haversine) or [`Geodesic`](crate::Geodesic).
///
///```
/// # use approx::assert_relative_eq;
/// use geo::wkt;
/// use geo::line_measures::{Haversine, Densifiable};
/// let line_string = wkt!(LINESTRING(0.0 0.0,0.0 6.0,1.0 7.0));
///
/// // For Haversine, the unit of distance is in meters
/// let max_dist = 200_000.0;
/// let densified = line_string.densify(&Haversine, max_dist);
/// // Haversine interprets coordinate points as lng/lat
/// let expected_output = wkt!(LINESTRING(
///     0.0 0.0,
///     0.0 1.5,
///     0.0 3.0,
///     0.0 4.5,
///     0.0 6.0,
///     1.0 7.0
/// ));
/// assert_relative_eq!(densified, expected_output, epsilon = 1e-14);
/// ```
/// [metric space]: crate::line_measures::metric_spaces
pub trait Densifiable<F: CoordFloat> {
    type Output;
    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>;
}

pub(crate) fn densify_between<F, MetricSpace>(
    metric_space: &MetricSpace,
    line_start: Point<F>,
    line_end: Point<F>,
    container: &mut Vec<Point<F>>,
    max_segment_length: F,
) where
    F: CoordFloat + FromPrimitive,
    MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
{
    assert!(max_segment_length > F::zero());
    let num_segments = (metric_space.distance(line_start, line_end) / max_segment_length)
        .ceil()
        .to_u64()
        .expect("unreasonable number of segments");

    // distance "unit" for this line segment
    let frac = F::one() / F::from(num_segments).unwrap();

    for segment_num in 1..num_segments {
        let ratio = frac * F::from(segment_num).unwrap();

        // PERF TODO: We recompute "total_distance" every step of this loop.
        // If we impl point_at_distance_between, we could compute it once and use it here.
        // At that point, I think this function could be a good candidate to be *the single* basis
        // for a unified generic of points_along_line for all metric spaces.
        let interpolated_point = metric_space.point_at_ratio_between(line_start, line_end, ratio);
        container.push(interpolated_point);
    }
}

impl<F: CoordFloat + FromPrimitive> Densifiable<F> for Line<F> {
    type Output = LineString<F>;

    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        let mut points = vec![self.start_point()];
        densify_between(
            metric_space,
            self.start_point(),
            self.end_point(),
            &mut points,
            max_segment_length,
        );
        points.push(self.end_point());
        LineString::from(points)
    }
}

impl<F: CoordFloat + FromPrimitive> Densifiable<F> for LineString<F> {
    type Output = Self;

    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> LineString<F>
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        if self.coords_count() == 0 {
            return LineString::new(vec![]);
        }

        let mut points = vec![];
        self.lines().for_each(|line| {
            points.push(line.start_point());
            densify_between(
                metric_space,
                line.start_point(),
                line.end_point(),
                &mut points,
                max_segment_length,
            )
        });

        // we're done, push the last coordinate on to finish
        let final_coord = *self
            .0
            .last()
            .expect("we already asserted the line string is not empty");
        points.push(final_coord.into());

        LineString::from(points)
    }
}

impl<F: CoordFloat + FromPrimitive> Densifiable<F> for MultiLineString<F> {
    type Output = Self;

    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        MultiLineString::new(
            self.iter()
                .map(|line_string| line_string.densify(metric_space, max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densifiable<F> for Polygon<F> {
    type Output = Self;

    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        Polygon::new(
            self.exterior().densify(metric_space, max_segment_length),
            self.interiors()
                .iter()
                .map(|interior| interior.densify(metric_space, max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densifiable<F> for MultiPolygon<F> {
    type Output = Self;

    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        MultiPolygon::new(
            self.iter()
                .map(|polygon| polygon.densify(metric_space, max_segment_length))
                .collect(),
        )
    }
}

impl<F: CoordFloat + FromPrimitive> Densifiable<F> for Rect<F> {
    type Output = Polygon<F>;

    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        self.to_polygon().densify(metric_space, max_segment_length)
    }
}

impl<F: CoordFloat + FromPrimitive> Densifiable<F> for Triangle<F> {
    type Output = Polygon<F>;

    fn densify<MetricSpace>(
        &self,
        metric_space: &MetricSpace,
        max_segment_length: F,
    ) -> Self::Output
    where
        MetricSpace: Distance<F, Point<F>, Point<F>> + InterpolatePoint<F>,
    {
        self.to_polygon().densify(metric_space, max_segment_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, polygon, wkt, Euclidean, Geodesic, Haversine, Rhumb};

    #[test]
    fn densify_line() {
        // London to Paris
        let line = Line::new(
            coord!(x: -0.1278f64, y: 51.5074),
            coord!(x: 2.3522, y: 48.8566),
        );

        let densified_line = Geodesic.densify(&line, 100_000.0); // max segment length 100km
        assert!(densified_line.coords_count() > 2);

        let densified_rhumb = Rhumb.densify(&line, 100_000.0);
        assert!(densified_rhumb.coords_count() > 2);

        let densified_haversine = Haversine.densify(&line, 100_000.0);
        assert!(densified_haversine.coords_count() > 2);
    }

    #[test]
    fn densify_line_string() {
        let line_string = LineString::new(vec![
            coord!(x: -58.3816f64, y: -34.6037), // Buenos Aires, Argentina
            coord!(x: -77.0428, y: -12.0464),    // Lima, Peru
            coord!(x: -47.9292, y: -15.7801),    // Brasília, Brazil
        ]);

        let densified_ls = Geodesic.densify(&line_string, 500_000.0); // 500 km max segment length
        assert!(densified_ls.coords_count() > line_string.coords_count());

        let densified_rhumb_ls = Rhumb.densify(&line_string, 500_000.0);
        assert!(densified_rhumb_ls.coords_count() > line_string.coords_count());

        let densified_haversine_ls = Haversine.densify(&line_string, 500_000.0);
        assert!(densified_haversine_ls.coords_count() > line_string.coords_count());
    }

    #[test]
    fn densify_polygon() {
        let polygon = polygon![
            (x: -58.3816f64, y: -34.6037), // Buenos Aires
            (x: -77.0428, y: -12.0464),    // Lima
            (x: -47.9292, y: -15.7801),    // Brasília
        ];

        let densified_polygon = Geodesic.densify(&polygon, 500_000.0); // 500 km max segment length
        assert!(densified_polygon.exterior().coords_count() > polygon.exterior().coords_count());
    }

    // ported from the old Deprecated trait, which only worked with Euclidean measures
    mod euclidean {
        use super::*;

        #[test]
        fn test_polygon_densify() {
            let polygon = wkt!(POLYGON(
                (-5.0 0.0,0.0 5.0,5.0 0.0,-5.0 0.0),
                (-3.0 0.0,0.0 3.0,3.0 0.0,-3.0 0.0)
            ));

            let expected = wkt!(POLYGON(
                (-5.0 0.0,-3.75 1.25,-2.5 2.5,-1.25 3.75,0.0 5.0,1.25 3.75,2.5 2.5,3.75 1.25,5.0 0.0,3.0 0.0,1.0 0.0,-1.0000000000000009 0.0,-3.0 0.0, -5.0 0.0),
                (-3.0 0.0,-2.0 1.0,-1.0 2.0,0.0 3.0,1.0 2.0,2.0 1.0,3.0 0.0,1.0 0.0,-1.0 0.0,-3.0 0.0)
            ));

            let max_dist = 2.0;
            let densified = Euclidean.densify(&polygon, max_dist);
            assert_eq!(densified, expected);
        }

        #[test]
        fn test_empty_linestring_densify() {
            let linestring = LineString::<f64>::new(vec![]);
            let max_dist = 2.0;
            let densified = Euclidean.densify(&linestring, max_dist);
            assert!(densified.0.is_empty());
        }

        #[test]
        fn test_linestring_densify() {
            let linestring = wkt!(LINESTRING(
               -1.0 0.0,
                0.0 0.0,
                0.0 6.0,
                1.0 8.0
            ));
            let expected = wkt!(LINESTRING(
               -1.0 0.0,
                0.0 0.0,
                0.0 2.0,
                0.0 4.0,
                0.0 6.0,
                0.5 7.0,
                1.0 8.0
            ));
            let max_dist = 2.0;
            let densified = Euclidean.densify(&linestring, max_dist);
            assert_eq!(densified, expected);
        }

        #[test]
        fn test_line_densify() {
            let line: Line<f64> = Line::new(coord! {x: 0.0, y: 6.0}, coord! {x: 1.0, y: 8.0});
            let correct: LineString<f64> = vec![[0.0, 6.0], [0.5, 7.0], [1.0, 8.0]].into();
            let max_dist = 2.0;
            let densified = Euclidean.densify(&line, max_dist);
            assert_eq!(densified, correct);
        }
    }

    // ported from the now deprecated DensifyHaversine
    mod lon_lat_tests {
        use super::*;

        #[test]
        fn test_polygon_densify() {
            let polygon = wkt!(POLYGON((
                4.925 45.804,
                4.732 45.941,
                4.935 46.513,
                5.821 46.103,
                5.627 45.611,
                5.355 45.883,
                4.925 45.804
            )));

            let exepcted_haversine = wkt!(POLYGON((
                4.925 45.804,
                4.732 45.941,
                4.8329711649985505 46.2270449096239,
                4.935 46.513,
                5.379659133344039 46.30885540136222,
                5.821 46.103,
                5.723570877658867 45.85704103535437,
                5.627 45.611,
                5.355 45.883,
                4.925 45.804
            )));

            let actual_haversine = Haversine.densify(&polygon, 50000.0);
            assert_relative_eq!(actual_haversine, exepcted_haversine);

            let expected_geodesic = wkt!(POLYGON((
                4.925 45.804,
                4.732 45.941,
                4.832972865149862 46.22705224065524,
                4.935 46.513,
                5.379653814979939 46.30886184400083,
                5.821 46.103,
                5.723572275808633 45.85704648840237,
                5.627 45.611,
                5.355 45.883,
                4.925 45.804
            )));
            let actual_geodesic = Geodesic.densify(&polygon, 50000.0);
            assert_relative_eq!(actual_geodesic, expected_geodesic);
        }

        #[test]
        fn test_linestring_densify() {
            let linestring = wkt!(LINESTRING(
                -3.202 55.9471,
                -3.2012 55.9476,
                -3.1994 55.9476,
                -3.1977 55.9481,
                -3.196 55.9483,
                -3.1947 55.9487,
                -3.1944 55.9488,
                -3.1944 55.949
            ));

            let expected = wkt!(LINESTRING(
                -3.202 55.9471,
                -3.2012 55.9476,
                -3.2002999999999995 55.94760000327935,
                -3.1994 55.9476,
                -3.1985500054877773 55.94785000292509,
                -3.1977 55.9481,
                -3.196 55.9483,
                -3.1947 55.9487,
                -3.1944 55.9488,
                -3.1944 55.949
            ));

            let dense = Haversine.densify(&linestring, 110.0);
            assert_relative_eq!(dense, expected);
        }

        #[test]
        fn test_line_densify() {
            let output = wkt!(LINESTRING(0.0 0.0, 0.0 0.5, 0.0 1.0));
            let line = Line::new(coord! {x: 0.0, y: 0.0}, coord! { x: 0.0, y: 1.0 });
            let dense = Haversine.densify(&line, 100000.0);
            assert_relative_eq!(dense, output);
        }
    }

    mod degenerate {
        use super::*;

        #[test]
        fn test_empty_linestring() {
            let input = wkt!(LINESTRING EMPTY);
            let dense = Euclidean.densify(&input, 1.0);
            assert_eq!(0, dense.coords_count());
            assert_eq!(input, dense);
        }

        #[test]
        fn test_one_point_linestring() {
            let input = wkt!(LINESTRING(1.0 1.0));
            let dense = Euclidean.densify(&input, 1.0);
            assert_eq!(1, dense.coords_count());
            assert_eq!(input, dense);
        }
    }
}
