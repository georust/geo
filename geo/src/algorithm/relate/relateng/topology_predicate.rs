//! The strategy API for DE-9IM topological predicates, and the tri-state
//! value they share.
//!
//! Port of JTS `TopologyPredicate` and `BasicPredicate`.

use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::{GeoFloat, Intersects, Rect};

/// Identifies one of the two input geometries of a relate operation.
///
/// Replaces JTS's `RelateGeometry.GEOM_A`/`GEOM_B` boolean convention.
/// The declaration order gives the "A sorts before B" ordering that
/// node-section sorting relies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum InputIndex {
    A,
    B,
}

/// The API for strategy types implementing spatial predicates based on the
/// DE-9IM topology model. Predicate values for specific geometry pairs are
/// evaluated by the RelateNG driver.
pub(crate) trait TopologyPredicate<F: GeoFloat> {
    /// The name of the predicate.
    fn name(&self) -> &'static str;

    /// Reports whether this predicate requires self-noding for geometries
    /// which contain crossing edges (for example line strings, or geometry
    /// collections containing lines or polygons which may self-intersect).
    /// Self-noding ensures that intersections are computed consistently in
    /// cases which contain self-crossings and mutual crossings.
    ///
    /// Most predicates require this, but it can be avoided for simple
    /// interaction detection (`intersects` and `disjoint`). Avoiding
    /// self-noding improves performance for polygonal inputs.
    fn requires_self_noding(&self) -> bool {
        true
    }

    /// Reports whether this predicate requires interaction between the input
    /// geometries: some of IM[I, I], IM[I, B], IM[B, I], IM[B, B] must be
    /// non-empty. This allows a fast result if the envelopes of the
    /// geometries are disjoint.
    fn requires_interaction(&self) -> bool {
        true
    }

    /// Reports whether this predicate requires that the source cover the
    /// target: IM[Ext(Src), Int(Tgt)] = F and IM[Ext(Src), Bdy(Tgt)] = F.
    /// If true, this allows a fast result if the source envelope does not
    /// cover the target envelope.
    fn requires_covers(&self, _source: InputIndex) -> bool {
        false
    }

    /// Reports whether this predicate requires checking if the source input
    /// intersects the exterior of the target input: IM[Int(Src), Ext(Tgt)]
    /// or IM[Bdy(Src), Ext(Tgt)] is non-empty. If false, this may permit a
    /// faster result in some geometric situations.
    fn requires_exterior_check(&self, _source: InputIndex) -> bool {
        true
    }

    /// Initialises the predicate for a specific geometric case. This may
    /// allow the predicate result to become known if it can be inferred from
    /// the dimensions.
    fn init_dimensions(&mut self, _dim_a: Dimensions, _dim_b: Dimensions) {
        // The default when dimensions provide no information.
    }

    /// Initialises the predicate from the input envelopes. `None` is the
    /// null envelope of an empty geometry. This may allow the predicate
    /// result to become known if it can be inferred from the envelopes.
    fn init_envelopes(&mut self, _env_a: Option<Rect<F>>, _env_b: Option<Rect<F>>) {
        // The default when envelopes provide no information.
    }

    /// Updates the entry in the DE-9IM intersection matrix for the given
    /// locations in the input geometries. An update with a dimension lower
    /// than the current value of an entry must not change the entry.
    fn update_dimension(&mut self, loc_a: CoordPos, loc_b: CoordPos, dimension: Dimensions);

    /// Indicates that the value of the predicate can be finalised based on
    /// its current state.
    fn finish(&mut self);

    /// Tests if the predicate value is known.
    fn is_known(&self) -> bool;

    /// The current value of the predicate result. The value is only valid if
    /// [`Self::is_known`] is `true`.
    fn value(&self) -> bool;
}

/// Tests if two geometries intersect based on an interaction at the given
/// locations: some location on both geometries is not the exterior.
pub(crate) fn is_intersection(loc_a: CoordPos, loc_b: CoordPos) -> bool {
    loc_a != CoordPos::Outside && loc_b != CoordPos::Outside
}

/// The tri-state predicate value: unknown until set, and the first set value
/// wins (JTS `BasicPredicate`).
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PredicateValue(Option<bool>);

impl PredicateValue {
    /// Updates the value to the given state if it is currently unknown.
    pub fn set_value(&mut self, val: bool) {
        if self.0.is_none() {
            self.0 = Some(val);
        }
    }

    pub fn set_value_if(&mut self, val: bool, cond: bool) {
        if cond {
            self.set_value(val);
        }
    }

    /// Sets the value to `false` unless the condition holds.
    pub fn require(&mut self, cond: bool) {
        if !cond {
            self.set_value(false);
        }
    }

    pub fn is_known(&self) -> bool {
        self.0.is_some()
    }

    pub fn value(&self) -> bool {
        self.0 == Some(true)
    }
}

// Envelope operations with JTS `Envelope` semantics: `None` is the null
// envelope of an empty geometry, which intersects, covers, and equals
// nothing (except that two null envelopes are equal).

pub(crate) fn envelopes_intersect<F: GeoFloat>(a: Option<Rect<F>>, b: Option<Rect<F>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.intersects(&b),
        _ => false,
    }
}

pub(crate) fn envelope_covers<F: GeoFloat>(a: Option<Rect<F>>, b: Option<Rect<F>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            a.min().x <= b.min().x
                && b.max().x <= a.max().x
                && a.min().y <= b.min().y
                && b.max().y <= a.max().y
        }
        _ => false,
    }
}

pub(crate) fn envelopes_equal<F: GeoFloat>(a: Option<Rect<F>>, b: Option<Rect<F>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a == b,
        (None, None) => true,
        _ => false,
    }
}
