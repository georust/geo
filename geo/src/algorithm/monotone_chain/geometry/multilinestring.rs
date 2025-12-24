use super::MonotoneChainLineString;
use crate::GeoNum;
use crate::geometry::MultiLineString;

/// A [`MultiLineString`] backed by [`MonotoneChain`](`crate::MonotoneChain`).
///
/// # Example
///
/// ```
/// use geo::{MultiLineString, line_string, MonotoneChainMultiLineString};
/// let mls = MultiLineString::new(vec![line_string![(x: 0., y: 0.), (x: 10., y: 10.)]]);
/// let m_mls:MonotoneChainMultiLineString<f64> = (&mls).into();
/// ```
///
pub struct MonotoneChainMultiLineString<'a, T: GeoNum> {
    geometry: &'a MultiLineString<T>,
    components: Vec<MonotoneChainLineString<'a, T>>,
}

impl<'a, T: GeoNum> MonotoneChainMultiLineString<'a, T> {
    pub fn geometry(&self) -> &'a MultiLineString<T> {
        self.geometry
    }
    pub fn components(&self) -> &[MonotoneChainLineString<'a, T>] {
        &self.components
    }
}

impl<'a, T: GeoNum> From<&'a MultiLineString<T>> for MonotoneChainMultiLineString<'a, T> {
    fn from(multi_linestring: &'a MultiLineString<T>) -> Self {
        Self {
            geometry: multi_linestring,
            components: multi_linestring
                .iter()
                .map(|linestring| linestring.into())
                .collect(),
        }
    }
}

impl<'a, T: GeoNum> From<MonotoneChainMultiLineString<'a, T>> for &'a MultiLineString<T> {
    fn from(val: MonotoneChainMultiLineString<'a, T>) -> Self {
        val.geometry
    }
}
