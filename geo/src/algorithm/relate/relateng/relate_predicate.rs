//! Predicates for the OGC-standard named topological relationships.
//!
//! Port of JTS `RelatePredicate` and `IntersectionMatrixPattern`. The eight
//! matrix-determined predicates share one struct over a kind enum; the
//! behaviour differences between them (JTS's anonymous subclass overrides)
//! are the match arms below. `intersects` and `disjoint` need no matrix and
//! have their own types.

use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::intersection_matrix::InvalidInputError;
use crate::{GeoFloat, Rect};

use super::im_predicate::{IMPatternMatcher, IMState, is_dims_compatible_with_covers};
use super::topology_predicate::{
    InputIndex, PredicateValue, TopologyPredicate, envelope_covers, envelopes_equal,
    envelopes_intersect, is_intersection,
};

/// DE-9IM matrix patterns for topological relationships that have no OGC
/// name (JTS `IntersectionMatrixPattern`).
pub(crate) mod intersection_matrix_pattern {
    /// Two polygonal geometries are adjacent along an edge, but do not
    /// overlap.
    pub const ADJACENT: &str = "F***1****";
    /// A geometry properly contains another geometry (which lies entirely
    /// in its interior).
    pub const CONTAINS_PROPERLY: &str = "T**FF*FF*";
    /// Two geometries intersect in their interiors.
    pub const INTERIOR_INTERSECTS: &str = "T********";
}

/// Creates a predicate to determine whether two geometries intersect: they
/// have at least one point in common.
pub(crate) fn intersects() -> IntersectsPredicate {
    IntersectsPredicate::default()
}

/// Creates a predicate to determine whether two geometries are disjoint:
/// they have no point in common.
pub(crate) fn disjoint() -> DisjointPredicate {
    DisjointPredicate::default()
}

/// Creates a predicate to determine whether geometry A contains geometry B.
/// Matches `[T*****FF*]`.
pub(crate) fn contains() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::Contains)
}

/// Creates a predicate to determine whether geometry A is within geometry B.
/// Matches `[T*F**F***]`.
pub(crate) fn within() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::Within)
}

/// Creates a predicate to determine whether geometry A covers geometry B.
pub(crate) fn covers() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::Covers)
}

/// Creates a predicate to determine whether geometry A is covered by
/// geometry B.
pub(crate) fn covered_by() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::CoveredBy)
}

/// Creates a predicate to determine whether two geometries cross.
pub(crate) fn crosses() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::Crosses)
}

/// Creates a predicate to determine whether two geometries are
/// topologically equal. Matches `[T*F**FFF*]`; all empty geometries are
/// topologically equal, regardless of dimension.
pub(crate) fn equals_topo() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::EqualsTopo)
}

/// Creates a predicate to determine whether two geometries overlap.
pub(crate) fn overlaps() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::Overlaps)
}

/// Creates a predicate to determine whether two geometries touch.
pub(crate) fn touches() -> NamedPredicate {
    NamedPredicate::new(PredicateKind::Touches)
}

/// Creates a predicate that matches a DE-9IM matrix pattern.
///
/// See [`intersection_matrix_pattern`] for patterns for some common
/// unnamed relationships.
pub(crate) fn matches(im_pattern: &str) -> Result<IMPatternMatcher, InvalidInputError> {
    IMPatternMatcher::new(im_pattern)
}

/// A predicate to determine whether two geometries intersect (JTS
/// `RelatePredicate.intersects()`). Needs no matrix: any non-exterior
/// interaction settles it.
#[derive(Debug, Default)]
pub(crate) struct IntersectsPredicate {
    value: PredicateValue,
}

impl<F: GeoFloat> TopologyPredicate<F> for IntersectsPredicate {
    fn name(&self) -> &'static str {
        "intersects"
    }

    fn requires_self_noding(&self) -> bool {
        // Self-noding is not required to check for a simple interaction.
        false
    }

    fn requires_exterior_check(&self, _source: InputIndex) -> bool {
        // Intersects only requires testing interaction.
        false
    }

    fn init_envelopes(&mut self, env_a: Option<Rect<F>>, env_b: Option<Rect<F>>) {
        self.value.require(envelopes_intersect(env_a, env_b));
    }

    fn update_dimension(&mut self, loc_a: CoordPos, loc_b: CoordPos, _dimension: Dimensions) {
        self.value.set_value_if(true, is_intersection(loc_a, loc_b));
    }

    fn finish(&mut self) {
        // No intersecting locations were found.
        self.value.set_value(false);
    }

    fn is_known(&self) -> bool {
        self.value.is_known()
    }

    fn value(&self) -> bool {
        self.value.value()
    }
}

/// A predicate to determine whether two geometries are disjoint (JTS
/// `RelatePredicate.disjoint()`).
#[derive(Debug, Default)]
pub(crate) struct DisjointPredicate {
    value: PredicateValue,
}

