use crate::GeoNum;
use crate::MonotoneChain;
use crate::geometry::LineString;

/// A [`LineString`] backed by [`MonotoneChain`](`crate::MonotoneChain`).
///
/// # Example
///
/// ```
/// use geo::{LineString, line_string, MonotoneChainLineString};
/// let ls: LineString<f64> = line_string![(x: 0., y: 0.), (x: 10., y: 10.)];
/// let m_ls:MonotoneChainLineString<f64> = (&ls).into();
/// ```
///
pub struct MonotoneChainLineString<'a, T: GeoNum> {
    geometry: &'a LineString<T>,
    chain: MonotoneChain<'a, T>,
}

impl<'a, T: GeoNum> MonotoneChainLineString<'a, T> {
    pub fn geometry(&self) -> &'a LineString<T> {
        self.geometry
    }
    pub fn chain(&self) -> &MonotoneChain<'a, T> {
        &self.chain
    }
}

impl<'a, T: GeoNum> From<&'a LineString<T>> for MonotoneChainLineString<'a, T> {
    fn from(linestring: &'a LineString<T>) -> Self {
        Self {
            geometry: linestring,
            chain: MonotoneChain::from(linestring),
        }
    }
}

impl<'a, T: GeoNum> From<MonotoneChainLineString<'a, T>> for &'a LineString<T> {
    fn from(val: MonotoneChainLineString<'a, T>) -> Self {
        val.geometry
    }
}
