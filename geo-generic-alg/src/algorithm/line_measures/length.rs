use super::Distance;
use crate::{CoordFloat, Line, LineString, MultiLineString, Point};
use geo_traits_ext::*;

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

/// Extension trait that enables the modern Length API for WKB and other generic geometry types.
///
/// This provides the same API as the concrete `LengthMeasurable` implementations but works with
/// any geometry type that implements the geo-traits-ext pattern.
///
/// # Examples
/// ```
/// use geo_generic_alg::algorithm::line_measures::{LengthMeasurableExt, Euclidean};
///
/// // Works with WKB geometries
/// let wkb_geom = geo_generic_tests::wkb::reader::read_wkb(&wkb_bytes).unwrap();
/// let length = wkb_geom.length_ext(&Euclidean);
/// ```
pub trait LengthMeasurableExt<F: CoordFloat> {
    /// Calculate the length using the given metric space.
    fn length_ext(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F;
}

// Implementation for WKB and other generic geometries using the type-tag pattern
impl<F, G> LengthMeasurableExt<F> for G
where
    F: CoordFloat,
    G: GeoTraitExtWithTypeTag + LengthMeasurableTrait<F, G::Tag>,
{
    fn length_ext(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        self.length_trait(metric_space)
    }
}

// Internal trait that handles the actual length computation for different geometry types
trait LengthMeasurableTrait<F, GT: GeoTypeTag>
where
    F: CoordFloat,
{
    fn length_trait(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F;
}

// Implementation for Line geometries
impl<F, L: LineTraitExt<T = F>> LengthMeasurableTrait<F, LineTag> for L
where
    F: CoordFloat,
{
    fn length_trait(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        let start = Point::new(self.start_coord().x, self.start_coord().y);
        let end = Point::new(self.end_coord().x, self.end_coord().y);
        metric_space.distance(start, end)
    }
}

// Implementation for LineString geometries
impl<F, LS: LineStringTraitExt<T = F>> LengthMeasurableTrait<F, LineStringTag> for LS
where
    F: CoordFloat,
{
    fn length_trait(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        let mut length = F::zero();
        for line in self.lines() {
            let start = Point::new(line.start_coord().x, line.start_coord().y);
            let end = Point::new(line.end_coord().x, line.end_coord().y);
            length = length + metric_space.distance(start, end);
        }
        length
    }
}

// Implementation for MultiLineString geometries
impl<F, MLS: MultiLineStringTraitExt<T = F>> LengthMeasurableTrait<F, MultiLineStringTag> for MLS
where
    F: CoordFloat,
{
    fn length_trait(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        let mut length = F::zero();
        for line_string in self.line_strings_ext() {
            length = length + line_string.length_trait(metric_space);
        }
        length
    }
}

// For geometry types that don't have a meaningful length (return zero)
impl<F, P: PointTraitExt<T = F>> LengthMeasurableTrait<F, PointTag> for P
where
    F: CoordFloat,
{
    fn length_trait(&self, _metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        F::zero()
    }
}

impl<F, MP: MultiPointTraitExt<T = F>> LengthMeasurableTrait<F, MultiPointTag> for MP
where
    F: CoordFloat,
{
    fn length_trait(&self, _metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        F::zero()
    }
}

impl<F, P: PolygonTraitExt<T = F>> LengthMeasurableTrait<F, PolygonTag> for P
where
    F: CoordFloat,
{
    fn length_trait(&self, _metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        // Length is a 1D concept, doesn't apply to 2D polygons
        F::zero()
    }
}

impl<F, MP: MultiPolygonTraitExt<T = F>> LengthMeasurableTrait<F, MultiPolygonTag> for MP
where
    F: CoordFloat,
{
    fn length_trait(&self, _metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        F::zero()
    }
}

impl<F, R: RectTraitExt<T = F>> LengthMeasurableTrait<F, RectTag> for R
where
    F: CoordFloat,
{
    fn length_trait(&self, _metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        F::zero()
    }
}

impl<F, T: TriangleTraitExt<T = F>> LengthMeasurableTrait<F, TriangleTag> for T
where
    F: CoordFloat,
{
    fn length_trait(&self, _metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        F::zero()
    }
}

// Implementation for GeometryCollection with runtime type dispatch
impl<F, GC: GeometryCollectionTraitExt<T = F>> LengthMeasurableTrait<F, GeometryCollectionTag>
    for GC
where
    F: CoordFloat,
{
    fn length_trait(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F {
        self.geometries_ext()
            .map(|g| match g.as_type_ext() {
                GeometryTypeExt::Point(_) => F::zero(),
                GeometryTypeExt::Line(line) => line.length_trait(metric_space),
                GeometryTypeExt::LineString(ls) => ls.length_trait(metric_space),
                GeometryTypeExt::Polygon(_) => F::zero(),
                GeometryTypeExt::MultiPoint(_) => F::zero(),
                GeometryTypeExt::MultiLineString(mls) => mls.length_trait(metric_space),
                GeometryTypeExt::MultiPolygon(_) => F::zero(),
                GeometryTypeExt::GeometryCollection(gc) => gc.length_trait(metric_space),
                GeometryTypeExt::Rect(_) => F::zero(),
                GeometryTypeExt::Triangle(_) => F::zero(),
            })
            .fold(F::zero(), |acc, next| acc + next)
    }
}

// Critical: GeometryTag implementation for WKB compatibility
impl<F, G: GeometryTraitExt<T = F>> LengthMeasurableTrait<F, GeometryTag> for G
where
    F: CoordFloat,
{
    crate::geometry_trait_ext_delegate_impl! {
        fn length_trait(&self, metric_space: &impl Distance<F, Point<F>, Point<F>>) -> F;
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
