use super::Distance;
use crate::{CoordFloat, Line, LineString, MultiLineString, Point};

/// Calculate the length of a `Line`, `LineString`, or `MultiLineString` using a given [metric space](crate::algorithm::line_measures::metric_spaces).
///
/// # Examples
/// ```
/// use geo::algorithm::line_measures::{Length, Euclidean, Haversine};
///
/// let line_string = geo::wkt!(LINESTRING(
///     0.0 0.0,
///     3.0 4.0,
///     3.0 5.0
/// ));
/// assert_eq!(Euclidean.length(&line_string), 6.);
///
/// let line_string_lon_lat = geo::wkt!(LINESTRING (
///     -47.9292 -15.7801f64,
///     -58.4173 -34.6118,
///     -70.6483 -33.4489
/// ));
/// assert_eq!(Haversine.length(&line_string_lon_lat).round(), 3_474_956.0);
/// ```
pub trait Length<F: CoordFloat> {
    fn length(&self, geometry: &impl LengthMeasurable<F>) -> F;
}

/// Something which can be measured by a [metric space](crate::algorithm::line_measures::metric_spaces),
/// such as a `Line`, `LineString`, or `MultiLineString`.
///
/// It's typically more convenient to use the [`Length`] trait instead of this trait directly.
///
/// # Examples
/// ```
/// use geo::algorithm::line_measures::{LengthMeasurable, Euclidean, Haversine};
///
/// let line_string = geo::wkt!(LINESTRING(
///     0.0 0.0,
///     3.0 4.0,
///     3.0 5.0
/// ));
/// assert_eq!(line_string.length(&Euclidean), 6.);
///
/// let line_string_lon_lat = geo::wkt!(LINESTRING (
///     -47.9292 -15.7801f64,
///     -58.4173 -34.6118,
///     -70.6483 -33.4489
/// ));
/// assert_eq!(line_string_lon_lat.length(&Haversine).round(), 3_474_956.0);
/// ```
pub trait LengthMeasurable<F: CoordFloat> {
    fn length(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F;
}

impl<F: CoordFloat, PointDistance: Distance<F, Point<F>, Point<F>>> Length<F> for PointDistance {
    fn length(&self, geometry: &impl LengthMeasurable<F>) -> F {
        geometry.length(self)
    }
}

impl<F: CoordFloat> LengthMeasurable<F> for Line<F> {
    fn length(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        metric_space.distance(self.start_point(), self.end_point())
    }
}

impl<F: CoordFloat> LengthMeasurable<F> for LineString<F> {
    fn length(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        let mut length = F::zero();
        for line in self.lines() {
            length = length + line.length(metric_space);
        }
        length
    }
}

impl<F: CoordFloat> LengthMeasurable<F> for MultiLineString<F> {
    fn length(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        let mut length = F::zero();
        for line in self {
            length = length + line.length(metric_space);
        }
        length
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord, Euclidean, Geodesic, Haversine, Rhumb};

    #[test]
    fn lines() {
        // london to paris
        let line = Line::new(
            coord!(x: -0.1278f64, y: 51.5074),
            coord!(x: 2.3522, y: 48.8566),
        );

        assert_eq!(
            343_923., // meters
            Geodesic.length(&line).round()
        );
        assert_eq!(
            343_572., // meters
            Rhumb.length(&line).round()
        );
        assert_eq!(
            343_557., // meters
            Haversine.length(&line).round()
        );

        // computing Euclidean length of an unprojected (lng/lat) line gives a nonsense answer
        assert_eq!(
            4., // nonsense!
            Euclidean.length(&line).round()
        );
        // london to paris in EPSG:3035
        let projected_line = Line::new(
            coord!(x: 3620451.74f64, y: 3203901.44),
            coord!(x: 3760771.86, y: 2889484.80),
        );
        assert_eq!(344_307., Euclidean.length(&projected_line).round());
    }

    #[test]
    fn line_strings() {
        let line_string = LineString::new(vec![
            coord!(x: -58.3816f64, y: -34.6037), // Buenos Aires, Argentina
            coord!(x: -77.0428, y: -12.0464),    // Lima, Peru
            coord!(x: -47.9292, y: -15.7801),    // Brasília, Brazil
        ]);

        assert_eq!(
            6_302_220., // meters
            Geodesic.length(&line_string).round()
        );
        assert_eq!(
            6_308_683., // meters
            Rhumb.length(&line_string).round()
        );
        assert_eq!(
            6_304_387., // meters
            Haversine.length(&line_string).round()
        );

        // computing Euclidean length of an unprojected (lng/lat) gives a nonsense answer
        assert_eq!(
            59., // nonsense!
            Euclidean.length(&line_string).round()
        );
        // EPSG:102033
        let projected_line_string = LineString::from(vec![
            coord!(x: 143042.46f64, y: -1932485.45), // Buenos Aires, Argentina
            coord!(x: -1797084.08, y: 583528.84),    // Lima, Peru
            coord!(x: 1240052.27, y: 207169.12),     // Brasília, Brazil
        ]);
        assert_eq!(6_237_538., Euclidean.length(&projected_line_string).round());
    }
}
