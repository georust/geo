use super::MonotoneChainPolygon;
use crate::GeoNum;
use crate::geometry::MultiPolygon;

/// A [`MultiPolygon`] backed by [`MonotoneChain`](`crate::MonotoneChain`).
///
/// # Example
/// ```
/// use geo::{MultiPolygon, Polygon, line_string, MonotoneChainMultiPolygon};
///
/// let polygon = Polygon::new(
///     line_string![
///         (x: 0., y: 0.),
///         (x: 10., y: 0.),
///         (x: 10., y: 10.),
///         (x: 0., y: 10.),
///         (x: 0., y: 0.),
///     ],
///     vec![]
/// );
///
/// let mp = MultiPolygon::new(vec![polygon.clone()]);
/// let m_mp:MonotoneChainMultiPolygon<f64> = (&mp).into();
/// ```
///
pub struct MonotoneChainMultiPolygon<'a, T: GeoNum> {
    geometry: &'a MultiPolygon<T>,
    components: Vec<MonotoneChainPolygon<'a, T>>,
}

impl<'a, T: GeoNum> MonotoneChainMultiPolygon<'a, T> {
    pub fn geometry(&self) -> &'a MultiPolygon<T> {
        self.geometry
    }
    pub fn components(&self) -> &[MonotoneChainPolygon<'a, T>] {
        &self.components
    }
}

impl<'a, T: GeoNum> From<&'a MultiPolygon<T>> for MonotoneChainMultiPolygon<'a, T> {
    fn from(multi_polygon: &'a MultiPolygon<T>) -> Self {
        Self {
            geometry: multi_polygon,
            components: multi_polygon.iter().map(|polygon| polygon.into()).collect(),
        }
    }
}

impl<'a, T: GeoNum> From<MonotoneChainMultiPolygon<'a, T>> for &'a MultiPolygon<T> {
    fn from(val: MonotoneChainMultiPolygon<'a, T>) -> Self {
        val.geometry
    }
}