impl<F: GeoFloat> TopologyPredicate<F> for DisjointPredicate {
    fn name(&self) -> &'static str {
        "disjoint"
    }

    fn requires_self_noding(&self) -> bool {
        // Self-noding is not required to check for a simple interaction.
        false
    }

    fn requires_interaction(&self) -> bool {
        false
    }

    fn requires_exterior_check(&self, _source: InputIndex) -> bool {
        // Disjoint only requires testing interaction.
        false
    }

    fn init_envelopes(&mut self, env_a: Option<Rect<F>>, env_b: Option<Rect<F>>) {
        self.value
            .set_value_if(true, !envelopes_intersect(env_a, env_b));
    }

    fn update_dimension(&mut self, loc_a: CoordPos, loc_b: CoordPos, _dimension: Dimensions) {
        self.value
            .set_value_if(false, is_intersection(loc_a, loc_b));
    }

    fn finish(&mut self) {
        // No intersecting locations were found.
        self.value.set_value(true);
    }

    fn is_known(&self) -> bool {
        self.value.is_known()
    }

    fn value(&self) -> bool {
        self.value.value()
    }
}

/// The OGC-standard named predicates that are determined via intersection
/// matrix entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PredicateKind {
    Contains,
    Within,
    Covers,
    CoveredBy,
    Crosses,
    EqualsTopo,
    Overlaps,
    Touches,
}

/// A matrix-determined named predicate. The per-kind behaviour mirrors the
/// anonymous `IMPredicate` subclasses in JTS `RelatePredicate`.
#[derive(Debug)]
pub(crate) struct NamedPredicate {
    kind: PredicateKind,
    pub(crate) state: IMState,
}

impl NamedPredicate {
    pub fn new(kind: PredicateKind) -> Self {
        Self {
            kind,
            state: IMState::new(),
        }
    }

    /// Tests whether predicate evaluation can be short-circuited due to the
    /// current state of the matrix providing enough information to determine
    /// the predicate value. If true, `value_im` provides the correct result.
    fn is_determined(kind: PredicateKind, state: &IMState) -> bool {
        use CoordPos::{Inside, OnBoundary, Outside};
        match kind {
            PredicateKind::Contains | PredicateKind::Covers => {
                state.intersects_exterior_of(InputIndex::A)
            }
            PredicateKind::Within | PredicateKind::CoveredBy => {
                state.intersects_exterior_of(InputIndex::B)
            }
            PredicateKind::Crosses => {
                if state.dim_a == Dimensions::OneDimensional
                    && state.dim_b == Dimensions::OneDimensional
                {
                    // An L/L interior interaction can only have dimension 0.
                    state.get_dimension(Inside, Inside) > Dimensions::ZeroDimensional
                } else if state.dim_a < state.dim_b {
                    state.is_intersects(Inside, Inside) && state.is_intersects(Inside, Outside)
                } else if state.dim_a > state.dim_b {
                    state.is_intersects(Inside, Inside) && state.is_intersects(Outside, Inside)
                } else {
                    false
                }
            }
            PredicateKind::EqualsTopo => {
                // Determined (false) as soon as either exterior is
                // intersected.
                state.is_intersects(Inside, Outside)
                    || state.is_intersects(OnBoundary, Outside)
                    || state.is_intersects(Outside, Inside)
                    || state.is_intersects(Outside, OnBoundary)
            }
            PredicateKind::Overlaps => {
                if state.dim_a == Dimensions::TwoDimensional
                    || state.dim_a == Dimensions::ZeroDimensional
                {
                    state.is_intersects(Inside, Inside)
                        && state.is_intersects(Inside, Outside)
                        && state.is_intersects(Outside, Inside)
                } else if state.dim_a == Dimensions::OneDimensional {
                    state.is_dimension(Inside, Inside, Dimensions::OneDimensional)
                        && state.is_intersects(Inside, Outside)
                        && state.is_intersects(Outside, Inside)
                } else {
                    false
                }
            }
            PredicateKind::Touches => {
                // For touches, the interiors cannot intersect.
                state.is_intersects(Inside, Inside)
            }
        }
    }

