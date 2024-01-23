use crate::{coordinate_position::CoordPos, dimensions::Dimensions};

use crate::geometry_cow::GeometryCow::Point;
use std::str::FromStr;

/// Models a *Dimensionally Extended Nine-Intersection Model (DE-9IM)* matrix.
///
/// DE-9IM matrix values (such as "212FF1FF2") specify the topological relationship between
/// two [Geometries](struct.Geometry.html).
///
/// DE-9IM matrices are 3x3 matrices that represent the topological locations
/// that occur in a geometry (Interior, Boundary, Exterior).
///
/// The indices are provided by the enum cases
/// [CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside](CoordPos).
///
/// The matrix entries represent the [Dimensions](enum.Dimension.html) of each intersection.
///
/// For a description of the DE-9IM and the spatial predicates derived from it,
/// see the following references:
/// - [OGC 99-049 OpenGIS Simple Features Specification for SQL](http://portal.opengeospatial.org/files/?artifact_id=829), Section 2.1.13
/// - [OGC 06-103r4 OpenGIS Implementation Standard for Geographic information - Simple feature access - Part 1: Common architecture](http://portal.opengeospatial.org/files/?artifact_id=25355), Section 6.1.15 (which provides some further details on certain predicate specifications).
/// - Wikipedia article on [DE-9IM](https://en.wikipedia.org/wiki/DE-9IM)
///
/// This implementation is heavily based on that from the [JTS project](https://github.com/locationtech/jts/blob/master/modules/core/src/main/java/org/locationtech/jts/geom/IntersectionMatrix.java).
#[derive(PartialEq, Eq, Clone)]
pub struct IntersectionMatrix(LocationArray<LocationArray<Dimensions>>);

/// Helper struct so we can index IntersectionMatrix by CoordPos
///
/// CoordPos enum members are ordered: OnBoundary, Inside, Outside
/// DE-9IM matrices are ordered: Inside, Boundary, Exterior
///
/// So we can't simply `CoordPos as usize` without losing the conventional ordering
/// of elements, which is useful for debug / interop.
#[derive(PartialEq, Eq, Clone, Copy)]
struct LocationArray<T>([T; 3]);

impl<T> LocationArray<T> {
    fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

impl<T> std::ops::Index<CoordPos> for LocationArray<T> {
    type Output = T;

    fn index(&self, index: CoordPos) -> &Self::Output {
        match index {
            CoordPos::Inside => &self.0[0],
            CoordPos::OnBoundary => &self.0[1],
            CoordPos::Outside => &self.0[2],
        }
    }
}

impl<T> std::ops::IndexMut<CoordPos> for LocationArray<T> {
    fn index_mut(&mut self, index: CoordPos) -> &mut Self::Output {
        match index {
            CoordPos::Inside => &mut self.0[0],
            CoordPos::OnBoundary => &mut self.0[1],
            CoordPos::Outside => &mut self.0[2],
        }
    }
}

#[derive(Debug)]
pub struct InvalidInputError {
    message: String,
}

impl InvalidInputError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::error::Error for InvalidInputError {}
impl std::fmt::Display for InvalidInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid input:  {}", self.message)
    }
}

impl std::fmt::Debug for IntersectionMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn char_for_dim(dim: &Dimensions) -> &'static str {
            match dim {
                Dimensions::Empty => "F",
                Dimensions::ZeroDimensional => "0",
                Dimensions::OneDimensional => "1",
                Dimensions::TwoDimensional => "2",
            }
        }
        let text = self
            .0
            .iter()
            .flat_map(|r| r.iter().map(char_for_dim))
            .collect::<Vec<&str>>()
            .join("");

        write!(f, "IntersectionMatrix({})", &text)
    }
}

impl IntersectionMatrix {
    pub fn empty() -> Self {
        IntersectionMatrix(LocationArray([LocationArray([Dimensions::Empty; 3]); 3]))
    }

    /// Set `dimensions` of the cell specified by the positions.
    ///
    /// `position_a`: which position `dimensions` applies to within the first geometry
    /// `position_b`: which position `dimensions` applies to within the second geometry
    /// `dimensions`: the dimension of the incident
    pub(crate) fn set(
        &mut self,
        position_a: CoordPos,
        position_b: CoordPos,
        dimensions: Dimensions,
    ) {
        self.0[position_a][position_b] = dimensions;
    }

