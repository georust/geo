use crate::{
    structs::{Geometry, LineString},
    Dimensions, LineStringTrait, MultiLineStringTrait,
};

/// A parsed MultiLineString.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiLineString<T: Copy> {
    pub(crate) line_strings: Vec<LineString<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> MultiLineString<T> {
    /// Create a new LineString from a sequence of [LineString] and known [Dimension].
    pub fn new(line_strings: Vec<LineString<T>>, dim: Dimensions) -> Self {
        MultiLineString { dim, line_strings }
    }

    /// Create a new empty MultiLineString.
    pub fn empty(dim: Dimensions) -> Self {
        Self::new(vec![], dim)
    }

    /// Return the [Dimensions] of this geometry.
    pub fn dimension(&self) -> Dimensions {
        self.dim
    }

    /// Access the inner line strings.
    pub fn line_strings(&self) -> &[LineString<T>] {
        &self.line_strings
    }

    /// Consume self and return the inner parts.
    pub fn into_inner(self) -> (Vec<LineString<T>>, Dimensions) {
        (self.line_strings, self.dim)
    }

    // Conversion from geo-traits' traits

    /// Create a new MultiLineString from a non-empty sequence of objects implementing [LineStringTrait].
    ///
    /// This will infer the dimension from the first line string, and will not validate that all
    /// line strings have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_line_strings(
        line_strings: impl IntoIterator<Item = impl LineStringTrait<T = T>>,
    ) -> Option<Self> {
        let line_strings = line_strings
            .into_iter()
            .map(|l| LineString::from_linestring(l))
            .collect::<Vec<_>>();
        if line_strings.is_empty() {
            None
        } else {
            let dim = line_strings[0].dimension();
            Some(Self::new(line_strings, dim))
        }
    }

    /// Create a new MultiLineString from an objects implementing [MultiLineStringTrait].
    pub fn from_multilinestring(multilinestring: impl MultiLineStringTrait<T = T>) -> Self {
        let line_strings = multilinestring
            .line_strings()
            .map(|l| LineString::from_linestring(l))
            .collect::<Vec<_>>();
        let dim = line_strings[0].dimension();
        Self::new(line_strings, dim)
    }
}

impl<T> From<MultiLineString<T>> for Geometry<T>
where
    T: Copy,
{
    fn from(value: MultiLineString<T>) -> Self {
        Geometry::MultiLineString(value)
    }
}

impl<T: Copy> MultiLineStringTrait for MultiLineString<T> {
    type InnerLineStringType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn num_line_strings(&self) -> usize {
        self.line_strings.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::InnerLineStringType<'_> {
        self.line_strings.get_unchecked(i)
    }
}

impl<T: Copy> MultiLineStringTrait for &MultiLineString<T> {
    type InnerLineStringType<'a>
        = &'a LineString<T>
    where
        Self: 'a;

    fn num_line_strings(&self) -> usize {
        self.line_strings.len()
    }

    unsafe fn line_string_unchecked(&self, i: usize) -> Self::InnerLineStringType<'_> {
        self.line_strings.get_unchecked(i)
    }
}