    /// The value of the predicate according to the current matrix state.
    ///
    /// `crosses`, `equals` and `overlaps` use the dimension-guarded JTS
    /// `IntersectionMatrix` forms, evaluated from the stored input
    /// dimensions; the rest delegate to the pattern-only geo matrix
    /// predicates, which match the JTS definitions exactly.
    fn value_im(kind: PredicateKind, state: &IMState) -> bool {
        use CoordPos::{Inside, OnBoundary, Outside};
        use Dimensions::Empty;
        match kind {
            PredicateKind::Contains => state.im.is_contains(),
            PredicateKind::Within => state.im.is_within(),
            PredicateKind::Covers => state.im.is_covers(),
            PredicateKind::CoveredBy => state.im.is_coveredby(),
            PredicateKind::Crosses => match state.dim_a.cmp(&state.dim_b) {
                // [T*T******]
                std::cmp::Ordering::Less => {
                    state.get_dimension(Inside, Inside) != Empty
                        && state.get_dimension(Inside, Outside) != Empty
                }
                // [T*****T**]
                std::cmp::Ordering::Greater => {
                    state.get_dimension(Inside, Inside) != Empty
                        && state.get_dimension(Outside, Inside) != Empty
                }
                // [0********] for the L/L case; equal-dimension P/P and A/A
                // never cross.
                std::cmp::Ordering::Equal => {
                    state.dim_a == Dimensions::OneDimensional
                        && state.get_dimension(Inside, Inside) == Dimensions::ZeroDimensional
                }
            },
            // JTS `IntersectionMatrix.isEquals(dimA, dimB)`: equal
            // dimensions and no exterior intersection either way. The
            // empty-equals-empty case is settled at envelope init and never
            // reaches here. geo's `is_equal_topo` is not used because its
            // empty-matrix special case would misread the pre-seeded
            // predicate matrix.
            PredicateKind::EqualsTopo => {
                state.dim_a == state.dim_b
                    && state.get_dimension(Inside, Inside) != Empty
                    && state.get_dimension(Inside, Outside) == Empty
                    && state.get_dimension(OnBoundary, Outside) == Empty
                    && state.get_dimension(Outside, Inside) == Empty
                    && state.get_dimension(Outside, OnBoundary) == Empty
            }
            // JTS `IntersectionMatrix.isOverlaps(dimA, dimB)`.
            PredicateKind::Overlaps => match (state.dim_a, state.dim_b) {
                // [T*T***T**]
                (Dimensions::ZeroDimensional, Dimensions::ZeroDimensional)
                | (Dimensions::TwoDimensional, Dimensions::TwoDimensional) => {
                    state.get_dimension(Inside, Inside) != Empty
                        && state.get_dimension(Inside, Outside) != Empty
                        && state.get_dimension(Outside, Inside) != Empty
                }
                // [1*T***T**]
                (Dimensions::OneDimensional, Dimensions::OneDimensional) => {
                    state.get_dimension(Inside, Inside) == Dimensions::OneDimensional
                        && state.get_dimension(Inside, Outside) != Empty
                        && state.get_dimension(Outside, Inside) != Empty
                }
                _ => false,
            },
            PredicateKind::Touches => state.im.is_touches(),
        }
    }
}

impl<F: GeoFloat> TopologyPredicate<F> for NamedPredicate {
    fn name(&self) -> &'static str {
        match self.kind {
            PredicateKind::Contains => "contains",
            PredicateKind::Within => "within",
            PredicateKind::Covers => "covers",
            PredicateKind::CoveredBy => "coveredBy",
            PredicateKind::Crosses => "crosses",
            PredicateKind::EqualsTopo => "equals",
            PredicateKind::Overlaps => "overlaps",
            PredicateKind::Touches => "touches",
        }
    }

    fn requires_interaction(&self) -> bool {
        // equalsTopo allows EMPTY = EMPTY.
        self.kind != PredicateKind::EqualsTopo
    }

    fn requires_covers(&self, source: InputIndex) -> bool {
        match self.kind {
            PredicateKind::Contains | PredicateKind::Covers => source == InputIndex::A,
            PredicateKind::Within | PredicateKind::CoveredBy => source == InputIndex::B,
            _ => false,
        }
    }

    fn requires_exterior_check(&self, source: InputIndex) -> bool {
        match self.kind {
            // Only the covered geometry needs checking against the exterior
            // of the covering one.
            PredicateKind::Contains | PredicateKind::Covers => source == InputIndex::B,
            PredicateKind::Within | PredicateKind::CoveredBy => source == InputIndex::A,
            _ => true,
        }
    }

    fn init_dimensions(&mut self, dim_a: Dimensions, dim_b: Dimensions) {
        self.state.init_dimensions(dim_a, dim_b);
        match self.kind {
            PredicateKind::Contains | PredicateKind::Covers => self
                .state
                .value
                .require(is_dims_compatible_with_covers(dim_a, dim_b)),
            PredicateKind::Within | PredicateKind::CoveredBy => self
                .state
                .value
                .require(is_dims_compatible_with_covers(dim_b, dim_a)),
            PredicateKind::Crosses => {
                let is_both_points_or_areas = (dim_a == Dimensions::ZeroDimensional
                    && dim_b == Dimensions::ZeroDimensional)
                    || (dim_a == Dimensions::TwoDimensional && dim_b == Dimensions::TwoDimensional);
                self.state.value.require(!is_both_points_or_areas);
            }
            PredicateKind::EqualsTopo => {
                // Equal dimensions are not required, because EMPTY = EMPTY
                // for all dimensions.
            }
            PredicateKind::Overlaps => self.state.value.require(dim_a == dim_b),
            PredicateKind::Touches => {
                // Points have only interiors, so they cannot touch.
                let is_both_points =
                    dim_a == Dimensions::ZeroDimensional && dim_b == Dimensions::ZeroDimensional;
                self.state.value.require(!is_both_points);
            }
        }
    }

    fn init_envelopes(&mut self, env_a: Option<Rect<F>>, env_b: Option<Rect<F>>) {
        match self.kind {
            PredicateKind::Contains | PredicateKind::Covers => {
                self.state.value.require(envelope_covers(env_a, env_b));
            }
            PredicateKind::Within | PredicateKind::CoveredBy => {
                self.state.value.require(envelope_covers(env_b, env_a));
            }
            PredicateKind::EqualsTopo => {
                // Handle the EMPTY = EMPTY cases.
                self.state
                    .value
                    .set_value_if(true, env_a.is_none() && env_b.is_none());
                self.state.value.require(envelopes_equal(env_a, env_b));
            }
            PredicateKind::Crosses | PredicateKind::Overlaps | PredicateKind::Touches => {}
        }
    }

    fn update_dimension(&mut self, loc_a: CoordPos, loc_b: CoordPos, dimension: Dimensions) {
        let kind = self.kind;
        self.state.update_dimension(
            loc_a,
            loc_b,
            dimension,
            |state| Self::is_determined(kind, state),
            |state| Self::value_im(kind, state),
        );
    }

    fn finish(&mut self) {
        let kind = self.kind;
        self.state.finish(|state| Self::value_im(kind, state));
    }

    fn is_known(&self) -> bool {
        self.state.value.is_known()
    }

    fn value(&self) -> bool {
        self.state.value.value()
    }
}

