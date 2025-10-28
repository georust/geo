use crate::{
    structs::{Coord, Geometry},
    CoordTrait, Dimensions, LineStringTrait, LineTrait,
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

    // Conversion from geo-traits' traits

    /// Create a new LineString from a non-empty sequence of objects implementing [CoordTrait].
    ///
    /// This will infer the dimension from the first coordinate, and will not validate that all
    /// coordinates have the same dimension.
    ///
    /// Returns `None` if the input iterator is empty; while an empty LINESTRING is valid, the
    /// dimension cannot be inferred.
    ///
    /// To handle empty input iterators, consider calling `unwrap_or` on the result and defaulting
    /// to an [empty][Self::empty] geometry with specified dimension.
    pub fn from_coords(coords: impl IntoIterator<Item = impl CoordTrait<T = T>>) -> Option<Self> {
        let coords = coords
            .into_iter()
            .map(|c| Coord::from_coord(&c))
            .collect::<Vec<_>>();
        if coords.is_empty() {
            None
        } else {
            let dim = coords[0].dim();
            Some(Self::new(coords, dim))
        }
    }

    pub(crate) fn from_coords_with_dim(
        coords: impl IntoIterator<Item = impl CoordTrait<T = T>>,
        dim: Dimensions,
    ) -> Self {
        match Self::from_coords(coords) {
            Some(line_string) => line_string,
            None => Self {
                coords: Vec::new(),
                dim,
            },
        }
    }

    /// Create a new LineString from an objects implementing [LineStringTrait].
    pub fn from_line_string(linestring: &impl LineStringTrait<T = T>) -> Self {
        Self::from_coords_with_dim(linestring.coords(), linestring.dim())
    }

    /// Create a new LineString from an objects implementing [LineTrait].
    pub fn from_line(line: &impl LineTrait<T = T>) -> Self {
        Self::from_coords_with_dim(line.coords(), line.dim())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LineStringTrait;

    #[test]
    fn empty_linestring_preserves_dimension() {
        let ls: LineString<i32> = LineString::empty(Dimensions::Xym);
        assert_eq!(ls.dimension(), Dimensions::Xym);
        assert!(ls.coords().is_empty());
    }

    #[test]
    fn from_coords_infers_dimension() {
        let coords = vec![
            Coord {
                x: 1,
                y: 2,
                z: Some(3),
                m: None,
            },
            Coord {
                x: 4,
                y: 5,
                z: Some(6),
                m: None,
            },
        ];
        let ls = LineString::from_coords(coords.clone()).expect("coords are non-empty");
        assert_eq!(ls.dimension(), Dimensions::Xyz);
        assert_eq!(ls.coords(), coords.as_slice());
    }

    #[test]
    fn from_coords_returns_none_for_empty_iter() {
        let empty = std::iter::empty::<Coord<i16>>();
        assert!(LineString::from_coords(empty).is_none());
    }

    #[test]
    fn from_linestring_copies_source() {
        let original = LineString::new(
            vec![
                Coord {
                    x: 1,
                    y: 2,
                    z: None,
                    m: Some(7),
                },
                Coord {
                    x: 3,
                    y: 4,
                    z: None,
                    m: Some(8),
                },
            ],
            Dimensions::Xym,
        );
        let converted = LineString::from_line_string(&original);
        assert_eq!(converted.dimension(), original.dimension());
        assert_eq!(converted.coords(), original.coords());
    }

    #[test]
    fn linestring_trait_coord_access() {
        let ls = LineString::new(
            vec![
                Coord {
                    x: 10,
                    y: 11,
                    z: None,
                    m: None,
                },
                Coord {
                    x: 12,
                    y: 13,
                    z: None,
                    m: None,
                },
            ],
            Dimensions::Xy,
        );

        let ls_ref = &ls;
        let second = ls_ref.coord(1).expect("second coord exists");
        assert_eq!(second, &ls.coords()[1]);
        assert!(ls_ref.coord(5).is_none());
    }
}
