use crate::{
    structs::{Geometry, LineString},
    Dimensions, LineStringTrait, MultiLineStringTrait,
};

/// A parsed MultiLineString.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiLineString<T: Copy = f64> {
    pub(crate) line_strings: Vec<LineString<T>>,
    pub(crate) dim: Dimensions,
}

impl<T: Copy> MultiLineString<T> {
    /// Create a new LineString from a sequence of [LineString] and known [Dimensions].
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
            .map(|l| LineString::from_linestring(&l))
            .collect::<Vec<_>>();
        if line_strings.is_empty() {
            None
        } else {
            let dim = line_strings[0].dimension();
            Some(Self::new(line_strings, dim))
        }
    }

    /// Create a new MultiLineString from an objects implementing [MultiLineStringTrait].
    pub fn from_multilinestring(multilinestring: &impl MultiLineStringTrait<T = T>) -> Self {
        let line_strings = multilinestring
            .line_strings()
            .map(|l| LineString::from_linestring(&l))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structs::Coord;
    use crate::MultiLineStringTrait;

    fn line_xy(coords: &[(i32, i32)]) -> LineString<i32> {
        LineString::new(
            coords
                .iter()
                .map(|&(x, y)| Coord {
                    x,
                    y,
                    z: None,
                    m: None,
                })
                .collect(),
            Dimensions::Xy,
        )
    }

    #[test]
    fn empty_multilinestring_preserves_dimension() {
        let mls: MultiLineString<i32> = MultiLineString::empty(Dimensions::Xyz);
        assert_eq!(mls.dimension(), Dimensions::Xyz);
        assert!(mls.line_strings().is_empty());
    }

    #[test]
    fn from_line_strings_infers_dimension() {
        let lines = vec![
            LineString::new(
                vec![
                    Coord {
                        x: 0,
                        y: 0,
                        z: Some(1),
                        m: None,
                    },
                    Coord {
                        x: 1,
                        y: 1,
                        z: Some(2),
                        m: None,
                    },
                ],
                Dimensions::Xyz,
            ),
            LineString::new(
                vec![
                    Coord {
                        x: 2,
                        y: 3,
                        z: Some(4),
                        m: None,
                    },
                    Coord {
                        x: 5,
                        y: 8,
                        z: Some(9),
                        m: None,
                    },
                ],
                Dimensions::Xyz,
            ),
        ];

        let mls =
            MultiLineString::from_line_strings(lines.clone()).expect("line strings are non-empty");
        assert_eq!(mls.dimension(), Dimensions::Xyz);
        assert_eq!(mls.line_strings(), lines.as_slice());
    }

    #[test]
    fn from_line_strings_returns_none_for_empty_iter() {
        let empty = std::iter::empty::<LineString<i64>>();
        assert!(MultiLineString::from_line_strings(empty).is_none());
    }

    #[test]
    fn from_multilinestring_copies_source() {
        let lines = vec![
            line_xy(&[(0, 0), (1, 0), (1, 1)]),
            line_xy(&[(2, 2), (3, 3), (3, 4)]),
        ];
        let original = MultiLineString::new(lines.clone(), Dimensions::Xy);
        let converted = MultiLineString::from_multilinestring(&original);

        assert_eq!(converted.dimension(), original.dimension());
        assert_eq!(converted.line_strings(), original.line_strings());
    }

    #[test]
    fn multilinestring_trait_accessors_work() {
        let lines = vec![line_xy(&[(0, 0), (0, 1)]), line_xy(&[(1, 1), (2, 2)])];
        let mls = MultiLineString::new(lines.clone(), Dimensions::Xy);

        assert_eq!(mls.num_line_strings(), 2);
        assert_eq!(mls.line_string(0), Some(&lines[0]));
        assert!(mls.line_string(5).is_none());

        let borrowed = &mls;
        let second = borrowed.line_string(1).expect("second line exists");
        assert_eq!(second, &lines[1]);
    }
}
