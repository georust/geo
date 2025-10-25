use crate::{
    structs::{Geometry, LineString},
    Dimensions, PolygonTrait,
};

/// A parsed Polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct Polygon<T: Copy> {
    pub(crate) rings: Vec<LineString<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> Polygon<T> {
    /// Create a new Polygon from a sequence of [LineString] and known [Dimensions].
    pub fn new(rings: Vec<LineString<T>>, dim: Dimensions) -> Self {
        Polygon { dim, rings }
    }

    /// Create a new empty polygon.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new polygon from a non-empty sequence of [LineString].
    ///
    /// This will infer the dimension from the first line string, and will not validate that all
    /// line strings have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_rings(rings: impl IntoIterator<Item = LineString<T>>) -> Option<Self> {
        let rings = rings.into_iter().collect::<Vec<_>>();
        if rings.is_empty() {
            None
        } else {
            let dim = rings[0].dimension();
            Some(Self::new(rings, dim))
        }
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the inner rings.
    ///
    /// The first ring is defined to be the exterior ring, and the rest are interior rings.
    pub fn rings(&self) -> &[LineString<T>] {
        &self.rings
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<LineString<T>>, Dimensions) {
        (self.rings, self.dim)
    }
}

impl<T> From<Polygon<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: Polygon<T>) -> Self {
        Geometry::Polygon(value)
    }
}

impl<T: Copy> PolygonTrait for Polygon<T> {
    type RingType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        self.rings.first()
    }

    fn num_interiors(&self) -> usize {
        self.rings.len().saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        self.rings.get_unchecked(i + 1)
    }
}

impl<T: Copy> PolygonTrait for &Polygon<T> {
    type RingType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn exterior(&self) -> Option<Self::RingType<'_>> {
        self.rings.first()
    }

    fn num_interiors(&self) -> usize {
        self.rings.len().saturating_sub(1)
    }

    unsafe fn interior_unchecked(&self, i: usize) -> Self::RingType<'_> {
        self.rings.get_unchecked(i + 1)
    }
}
