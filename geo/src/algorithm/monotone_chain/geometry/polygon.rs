use super::MonotoneChainLineString;
use crate::GeoNum;
use crate::geometry::Polygon;

/// A [`Polygon`] backed by [`MonotoneChain`](`crate::MonotoneChain`).
///
/// # Example
/// ```
/// use geo::{line_string, Polygon, MonotoneChainPolygon};
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
/// let m_polygon:MonotoneChainPolygon<f64> = (&polygon).into();
/// ```
///
pub struct MonotoneChainPolygon<'a, T: GeoNum> {
    geometry: &'a Polygon<T>,
    exterior: MonotoneChainLineString<'a, T>,
    interiors: Vec<MonotoneChainLineString<'a, T>>,
}

impl<'a, T: GeoNum> MonotoneChainPolygon<'a, T> {
    pub fn geometry(&self) -> &'a Polygon<T> {
        self.geometry
    }
    pub fn exterior(&self) -> &MonotoneChainLineString<'a, T> {
        &self.exterior
    }
    pub fn interiors(&self) -> &[MonotoneChainLineString<'a, T>] {
        &self.interiors
    }
}

impl<'a, T: GeoNum> From<&'a Polygon<T>> for MonotoneChainPolygon<'a, T> {
    fn from(polygon: &'a Polygon<T>) -> Self {
        Self {
            geometry: polygon,
            exterior: polygon.exterior().into(),
            interiors: polygon.interiors().iter().map(|ring| ring.into()).collect(),
        }
    }
}

impl<'a, T: GeoNum> From<MonotoneChainPolygon<'a, T>> for &'a Polygon<T> {
    fn from(val: MonotoneChainPolygon<'a, T>) -> Self {
        val.geometry
    }
}
