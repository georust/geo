use crate::{
    structs::{Coord, Geometry},
    CoordTrait as _, Dimensions, LineStringTrait,
};

/// A parsed LineString.
#[derive(Clone, Debug, PartialEq)]
pub struct LineString<T: Copy = f64> {
    pub(crate) coords: Vec<super::Coord<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> LineString<T> {
    /// Create a new LineString from a sequence of [Coord] and known [Dimensions].
    pub fn new(coords: Vec<Coord<T>>, dim: Dimensions) -> Self {
        LineString { dim, coords }
    }

    /// Create a new empty LineString.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Create a new LineString from a non-empty sequence of [Coord].
    ///
    /// This will infer the dimension from the first coordinate, and will not validate that all
    /// coordinates have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_coords(coords: impl IntoIterator<Item = Coord<T>>) -> Option<Self> {
        let coords = coords.into_iter().collect::<Vec<_>>();
        if coords.is_empty() {
            None
        } else {
            let dim = coords[0].dim();
            Some(Self::new(coords, dim))
        }
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the coordinates of this LineString.
    pub fn coords(&self) -> &[Coord<T>] {
        &self.coords
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<Coord<T>>, Dimensions) {
        (self.coords, self.dim)
    }
}

impl<T> From<LineString<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: LineString<T>) -> Self {
        Geometry::LineString(value)
    }
}

impl<T: Copy> LineStringTrait for LineString<T> {
    type CoordType<'a>
        = &'a Coord<T>
    where
        Self: 'a;

    fn num_coords(&self) -> usize {
        self.coords.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.coords.get_unchecked(i)
    }
}

impl<'a, T: Copy> LineStringTrait for &'a LineString<T> {
    type CoordType<'b>
        = &'a Coord<T>
    where
        Self: 'b;

    fn num_coords(&self) -> usize {
        self.coords.len()
    }

    unsafe fn coord_unchecked(&self, i: usize) -> Self::CoordType<'_> {
        self.coords.get_unchecked(i)
    }
}