    /// Reports an incident of `dimensions`, which updates the IntersectionMatrix if it's greater
    /// than what has been reported so far.
    ///
    /// `position_a`: which position `minimum_dimensions` applies to within the first geometry
    /// `position_b`: which position `minimum_dimensions` applies to within the second geometry
    /// `minimum_dimensions`: the dimension of the incident
    pub(crate) fn set_at_least(
        &mut self,
        position_a: CoordPos,
        position_b: CoordPos,
        minimum_dimensions: Dimensions,
    ) {
        if self.0[position_a][position_b] < minimum_dimensions {
            self.0[position_a][position_b] = minimum_dimensions;
        }
    }

    /// If both geometries have `Some` position, then changes the specified element to at
    /// least `minimum_dimensions`.
    ///
    /// Else, if either is none, do nothing.
    ///
    /// `position_a`: which position `minimum_dimensions` applies to within the first geometry, or
    ///               `None` if the dimension was not incident with the first geometry.
    /// `position_b`: which position `minimum_dimensions` applies to within the second geometry, or
    ///               `None` if the dimension was not incident with the second geometry.
    /// `minimum_dimensions`: the dimension of the incident
    pub(crate) fn set_at_least_if_in_both(
        &mut self,
        position_a: Option<CoordPos>,
        position_b: Option<CoordPos>,
        minimum_dimensions: Dimensions,
    ) {
        if let (Some(position_a), Some(position_b)) = (position_a, position_b) {
            self.set_at_least(position_a, position_b, minimum_dimensions);
        }
    }

