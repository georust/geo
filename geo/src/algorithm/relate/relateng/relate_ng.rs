//! The RelateNG driver: computes the value of topological predicates (or
//! the full DE-9IM matrix) between two geometries.
//!
//! Port of JTS `RelateNG`.
//!
//! Evaluation proceeds in phases, with predicate short-circuit checks
//! after each step: envelope gates, dimension and envelope predicate
//! initialisation, an optimised point/point path, point phases (points,
//! line ends and area vertices located on the opposing geometry), the
//! edge phase (segment intersection detection and node evaluation), and a
//! final predicate finish. No noded topology graph is built and no
//! constructed intersection points enter shared topology, which avoids
//! the robustness failures of the graph-based engine (see georust/geo
//! issue #1585).

use std::cell::OnceCell;

use crate::bounding_rect::BoundingRect;
use crate::coordinate_position::CoordPos;
use crate::dimensions::{Dimensions, HasDimensions};
use crate::geometry_cow::GeometryCow;
use crate::rect_ops::RectOps;
use crate::relate::IntersectionMatrix;
use crate::{Coord, GeoFloat, LineString, Rect};

use super::edge_segment_intersector::{
    EdgeSegmentIntersector, MutualSegmentSetIntersector, intersect_all,
};
use super::im_predicate::RelateMatrixPredicate;
use super::relate_geometry::{GeometryMeta, RelateGeometry};
use super::relate_point_locator::{Mode, Polygonal, PreparedLocatorCache};
use super::topology_computer::TopologyComputer;
use super::topology_predicate::{
    InputIndex, TopologyPredicate, envelope_covers, envelopes_intersect,
};

/// Computes the DE-9IM matrix for a pair of geometries.
pub(crate) fn relate<F: GeoFloat>(
    a: &GeometryCow<'_, F>,
    b: &GeometryCow<'_, F>,
) -> IntersectionMatrix {
    let state = PreparedRelateState::default();
    RelateNG::new(a, &state).evaluate_matrix(b)
}

/// Evaluates a topological predicate for a pair of geometries, with
/// short-circuiting.
pub(crate) fn eval<F: GeoFloat>(
    a: &GeometryCow<'_, F>,
    b: &GeometryCow<'_, F>,
    predicate: &mut dyn TopologyPredicate<F>,
) -> bool {
    let state = PreparedRelateState::default();
    RelateNG::new(a, &state).evaluate(b, predicate)
}

/// The A-side caches of a prepared relate evaluation. The state is fully
/// owned (segment strings own their coordinates, the area indexes are
/// owned), so it can live in a `PreparedGeometry` and be shared into
/// per-call engines without borrowing the geometry.
pub(crate) struct PreparedRelateState<F: GeoFloat> {
    /// The A-side segment index, built over all A edges on first use.
    edge_mutual_int: OnceCell<MutualSegmentSetIntersector<F>>,
    /// The point-locator state of A that costs a coordinate walk.
    locator_cache: PreparedLocatorCache<F>,
    /// The unique points of A (for the P/P fast path).
    unique_points:
        OnceCell<std::collections::BTreeSet<crate::relate::geomgraph::node_map::NodeKey<F>>>,
    /// The envelope and dimension analysis of A.
    meta: OnceCell<GeometryMeta<F>>,
}

impl<F: GeoFloat> Default for PreparedRelateState<F> {
    fn default() -> Self {
        Self {
            edge_mutual_int: OnceCell::new(),
            locator_cache: PreparedLocatorCache::default(),
            unique_points: OnceCell::new(),
            meta: OnceCell::new(),
        }
    }
}

impl<F: GeoFloat> PreparedRelateState<F> {
    /// The metadata of the A geometry, computed on first use.
    pub fn meta(&self, a: &GeometryCow<'_, F>) -> &GeometryMeta<F> {
        self.meta.get_or_init(|| GeometryMeta::of(a))
    }
}

pub(crate) struct RelateNG<'a, F: GeoFloat> {
    geom_a: RelateGeometry<'a, F>,
    /// The A-side caches. In prepared mode the segment index is built over
    /// all A edges and reused across evaluations; otherwise it is built
    /// filtered to the first evaluation's envelope, so a non-prepared
    /// instance is single-use, as in JTS.
    state: &'a PreparedRelateState<F>,
}

impl<'a, F: GeoFloat> RelateNG<'a, F> {
    /// An engine for a single evaluation.
    pub fn new(a: &'a GeometryCow<'a, F>, state: &'a PreparedRelateState<F>) -> Self {
        Self {
            geom_a: RelateGeometry::new(a, Mode::Simple),
            state,
        }
    }

