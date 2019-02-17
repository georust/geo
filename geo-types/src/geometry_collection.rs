use crate::{CoordinateType, Geometry};
use std::iter::FromIterator;

/// A collection of [`Geometry`](enum.Geometry.html) types.
///
/// Can be created from a `Vec` of Geometries, or from an Iterator which yields Geometries.
///
/// Iterating over this objects, yields the component Geometries.
#[derive(PartialEq, Clone, Debug)]
pub struct GeometryCollection<T>(pub Vec<Geometry<T>>)
where
    T: CoordinateType;

impl<T: CoordinateType> GeometryCollection<T> {
    /// Return an empty GeometryCollection
    pub fn new() -> GeometryCollection<T> {
        GeometryCollection(Vec::new())
    }

    /// Number of geometries in this GeometryCollection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is this GeometryCollection empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// Convert any Geometry (or anything that can be converted to a Geometry) into a
/// GeometryCollection
impl<T: CoordinateType, IG: Into<Geometry<T>>> From<IG> for GeometryCollection<T> {
    fn from(x: IG) -> Self {
        GeometryCollection(vec![x.into()])
    }
}

/// Collect Geometries (or what can be converted to a Geometry) into a GeometryCollection
impl<T: CoordinateType, IG: Into<Geometry<T>>> FromIterator<IG> for GeometryCollection<T> {
    fn from_iter<I: IntoIterator<Item = IG>>(iter: I) -> Self {
        GeometryCollection(iter.into_iter().map(|g| g.into()).collect())
    }
}

impl<T: CoordinateType> IntoIterator for GeometryCollection<T> {
    type Item = Geometry<T>;
    type IntoIter = ::std::vec::IntoIter<Geometry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
