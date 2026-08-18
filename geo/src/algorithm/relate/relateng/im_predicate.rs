//! Shared state for predicates determined via [`IntersectionMatrix`]
//! entries, plus the two predicates that work on the raw matrix: the DE-9IM
//! pattern matcher and the full-matrix evaluator.
//!
//! Port of JTS `IMPredicate`, `IMPatternMatcher` and `RelateMatrixPredicate`.

use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::IntersectionMatrix;
use crate::relate::geomgraph::intersection_matrix::InvalidInputError;
use crate::relate::geomgraph::intersection_matrix::dimension_matcher::DimensionMatcher;
use crate::{GeoFloat, Rect};

use super::topology_predicate::{
    InputIndex, PredicateValue, TopologyPredicate, envelopes_intersect,
};

/// Tests whether a geometry of dimension `dim0` can possibly cover a
/// geometry of dimension `dim1`. Points can be covered by zero-length lines.
pub(crate) fn is_dims_compatible_with_covers(dim0: Dimensions, dim1: Dimensions) -> bool {
    if dim0 == Dimensions::ZeroDimensional && dim1 == Dimensions::OneDimensional {
        return true;
    }
    dim0 >= dim1
}

/// The state shared by predicates which are determined using entries in an
/// [`IntersectionMatrix`] (JTS `IMPredicate`).
///
/// Matrix entries start `Empty` (JTS `Dimension.FALSE`) with E/E pre-set to
/// dimension 2, and only ever increase in dimension.
#[derive(Debug, Clone)]
pub(crate) struct IMState {
    pub dim_a: Dimensions,
    pub dim_b: Dimensions,
    pub im: IntersectionMatrix,
    pub value: PredicateValue,
}

impl IMState {
    pub fn new() -> Self {
        Self {
            dim_a: Dimensions::Empty,
            dim_b: Dimensions::Empty,
            im: IntersectionMatrix::empty_disjoint(),
            value: PredicateValue::default(),
        }
    }

    pub fn init_dimensions(&mut self, dim_a: Dimensions, dim_b: Dimensions) {
        self.dim_a = dim_a;
        self.dim_b = dim_b;
    }

    /// The common update flow (JTS `IMPredicate.updateDimension`): only an
    /// increased dimension value is recorded; when the update lets the
    /// predicate short-circuit, its value is fixed from `value_im`.
    pub fn update_dimension(
        &mut self,
        loc_a: CoordPos,
        loc_b: CoordPos,
        dimension: Dimensions,
        is_determined: impl FnOnce(&IMState) -> bool,
        value_im: impl FnOnce(&IMState) -> bool,
    ) {
        if dimension > self.im.get(loc_a, loc_b) {
            self.im.set(loc_a, loc_b, dimension);
            if is_determined(self) {
                let val = value_im(self);
                self.value.set_value(val);
            }
        }
    }

    /// Fixes the final value based on the state of the matrix (JTS
    /// `IMPredicate.finish`).
    pub fn finish(&mut self, value_im: impl FnOnce(&IMState) -> bool) {
        let val = value_im(self);
        self.value.set_value(val);
    }

    /// Tests whether the exterior of the specified input geometry is
    /// intersected by any part of the other input.
    pub fn intersects_exterior_of(&self, input: InputIndex) -> bool {
        match input {
            InputIndex::A => {
                self.is_intersects(CoordPos::Outside, CoordPos::Inside)
                    || self.is_intersects(CoordPos::Outside, CoordPos::OnBoundary)
            }
            InputIndex::B => {
                self.is_intersects(CoordPos::Inside, CoordPos::Outside)
                    || self.is_intersects(CoordPos::OnBoundary, CoordPos::Outside)
            }
        }
    }

    pub fn is_intersects(&self, loc_a: CoordPos, loc_b: CoordPos) -> bool {
        self.im.get(loc_a, loc_b) >= Dimensions::ZeroDimensional
    }

    pub fn get_dimension(&self, loc_a: CoordPos, loc_b: CoordPos) -> Dimensions {
        self.im.get(loc_a, loc_b)
    }
}

/// A predicate that matches a DE-9IM pattern (JTS `IMPatternMatcher`).
///
/// The pattern is a 9-character string of `0`, `1`, `2`, `F`, `T`, `*`
/// entries, listed row-wise.
pub(crate) struct IMPatternMatcher {
    pattern: String,
    pattern_entries: [[DimensionMatcher; 3]; 3],
    pub(crate) state: IMState,
}

pub(crate) const LOCATION_ORDER: [CoordPos; 3] =
    [CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside];

impl IMPatternMatcher {
    pub fn new(pattern: &str) -> Result<Self, InvalidInputError> {
        if pattern.len() != 9 {
            return Err(InvalidInputError::new(format!(
                "DE-9IM pattern must be 9 characters, got: {pattern}"
            )));
        }
        let mut pattern_entries = [[DimensionMatcher::Anything; 3]; 3];
        for (i, ch) in pattern.chars().enumerate() {
            pattern_entries[i / 3][i % 3] = DimensionMatcher::try_from(ch)?;
        }
        Ok(Self {
            pattern: pattern.to_owned(),
            pattern_entries,
            state: IMState::new(),
        })
    }