    /// An engine for repeated evaluations against the same A geometry:
    /// the indexes are stored in the state and reused by every engine
    /// constructed over it.
    pub fn prepared(a: &'a GeometryCow<'a, F>, state: &'a PreparedRelateState<F>) -> Self {
        Self {
            geom_a: RelateGeometry::with_meta(
                a,
                Mode::Prepared(&state.locator_cache),
                *state.meta(a),
            ),
            state,
        }
    }

    /// Computes the full DE-9IM matrix against the B geometry.
    pub fn evaluate_matrix(&self, b: &GeometryCow<'_, F>) -> IntersectionMatrix {
        let mut predicate = RelateMatrixPredicate::new();
        self.evaluate(b, &mut predicate);
        predicate.into_im()
    }

    /// Evaluates a topological predicate against the B geometry.
    pub fn evaluate(
        &self,
        b: &GeometryCow<'_, F>,
        predicate: &mut dyn TopologyPredicate<F>,
    ) -> bool {
        // The B wrapper is built first so the envelope gate reuses its
        // cached envelope instead of walking B's coordinates twice (JTS
        // gets the gate's envelope for free from the Geometry cache).
        let geom_b = RelateGeometry::new(b, Mode::Simple);

        // Fast envelope checks.
        if !self.has_required_envelope_interaction(geom_b.envelope(), predicate) {
            return false;
        }

        let dim_a = self.geom_a.dimension_real();
        let dim_b = geom_b.dimension_real();

        // Check if the predicate is determined by the dimensions or the
        // envelopes.
        predicate.init_dimensions(dim_a, dim_b);
        if predicate.is_known() {
            return finish_value(predicate);
        }
        predicate.init_envelopes(self.geom_a.envelope(), geom_b.envelope());
        if predicate.is_known() {
            return finish_value(predicate);
        }

        let mut computer = TopologyComputer::new(predicate, &self.geom_a, &geom_b);

        // Optimised P/P evaluation.
        if dim_a == Dimensions::ZeroDimensional && dim_b == Dimensions::ZeroDimensional {
            self.compute_pp(&geom_b, &mut computer);
            computer.finish();
            return computer.result();
        }

        // Test points against the (potentially indexed) geometry first.
        compute_at_points(&geom_b, InputIndex::B, &self.geom_a, &mut computer);
        if computer.is_result_known() {
            return computer.result();
        }
        compute_at_points(&self.geom_a, InputIndex::A, &geom_b, &mut computer);
        if computer.is_result_known() {
            return computer.result();
        }

        if self.geom_a.has_edges() && geom_b.has_edges() {
            self.compute_at_edges(&geom_b, &mut computer);
        }

        // After all processing, set the remaining unknown values.
        computer.finish();
        computer.result()
    }

    fn has_required_envelope_interaction(
        &self,
        env_b: Option<Rect<F>>,
        predicate: &dyn TopologyPredicate<F>,
    ) -> bool {
        let env_a = self.geom_a.envelope();
        let mut is_interacts = false;
        if predicate.requires_covers(InputIndex::A) {
            if !envelope_covers(env_a, env_b) {
                return false;
            }
            is_interacts = true;
        } else if predicate.requires_covers(InputIndex::B) {
            if !envelope_covers(env_b, env_a) {
                return false;
            }
            is_interacts = true;
        }
        if !is_interacts && predicate.requires_interaction() && !envelopes_intersect(env_a, env_b) {
            return false;
        }
        true
    }

    /// An optimised algorithm for evaluating P/P cases: tests one point
    /// set against the other.
    fn compute_pp(
        &self,
        geom_b: &RelateGeometry<'_, F>,
        computer: &mut TopologyComputer<'_, '_, '_, F>,
    ) {
        // The A-side unique points live in the (possibly shared) state so
        // prepared use computes them once.
        let pts_a = self
            .state
            .unique_points
            .get_or_init(|| self.geom_a.compute_unique_points());
        let pts_b = geom_b.unique_points();

        let mut num_b_in_a = 0;
        for pt_b in pts_b {
            if pts_a.contains(pt_b) {
                num_b_in_a += 1;
                computer.add_point_on_point_interior();
            } else {
                computer.add_point_on_point_exterior(InputIndex::B);
            }
            if computer.is_result_known() {
                return;
            }
        }
        // If the number of matched B points is less than the size of A,
        // there must be at least one A point in the exterior of B.
        if num_b_in_a < pts_a.len() {
            computer.add_point_on_point_exterior(InputIndex::A);
        }
    }

