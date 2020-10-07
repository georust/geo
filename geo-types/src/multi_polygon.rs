use crate::{CoordinateType, Polygon};
use std::iter::FromIterator;

/// A collection of [`Polygon`s](struct.Polygon.html). Can
/// be created from a `Vec` of `Polygon`s, or from an
/// Iterator which yields `Polygon`s. Iterating over this
/// object yields the component `Polygon`s.
///
/// # Semantics
///
/// The _interior_ and the _boundary_ are the union of the
/// interior and the boundary of the constituent polygons.
///
/// # Validity
///
/// - The interiors of no two constituent polygons may intersect.
///
/// - The boundaries of two (distinct) constituent polygons
/// may only intersect at finitely many points.
///
/// Refer to section 6.1.14 of the OGC-SFA for a formal
/// definition of validity. Note that the validity is not
/// enforced, but expected by the operations and
/// predicates that operate on it.
#[derive(Eq, PartialEq, Clone, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MultiPolygon<T>(pub Vec<Polygon<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType, IP: Into<Polygon<T>>> From<IP> for MultiPolygon<T> {
    fn from(x: IP) -> Self {
        MultiPolygon(vec![x.into()])
    }
}

impl<T: CoordinateType, IP: Into<Polygon<T>>> From<Vec<IP>> for MultiPolygon<T> {
    fn from(x: Vec<IP>) -> Self {
        MultiPolygon(x.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordinateType, IP: Into<Polygon<T>>> FromIterator<IP> for MultiPolygon<T> {
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        MultiPolygon(iter.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordinateType> IntoIterator for MultiPolygon<T> {
    type Item = Polygon<T>;
    type IntoIter = ::std::vec::IntoIter<Polygon<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
