use crate::{structs::Geometry, Dimensions, GeometryCollectionTrait, GeometryTrait};

/// A parsed GeometryCollection.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryCollection<T: Copy = f64> {
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

    // Conversion from geo-traits' traits

    /// Create a new GeometryCollection from a non-empty sequence of objects implementing [GeometryTrait].
    ///
    /// This will infer the dimension from the first geometry, and will not validate that all
    /// geometries have the same dimension.
    ///
    /// This returns `None` when
    ///
    /// - the input iterator is empty
    /// - all the geometries are `Line`, `Triangle`, or `Rect` (these geometries are silently skipped)
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_geometries(
        geoms: impl IntoIterator<Item = impl GeometryTrait<T = T>>,
    ) -> Option<Self> {
        let geoms = geoms
            .into_iter()
            .map(|g| Geometry::from_geometry(&g))
            .collect::<Vec<_>>();
        if geoms.is_empty() {
            None
        } else {
            let dim = geoms[0].dimension();
            Some(Self::new(geoms, dim))
        }
    }

    /// Create a new GeometryCollection from an objects implementing [GeometryCollectionTrait].
    pub fn from_geometry_collection(geoms: &impl GeometryCollectionTrait<T = T>) -> Self {
        Self::from_geometries(geoms.geometries()).unwrap()
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