    fn compute_at_edges(
        &self,
        geom_b: &RelateGeometry<'_, F>,
        computer: &mut TopologyComputer<'_, '_, '_, F>,
    ) {
        let (Some(env_a), Some(env_b)) = (self.geom_a.envelope(), geom_b.envelope()) else {
            return;
        };
        let Some(env_int) = env_a.rect_intersection(env_b) else {
            return;
        };

        let edges_b = geom_b.extract_segment_strings(InputIndex::B, Some(&env_int));

        if computer.is_self_noding_required() {
            let edges_a = self
                .geom_a
                .extract_segment_strings(InputIndex::A, Some(&env_int));
            let mut intersector = EdgeSegmentIntersector::new(computer);
            intersect_all(&edges_a, &edges_b, Some(&env_int), &mut intersector);
        } else {
            // In prepared mode the A edge index is reused across
            // evaluations.
            let mutual = self.state.edge_mutual_int.get_or_init(|| {
                let env_extract = if self.geom_a.is_prepared() {
                    None
                } else {
                    Some(&env_int)
                };
                let edges_a = self
                    .geom_a
                    .extract_segment_strings(InputIndex::A, env_extract);
                MutualSegmentSetIntersector::new(edges_a, env_extract)
            });
            let mut intersector = EdgeSegmentIntersector::new(computer);
            mutual.process(&edges_b, &mut intersector);
        }
        if computer.is_result_known() {
            return;
        }

        computer.evaluate_nodes();
    }
}

fn finish_value<F: GeoFloat>(predicate: &mut dyn TopologyPredicate<F>) -> bool {
    predicate.finish();
    predicate.value()
}

fn compute_at_points<F: GeoFloat>(
    geom: &RelateGeometry<'_, F>,
    input: InputIndex,
    geom_target: &RelateGeometry<'_, F>,
    computer: &mut TopologyComputer<'_, '_, '_, F>,
) {
    if compute_points(geom, input, geom_target, computer) {
        return;
    }

    // Performance optimisation: only check line ends and area vertices
    // against the target if it has areas, or if the predicate requires
    // checking for exterior interaction. In particular this avoids
    // testing line ends against lines for the intersects predicate
    // (they are checked during segment intersection anyway). Checking
    // points against areas is necessary, since the input linework is
    // disjoint if one input lies wholly inside an area.
    let check_disjoint_points = geom_target.has_dimension(Dimensions::TwoDimensional)
        || computer.is_exterior_check_required(input);
    if !check_disjoint_points {
        return;
    }

    if compute_line_ends(geom, input, geom_target, computer) {
        return;
    }

    compute_area_vertex(geom, input, geom_target, computer);
}

fn compute_points<F: GeoFloat>(
    geom: &RelateGeometry<'_, F>,
    input: InputIndex,
    geom_target: &RelateGeometry<'_, F>,
    computer: &mut TopologyComputer<'_, '_, '_, F>,
) -> bool {
    if !geom.has_dimension(Dimensions::ZeroDimensional) {
        return false;
    }

    for pt in geom.effective_points() {
        compute_point(input, pt, geom_target, computer);
        if computer.is_result_known() {
            return true;
        }
    }
    false
}

fn compute_point<F: GeoFloat>(
    input: InputIndex,
    pt: Coord<F>,
    geom_target: &RelateGeometry<'_, F>,
    computer: &mut TopologyComputer<'_, '_, '_, F>,
) {
    let loc_dim_target = geom_target.locate_with_dim(pt);
    let loc_target = loc_dim_target.location();
    let dim_target = loc_dim_target.dimension_with_exterior(computer.dimension(input.other()));
    computer.add_point_on_geometry(input, loc_target, dim_target);
}

fn compute_line_ends<F: GeoFloat>(
    geom: &RelateGeometry<'_, F>,
    input: InputIndex,
    geom_target: &RelateGeometry<'_, F>,
    computer: &mut TopologyComputer<'_, '_, '_, F>,
) -> bool {
    if !geom.has_dimension(Dimensions::OneDimensional) {
        return false;
    }

    // Track exterior intersections of interior and boundary line ends
    // separately (JTS PR #1200): only once both are recorded may
    // known-exterior line components be skipped.
    let mut has_interior_exterior_intersection = false;
    let mut has_boundary_exterior_intersection = false;

    for elem in geom.lines() {
        let coords = &elem.line.0;
        let e0 = coords[0];
        let e1 = coords[coords.len() - 1];

        // Once intersections with the target exterior are recorded for
        // both interior and boundary line ends, skip further
        // known-exterior line components.
        if has_interior_exterior_intersection
            && has_boundary_exterior_intersection
            && !envelopes_intersect(elem.env, geom_target.envelope())
        {
            continue;
        }

        // A closed line has a single end point.
        let ends: &[Coord<F>] = if e0 == e1 { &[e0] } else { &[e0, e1] };
        for &pt in ends {
            match compute_line_end(geom, input, pt, geom_target, computer) {
                Some(CoordPos::Inside) => has_interior_exterior_intersection = true,
                Some(CoordPos::OnBoundary) => has_boundary_exterior_intersection = true,
                _ => {}
            }
            if computer.is_result_known() {
                return true;
            }
        }
    }
    false
}