    pub(crate) fn set_at_least_from_string(
        &mut self,
        dimensions: &str,
    ) -> Result<(), InvalidInputError> {
        if dimensions.len() != 9 {
            let message = format!("Expected dimensions length 9, found: {}", dimensions.len());
            return Err(InvalidInputError::new(message));
        }

        let mut chars = dimensions.chars();
        for a in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
            for b in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
                match chars.next().expect("already validated length is 9") {
                    '0' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::ZeroDimensional),
                    '1' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::OneDimensional),
                    '2' => self.0[*a][*b] = self.0[*a][*b].max(Dimensions::TwoDimensional),
                    'F' => {}
                    other => {
                        let message = format!("expected '0', '1', '2', or 'F'. Found: {other}");
                        return Err(InvalidInputError::new(message));
                    }
                }
            }
        }

        Ok(())
    }

    // NOTE for implementers
    // See https://en.wikipedia.org/wiki/DE-9IM#Spatial_predicates for a mapping between predicates and matrices
    // The number of constraints in your relation function MUST match the number of NON-MASK (T or F) matrix entries

    // Indexes of the IntersectionMatrix map to indexes of a DE-9IM specification string as follows:
    // ==================================================================
    // self.0[CoordPos::Inside][CoordPos::Inside]: 0
    // self.0[CoordPos::Inside][CoordPos::OnBoundary]: 1
    // self.0[CoordPos::Inside][CoordPos::Outside]: 2

    // self.0[CoordPos::OnBoundary][CoordPos::Inside]: 3
    // self.0[CoordPos::OnBoundary][CoordPos::OnBoundary]: 4
    // self.0[CoordPos::OnBoundary][CoordPos::Outside]: 5

    // self.0[CoordPos::Outside][CoordPos::Inside]: 6
    // self.0[CoordPos::Outside][CoordPos::OnBoundary]: 7
    // self.0[CoordPos::Outside][CoordPos::Outside]: 8
    // ==================================================================

    // Relationship between matrix entry and Dimensions
    // ==================================================================
    // A `T` entry translates to `!= Dimensions::Empty`
    // An `F` entry translates to `== Dimensions::Empty`
    // A `*` (mask) entry is OMITTED
    // ==================================================================

    // Examples
    // ==================================================================
    // `[T********]` -> `self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty`
    // `[********F]` -> `self.0[CoordPos::Outside][CoordPos::Outside] == Dimensions::Empty`
    // `[**T****F*]` -> `self.0[CoordPos::Inside][CoordPos::Outside] != Dimensions::Empty
    //     && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty`
    // ==================================================================

    /// Tests if this matrix matches `[FF*FF****]`.
    ///
    /// returns `true` if the two geometries related by this matrix are disjoint
    pub fn is_disjoint(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::OnBoundary] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::OnBoundary] == Dimensions::Empty
    }

    /// Tests if `is_disjoint` returns false.
    ///
    /// returns `true` if the two geometries related by this matrix intersect.
    pub fn is_intersects(&self) -> bool {
        !self.is_disjoint()
    }

    /// Tests whether this matrix matches `[T*F**F***]`.
    ///
    /// returns `true` if the first geometry is within the second.
    pub fn is_within(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    /// Tests whether this matrix matches `[T*****FF*]`.
    ///
    /// returns `true` if the first geometry contains the second.
    pub fn is_contains(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
    }

    /// Tests whether this matrix matches `[T*F**FFF*]`.
    ///
    /// returns `true` if the first geometry is *topologically* equal to the second.
    pub fn is_equal_topo(&self) -> bool {
        self.0[CoordPos::Inside][CoordPos::Inside] != Dimensions::Empty
            && self.0[CoordPos::Inside][CoordPos::Outside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::Inside] == Dimensions::Empty
            && self.0[CoordPos::Outside][CoordPos::OnBoundary] == Dimensions::Empty
            && self.0[CoordPos::OnBoundary][CoordPos::Outside] == Dimensions::Empty
    }

    /// Directly accesses this matrix
    ///
    /// ```
    /// use geo_types::{LineString, Rect, line_string};
    /// use geo::{coordinate_position::CoordPos, dimensions::Dimensions, relate::Relate};
    ///
    /// let line_string: LineString = line_string![(x: 0.0, y: 0.0), (x: 10.0, y: 0.0), (x: 5.0, y: 5.0)];
    /// let rect = Rect::new((0.0, 0.0), (5.0, 5.0));
    ///
    /// let intersection = line_string.relate(&rect);
    ///
    /// // The intersection of the two interiors is empty, because no part of the string is inside the rect
    /// assert_eq!(intersection.get(CoordPos::Inside, CoordPos::Inside), Dimensions::Empty);
    ///
    /// // The intersection of the line string's interior with the rect's boundary is one-dimensional, because part of the first line overlaps one of the rect's edges
    /// assert_eq!(intersection.get(CoordPos::Inside, CoordPos::OnBoundary), Dimensions::OneDimensional);
    ///
    /// // The intersection of the line string's interior with the rect's exterior is one-dimensional, because part of the string is outside the rect
    /// assert_eq!(intersection.get(CoordPos::Inside, CoordPos::Outside), Dimensions::OneDimensional);
    ///
    /// // The intersection of the line string's boundary with the rect's interior is empty, because neither of its end points are inside the rect
    /// assert_eq!(intersection.get(CoordPos::OnBoundary, CoordPos::Inside), Dimensions::Empty);
    ///
    /// // The intersection of the line string's boundary with the rect's boundary is zero-dimensional, because the string's start and end points are on the rect's edges
    /// assert_eq!(intersection.get(CoordPos::OnBoundary, CoordPos::OnBoundary), Dimensions::ZeroDimensional);
    ///
    /// // The intersection of the line string's boundary with the rect's exterior is empty, because neither of its end points are outside the rect
    /// assert_eq!(intersection.get(CoordPos::OnBoundary, CoordPos::Outside), Dimensions::Empty);
    ///
    /// // The intersection of the the line's exterior with the rect's interior is two-dimensional, because it's simply the rect's interior
    /// assert_eq!(intersection.get(CoordPos::Outside, CoordPos::Inside), Dimensions::TwoDimensional);
    ///
    /// // The intersection of the line's exterior with the rect's boundary is one-dimensional, because it's the rect's edges (minus where the string overlaps it)
    /// assert_eq!(intersection.get(CoordPos::Outside, CoordPos::OnBoundary), Dimensions::OneDimensional);
    ///
    /// // The intersection of the two exteriors is two-dimensional, because it's the whole plane around the two shapes
    /// assert_eq!(intersection.get(CoordPos::Outside, CoordPos::Outside), Dimensions::TwoDimensional);
    /// ```
    pub fn get(&self, lhs: CoordPos, rhs: CoordPos) -> Dimensions {
        self.0[lhs][rhs]
    }

    /// Does the intersection matrix match the provided de-9im specification string?
    ///
    /// A de-9im spec string must be 9 characters long, and each character
    /// must be one of the following:
    ///
    /// - 0: matches a 0-dimensional (point) intersection
    /// - 1: matches a 1-dimensional (line) intersection
    /// - 2: matches a 2-dimensional (area) intersection
    /// - f or F: matches only empty dimensions
    /// - t or T: matches anything non-empty
    /// - *: matches anything
    ///
    /// ```
    /// use geo::algorithm::Relate;
    /// use geo::geometry::Polygon;
    /// use wkt::TryFromWkt;
    ///
    /// let a = Polygon::<f64>::try_from_wkt_str("POLYGON((0 0,4 0,4 4,0 4,0 0))").expect("valid WKT");
    /// let b = Polygon::<f64>::try_from_wkt_str("POLYGON((1 1,4 0,4 4,0 4,1 1))").expect("valid WKT");
    /// let im = a.relate(&b);
    /// assert!(im.matches("212F11FF2").expect("valid de-9im spec"));
    /// assert!(im.matches("TTT***FF2").expect("valid de-9im spec"));
    /// assert!(!im.matches("TTT***FFF").expect("valid de-9im spec"));
    /// ```
    pub fn matches(&self, spec: &str) -> Result<bool, InvalidInputError> {
        if spec.len() != 9 {
            return Err(InvalidInputError::new(format!(
                "de-9im specification must be exactly 9 characters. Got {len}",
                len = spec.len()
            )));
        }

        let mut chars = spec.chars();
        for a in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
            for b in &[CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside] {
                let dim_spec = dimension_matcher::DimensionMatcher::try_from(
                    chars.next().expect("already validated length is 9"),
                )?;
                if !dim_spec.matches(self.0[*a][*b]) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

/// Build an IntersectionMatrix based on a string specification.
/// ```
/// use geo::algorithm::relate::IntersectionMatrix;
/// use std::str::FromStr;
///
/// let intersection_matrix = IntersectionMatrix::from_str("212101212").expect("valid de-9im specification");
/// assert!(intersection_matrix.is_intersects());
/// assert!(!intersection_matrix.is_contains());
/// ```
impl FromStr for IntersectionMatrix {
    type Err = InvalidInputError;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut im = IntersectionMatrix::empty();
        im.set_at_least_from_string(str)?;
        Ok(im)
    }
}

pub(crate) mod dimension_matcher {
    use super::Dimensions;
    use super::InvalidInputError;

    /// A single letter from a de-9im matching specification like "1*T**FFF*"
    pub(crate) enum DimensionMatcher {
        Anything,
        NonEmpty,
        Exact(Dimensions),
    }

    impl DimensionMatcher {
        pub fn matches(&self, dim: Dimensions) -> bool {
            match (self, dim) {
                (Self::Anything, _) => true,
                (DimensionMatcher::NonEmpty, d) => d != Dimensions::Empty,
                (DimensionMatcher::Exact(a), b) => a == &b,
            }
        }
    }

    impl TryFrom<char> for DimensionMatcher {
        type Error = InvalidInputError;

        fn try_from(value: char) -> Result<Self, Self::Error> {
            Ok(match value {
                '*' => Self::Anything,
                't' | 'T' => Self::NonEmpty,
                'f' | 'F' => Self::Exact(Dimensions::Empty),
                '0' => Self::Exact(Dimensions::ZeroDimensional),
                '1' => Self::Exact(Dimensions::OneDimensional),
                '2' => Self::Exact(Dimensions::TwoDimensional),
                _ => {
                    return Err(InvalidInputError::new(format!(
                        "invalid de-9im specification character: {value}"
                    )))
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subject() -> IntersectionMatrix {
        // Topologically, this is a nonsense IM
        IntersectionMatrix::from_str("F00111222").unwrap()
    }

    #[test]
    fn matches_exactly() {
        assert!(subject().matches("F00111222").unwrap());
    }

    #[test]
    fn doesnt_match() {
        assert!(!subject().matches("222222222").unwrap());
    }

    #[test]
    fn matches_truthy() {
        assert!(subject().matches("FTTTTTTTT").unwrap());
    }

    #[test]
    fn matches_wildcard() {
        assert!(subject().matches("F0011122*").unwrap());
    }
}