    /// A pattern entry requires interaction when it demands a non-empty
    /// intersection (`T`, `0`, `1`, or `2`).
    fn entry_requires_interaction(entry: DimensionMatcher) -> bool {
        match entry {
            DimensionMatcher::NonEmpty => true,
            DimensionMatcher::Exact(dim) => dim != Dimensions::Empty,
            DimensionMatcher::Anything => false,
        }
    }

    fn pattern_requires_interaction(&self) -> bool {
        // Interaction is required if the pattern specifies any non-empty
        // entry in the I/B rows and columns.
        [(0, 0), (0, 1), (1, 0), (1, 1)]
            .iter()
            .any(|&(i, j)| Self::entry_requires_interaction(self.pattern_entries[i][j]))
    }

    fn is_determined(state: &IMState, pattern_entries: &[[DimensionMatcher; 3]; 3]) -> bool {
        // Matrix entries only increase in dimension as topology is computed.
        // The predicate can be short-circuited (as false) when a computed
        // entry exceeds an exact pattern entry. A `T` entry keeps the result
        // undetermined until its matrix entry is non-empty.
        for (i, row) in pattern_entries.iter().enumerate() {
            for (j, pattern_entry) in row.iter().enumerate() {
                let matrix_dim = state.im.get(LOCATION_ORDER[i], LOCATION_ORDER[j]);
                match pattern_entry {
                    DimensionMatcher::Anything => continue,
                    DimensionMatcher::NonEmpty => {
                        if matrix_dim == Dimensions::Empty {
                            return false;
                        }
                    }
                    DimensionMatcher::Exact(dim) => {
                        if matrix_dim > *dim {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    fn value_im(state: &IMState, pattern_entries: &[[DimensionMatcher; 3]; 3]) -> bool {
        pattern_entries.iter().enumerate().all(|(i, row)| {
            row.iter().enumerate().all(|(j, pattern_entry)| {
                pattern_entry.matches(state.im.get(LOCATION_ORDER[i], LOCATION_ORDER[j]))
            })
        })
    }
}

impl<F: GeoFloat> TopologyPredicate<F> for IMPatternMatcher {
    fn requires_interaction(&self) -> bool {
        self.pattern_requires_interaction()
    }

    fn init_dimensions(&mut self, dim_a: Dimensions, dim_b: Dimensions) {
        self.state.init_dimensions(dim_a, dim_b);
    }

    fn init_envelopes(&mut self, env_a: Option<Rect<F>>, env_b: Option<Rect<F>>) {
        // If the pattern requires interaction, the envelopes must not be
        // disjoint.
        let is_disjoint = !envelopes_intersect(env_a, env_b);
        self.state
            .value
            .set_value_if(false, self.pattern_requires_interaction() && is_disjoint);
    }

    fn update_dimension(&mut self, loc_a: CoordPos, loc_b: CoordPos, dimension: Dimensions) {
        let pattern_entries = self.pattern_entries;
        self.state.update_dimension(
            loc_a,
            loc_b,
            dimension,
            |state| Self::is_determined(state, &pattern_entries),
            |state| Self::value_im(state, &pattern_entries),
        );
    }

    fn finish(&mut self) {
        let pattern_entries = self.pattern_entries;
        self.state
            .finish(|state| Self::value_im(state, &pattern_entries));
    }

    fn result(&self) -> PredicateValue {
        self.state.value
    }
}

impl std::fmt::Debug for IMPatternMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IMPattern({})", self.pattern)
    }
}

/// Evaluates the full relate [`IntersectionMatrix`]: never short-circuits,
/// so the whole matrix is computed (JTS `RelateMatrixPredicate`).
#[derive(Debug)]
pub(crate) struct RelateMatrixPredicate {
    pub(crate) state: IMState,
}

impl RelateMatrixPredicate {
    pub fn new() -> Self {
        Self {
            state: IMState::new(),
        }
    }

    /// The current state of the matrix (which may only be partially
    /// complete until evaluation has finished).
    pub fn into_im(self) -> IntersectionMatrix {
        self.state.im
    }
}

impl<F: GeoFloat> TopologyPredicate<F> for RelateMatrixPredicate {
    fn requires_interaction(&self) -> bool {
        // Ensure the entire matrix is computed.
        false
    }

    fn init_dimensions(&mut self, dim_a: Dimensions, dim_b: Dimensions) {
        self.state.init_dimensions(dim_a, dim_b);
    }

    fn update_dimension(&mut self, loc_a: CoordPos, loc_b: CoordPos, dimension: Dimensions) {
        // Never determined early: ensure the entire matrix is computed.
        self.state
            .update_dimension(loc_a, loc_b, dimension, |_| false, |_| false);
    }

    fn finish(&mut self) {
        // The result value signals that a full matrix was evaluated; only
        // the matrix itself is meaningful.
        self.state.finish(|_| false);
    }

    fn result(&self) -> PredicateValue {
        self.state.value
    }
}