/// Computes the topology of a line endpoint. Returns the location of the
/// line end (interior or boundary) if it lies in the exterior of the
/// target, to allow skipping further exterior endpoints; `None` otherwise.
fn compute_line_end<F: GeoFloat>(
    geom: &RelateGeometry<'_, F>,
    input: InputIndex,
    pt: Coord<F>,
    geom_target: &RelateGeometry<'_, F>,
    computer: &mut TopologyComputer<'_, '_, '_, F>,
) -> Option<CoordPos> {
    let loc_dim_line_end = geom.locate_line_end_with_dim(pt);
    let dim_line_end = loc_dim_line_end.dimension_with_exterior(computer.dimension(input));
    // Skip line ends which are in a collection area.
    if dim_line_end != Dimensions::OneDimensional {
        return None;
    }
    let loc_line_end = loc_dim_line_end.location();

    let loc_dim_target = geom_target.locate_with_dim(pt);
    let loc_target = loc_dim_target.location();
    let dim_target = loc_dim_target.dimension_with_exterior(computer.dimension(input.other()));
    computer.add_line_end_on_geometry(input, loc_line_end, loc_target, dim_target);
    if loc_target == CoordPos::Outside {
        return Some(loc_line_end);
    }
    None
}

fn compute_area_vertex<F: GeoFloat>(
    geom: &RelateGeometry<'_, F>,
    input: InputIndex,
    geom_target: &RelateGeometry<'_, F>,
    computer: &mut TopologyComputer<'_, '_, '_, F>,
) -> bool {
    if !geom.has_dimension(Dimensions::TwoDimensional) {
        return false;
    }
    // Evaluate for line and area targets only, since points are handled
    // in the reverse direction.
    if geom_target.dimension() < Dimensions::OneDimensional {
        return false;
    }

    let mut has_exterior_intersection = false;

    for polygon in geom.polygonals().iter().flat_map(Polygonal::polygons) {
        if polygon.is_empty() {
            continue;
        }
        // Once an intersection with the target exterior is recorded, skip
        // further known-exterior elements.
        if has_exterior_intersection
            && !envelopes_intersect(polygon.bounding_rect(), geom_target.envelope())
        {
            continue;
        }

        has_exterior_intersection |=
            compute_area_vertex_on_ring(geom, input, polygon.exterior(), geom_target, computer);
        if computer.is_result_known() {
            return true;
        }
        for hole in polygon.interiors() {
            has_exterior_intersection |=
                compute_area_vertex_on_ring(geom, input, hole, geom_target, computer);
            if computer.is_result_known() {
                return true;
            }
        }
    }
    false
}

fn compute_area_vertex_on_ring<F: GeoFloat>(
    geom: &RelateGeometry<'_, F>,
    input: InputIndex,
    ring: &LineString<F>,
    geom_target: &RelateGeometry<'_, F>,
    computer: &mut TopologyComputer<'_, '_, '_, F>,
) -> bool {
    if ring.0.is_empty() {
        return false;
    }
    let pt = ring.0[0];

    let loc_area = geom.locate_area_vertex(pt);
    let loc_dim_target = geom_target.locate_with_dim(pt);
    let loc_target = loc_dim_target.location();
    let dim_target = loc_dim_target.dimension_with_exterior(computer.dimension(input.other()));
    computer.add_area_vertex(input, loc_area, loc_target, dim_target);
    loc_target == CoordPos::Outside
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wkt;

    // Regression test for georust/geo issue #1585: a MultiLineString must
    // contain its own member, even when another member crosses it at a
    // point which is not exactly representable. The old graph-based
    // engine returns the incorrect matrix 0F1F001F2 here.
    #[test]
    fn test_issue_1585_multilinestring_contains_own_member() {
        let m0 = wkt!(LINESTRING (-1.0 1.0, -4.0 -1.0));
        let m1 = wkt!(LINESTRING (-85.38966060611993 -1.0, 0.0 0.0));
        let mls = crate::MultiLineString::new(vec![m0.clone(), m1]);

        let cow_mls = GeometryCow::from(&mls);
        let cow_m0 = GeometryCow::from(&m0);

        let im = relate(&cow_mls, &cow_m0);
        assert_eq!(im, "1F1F00FF2".parse().unwrap());
        assert!(im.is_contains());

        let mut contains = super::super::relate_predicate::contains();
        assert!(eval(&cow_mls, &cow_m0, &mut contains));
    }
}
