use crate::{CoordinateType, Polygon};
use std::iter::FromIterator;
use std::hash::{Hash, Hasher};

/// A collection of [`Polygon`s](struct.Polygon.html).
///
/// Can be created from a `Vec` of `Polygon`s, or `collect`ed from an Iterator which yields `Polygon`s.
///
/// Iterating over this object yields the component Polygons.
#[derive(PartialEq, Clone, Debug)]
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

impl<T: CoordinateType + Hash> Hash for MultiPolygon<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
