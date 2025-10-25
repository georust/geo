use crate::{
    structs::{Geometry, Polygon},
    Dimensions, MultiPolygonTrait, PolygonTrait,
};

/// A parsed MultiPolygon.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiPolygon<T: Copy> {
    pub(crate) polygons: Vec<Polygon<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> MultiPolygon<T> {
    /// Create a new MultiPolygon from a sequence of [Polygon] and known [Dimensions].
    pub fn new(polygons: Vec<Polygon<T>>, dim: Dimensions) -> Self {
        MultiPolygon { dim, polygons }
    }

    /// Create a new empty MultiPolygon.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the inner polygons.
    pub fn polygons(&self) -> &[Polygon<T>] {
        &self.polygons
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<Polygon<T>>, Dimensions) {
        (self.polygons, self.dim)
    }

    // Conversion from geo-traits' traits

    /// Create a new MultiPolygon from a non-empty sequence of objects implementing [PolygonTrait].
    ///
    /// This will infer the dimension from the first polygon, and will not validate that all
    /// polygons have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_polygons(
        polygons: impl IntoIterator<Item = impl PolygonTrait<T = T>>,
    ) -> Option<Self> {
        let polygons = polygons
            .into_iter()
            .map(|p| Polygon::from_polygon(p))
            .collect::<Vec<_>>();
        if polygons.is_empty() {
            None
        } else {
            let dim = polygons[0].dimension();
            Some(Self::new(polygons, dim))
        }
    }

    /// Create a new MultiPolygon from an objects implementing [MultiPolygonTrait].
    pub fn from_multipolygon(multipolygon: impl MultiPolygonTrait<T = T>) -> Self {
        let polygons = multipolygon
            .polygons()
            .map(|p| Polygon::from_polygon(p))
            .collect::<Vec<_>>();
        let dim = polygons[0].dimension();
        Self::new(polygons, dim)
    }
}

impl<T> From<MultiPolygon<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: MultiPolygon<T>) -> Self {
        Geometry::MultiPolygon(value)
    }
}

impl<T: Copy> MultiPolygonTrait for MultiPolygon<T> {
    type InnerPolygonType<'a>
        = &'a Polygon<T>
    where
        Self: 'a;

    fn num_polygons(&self) -> usize {
        self.polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::InnerPolygonType<'_> {
        self.polygons.get_unchecked(i)
    }
}

impl<T: Copy> MultiPolygonTrait for &MultiPolygon<T> {
    type InnerPolygonType<'a>
        = &'a Polygon<T>
    where
        Self: 'a;

    fn num_polygons(&self) -> usize {
        self.polygons.len()
    }

    unsafe fn polygon_unchecked(&self, i: usize) -> Self::InnerPolygonType<'_> {
        self.polygons.get_unchecked(i)
    }
}
