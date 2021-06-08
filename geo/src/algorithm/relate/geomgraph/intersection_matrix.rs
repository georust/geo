use crate::algorithm::coordinate_position::CoordPos;
use crate::algorithm::dimensions::Dimensions;

/// Models a *Dimensionally Extended Nine-Intersection Model (DE-9IM)* matrix.
///
/// DE-9IM matrix values (such as "212FF1FF2") specify the topological relationship between
/// two [Geometeries](struct.Geometry.html).
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
#[derive(PartialEq, Eq)]
pub struct IntersectionMatrix(LocationArray<LocationArray<Dimensions>>);

/// Helper struct so we can index IntersectionMatrix by CoordPos
///
/// CoordPos enum members are ordered: OnBondary, Inside, Outside
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
                        let message = format!("expected '0', '1', '2', or 'F'. Found: {}", other);
                        return Err(InvalidInputError::new(message));
                    }
                }
            }
        }

        Ok(())
    }

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
}

impl std::str::FromStr for IntersectionMatrix {
    type Err = InvalidInputError;
    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let mut im = IntersectionMatrix::empty();
        im.set_at_least_from_string(str)?;
        Ok(im)
    }
}