#[cfg(test)]
mod tests {
    // Tests ported from JTS RelatePredicateTest.java (master, ab57bff).
    use super::super::topology_predicate::TopologyPredicate;
    use super::*;

    const A_EXT_B_INT: &str = "***.***.1**";
    const A_INT_B_INT: &str = "1**.***.***";

    const LOCATION_ORDER: [CoordPos; 3] =
        [CoordPos::Inside, CoordPos::OnBoundary, CoordPos::Outside];

    /// Applies the non-empty entries of a dotted DE-9IM string to the
    /// predicate, in row order.
    fn apply_im(im: &str, pred: &mut impl TopologyPredicate<f64>) {
        let entries: Vec<char> = im.chars().filter(|&c| c != '.').collect();
        assert_eq!(entries.len(), 9);
        for (i, &entry) in entries.iter().enumerate() {
            let dim = match entry {
                '0' => Dimensions::ZeroDimensional,
                '1' => Dimensions::OneDimensional,
                '2' => Dimensions::TwoDimensional,
                _ => continue,
            };
            pred.update_dimension(LOCATION_ORDER[i / 3], LOCATION_ORDER[i % 3], dim);
        }
    }

    fn check_predicate(mut pred: impl TopologyPredicate<f64>, im: &str, expected: bool) {
        apply_im(im, &mut pred);
        check_pred(pred, expected);
    }

    fn check_predicate_partial(mut pred: impl TopologyPredicate<f64>, im: &str, expected: bool) {
        apply_im(im, &mut pred);
        assert!(pred.is_known(), "predicate value is not known");
        check_pred(pred, expected);
    }

    fn check_pred(mut pred: impl TopologyPredicate<f64>, expected: bool) {
        pred.finish();
        assert_eq!(pred.value(), expected);
    }

    #[test]
    fn test_intersects() {
        check_predicate(intersects(), A_INT_B_INT, true);
    }

    #[test]
    fn test_disjoint() {
        check_predicate(intersects(), A_EXT_B_INT, false);
        check_predicate(disjoint(), A_EXT_B_INT, true);
    }

    #[test]
    fn test_covers() {
        check_predicate(covers(), A_INT_B_INT, true);
        check_predicate(covers(), A_EXT_B_INT, false);
    }

    #[test]
    fn test_covers_fast() {
        check_predicate_partial(covers(), A_EXT_B_INT, false);
    }

    #[test]
    fn test_match() {
        check_predicate(
            matches("1***T*0**").expect("valid pattern"),
            "1**.*2*.0**",
            true,
        );
    }

    // Not from JTS: an invalid pattern must be rejected, not panic.
    #[test]
    fn test_match_invalid_pattern() {
        assert!(matches("1***T*0*").is_err()); // eight characters
        assert!(matches("1***T*0*X").is_err()); // invalid character
    }
}
