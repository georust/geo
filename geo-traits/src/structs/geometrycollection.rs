use crate::{structs::Geometry, Dimensions, GeometryCollectionTrait};

/// A parsed GeometryCollection.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryCollection<T: Copy> {
    pub(crate) geoms: Vec<Geometry<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> GeometryCollection<T> {
    /// Create a new GeometryCollection from a sequence of [Geometry].
    pub fn new(geoms: Vec<Geometry<T>>, dim: Dimensions) -> Self {
        Self { geoms, dim }
    }

    /// Create a new empty GeometryCollection.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new GeometryCollection from a non-empty sequence of [Geometry].
    ///
    /// This will infer the dimension from the first geometry, and will not validate that all
    /// geometries have the same dimension.
    ///
    /// ## Errors
    ///
    /// If the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_geometries(geoms: impl IntoIterator<Item = Geometry<T>>) -> Option<Self> {
        let geoms = geoms.into_iter().collect::<Vec<_>>();
        if geoms.is_empty() {
            None
        } else {
            let dim = geoms[0].dimension();
            Some(Self::new(geoms, dim))
        }
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the underlying geometries.
    pub fn geometries(&self) -> &[Geometry<T>] {
        &self.geoms
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<Geometry<T>>, Dimensions) {
        (self.geoms, self.dim)
    }
}

impl<T> From<GeometryCollection<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: GeometryCollection<T>) -> Self {
        Geometry::GeometryCollection(value)
    }
}

impl<T: Copy> GeometryCollectionTrait for GeometryCollection<T> {
    type GeometryType<'a>
        = &'a Geometry<T>
    where
        Self: 'a;

    fn num_geometries(&self) -> usize {
        self.geoms.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.geoms.get_unchecked(i)
    }
}

impl<T: Copy> GeometryCollectionTrait for &GeometryCollection<T> {
    type GeometryType<'a>
        = &'a Geometry<T>
    where
        Self: 'a;

    fn num_geometries(&self) -> usize {
        self.geoms.len()
    }

    unsafe fn geometry_unchecked(&self, i: usize) -> Self::GeometryType<'_> {
        self.geoms.get_unchecked(i)
    }
}
