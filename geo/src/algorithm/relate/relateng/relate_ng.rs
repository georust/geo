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
use crate::dimensions::Dimensions;
use crate::geometry_cow::GeometryCow;
use crate::rect_ops::RectOps;
use crate::relate::IntersectionMatrix;
use crate::{Coord, GeoFloat, Geometry, Line, LineString, Polygon, Rect};

use super::edge_segment_intersector::{
    EdgeSegmentIntersector, MutualSegmentSetIntersector, intersect_all,
};
use super::im_predicate::RelateMatrixPredicate;
use super::relate_geometry::RelateGeometry;
use super::topology_computer::TopologyComputer;
use super::topology_predicate::{
    InputIndex, TopologyPredicate, envelope_covers, envelopes_intersect,
};

/// Computes the DE-9IM matrix for a pair of geometries.
pub(crate) fn relate<F: GeoFloat>(
    a: &GeometryCow<'_, F>,
    b: &GeometryCow<'_, F>,
) -> IntersectionMatrix {
    RelateNG::new(a).evaluate_matrix(b)
}

/// Evaluates a topological predicate for a pair of geometries, with
/// short-circuiting.
pub(crate) fn eval<F: GeoFloat>(
    a: &GeometryCow<'_, F>,
    b: &GeometryCow<'_, F>,
    predicate: &mut dyn TopologyPredicate<F>,
) -> bool {
    RelateNG::new(a).evaluate(b, predicate)
}

pub(crate) struct RelateNG<'a, F: GeoFloat> {
    geom_a: RelateGeometry<'a, F>,
    /// The cached A-side segment index. In prepared mode it is built over
    /// all A edges and reused across evaluations; otherwise it is built
    /// filtered to the first evaluation's envelope (a non-prepared
    /// instance is single-use, as in JTS).
    edge_mutual_int: OnceCell<MutualSegmentSetIntersector<F>>,
}

impl<'a, F: GeoFloat> RelateNG<'a, F> {
    pub fn new(a: &'a GeometryCow<'a, F>) -> Self {
        Self::new_with_prepared(a, false)
    }

    /// Creates an instance with cached spatial indexes, for repeated
    /// evaluations against the same A geometry.
    pub fn prepare(a: &'a GeometryCow<'a, F>) -> Self {
        Self::new_with_prepared(a, true)
    }

    fn new_with_prepared(a: &'a GeometryCow<'a, F>, is_prepared: bool) -> Self {
        Self {
            geom_a: RelateGeometry::new_with_prepared(a, is_prepared),
            edge_mutual_int: OnceCell::new(),
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
        // Fast envelope checks.
        if !self.has_required_envelope_interaction(b, predicate) {
            return false;
        }

        let geom_b = RelateGeometry::new(b);

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
        b: &GeometryCow<'_, F>,
        predicate: &dyn TopologyPredicate<F>,
    ) -> bool {
        let env_b = b.bounding_rect();
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
        let pts_a = self.geom_a.unique_points();
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
            let mutual = self.edge_mutual_int.get_or_init(|| {
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
    let dim_target = loc_dim_target.dimension_with_exterior(computer.dimension(other(input)));
    computer.add_point_on_geometry(input, loc_target, dim_target, pt);
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

    for elem in linear_elements(geom.geometry()) {
        let (e0, e1, is_closed, elem_env) = match elem {
            LinearElement::LineString(ls) => {
                let first = ls.0[0];
                let last = ls.0[ls.0.len() - 1];
                (first, last, ls.is_closed(), ls.bounding_rect())
            }
            LinearElement::Line(l) => (
                l.start,
                l.end,
                l.start == l.end,
                Some(Rect::new(l.start, l.end)),
            ),
        };

        // Once intersections with the target exterior are recorded for
        // both interior and boundary line ends, skip further
        // known-exterior line components.
        if has_interior_exterior_intersection
            && has_boundary_exterior_intersection
            && !envelopes_intersect(elem_env, geom_target.envelope())
        {
            continue;
        }

        let loc0 = compute_line_end(geom, input, e0, geom_target, computer);
        match loc0 {
            Some(CoordPos::Inside) => has_interior_exterior_intersection = true,
            Some(CoordPos::OnBoundary) => has_boundary_exterior_intersection = true,
            _ => {}
        }
        if computer.is_result_known() {
            return true;
        }

        if !is_closed {
            let loc1 = compute_line_end(geom, input, e1, geom_target, computer);
            match loc1 {
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
    let dim_target = loc_dim_target.dimension_with_exterior(computer.dimension(other(input)));
    computer.add_line_end_on_geometry(input, loc_line_end, loc_target, dim_target, pt);
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

    for elem in area_elements(geom.geometry()) {
        let polygon = elem.polygon();
        if polygon.exterior().0.is_empty() {
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
    let dim_target = loc_dim_target.dimension_with_exterior(computer.dimension(other(input)));
    computer.add_area_vertex(input, loc_area, loc_target, dim_target, pt);
    loc_target == CoordPos::Outside
}

fn other(input: InputIndex) -> InputIndex {
    match input {
        InputIndex::A => InputIndex::B,
        InputIndex::B => InputIndex::A,
    }
}

/// The linear elements of a geometry (LineStrings and Lines), in document
/// order, skipping empty ones.
enum LinearElement<'g, F: GeoFloat> {
    LineString(&'g LineString<F>),
    Line(&'g Line<F>),
}

fn linear_elements<'g, F: GeoFloat>(geom: &'g GeometryCow<'g, F>) -> Vec<LinearElement<'g, F>> {
    let mut elems = Vec::new();
    match geom {
        GeometryCow::Line(l) => elems.push(LinearElement::Line(l)),
        GeometryCow::LineString(ls) => {
            if !ls.0.is_empty() {
                elems.push(LinearElement::LineString(ls));
            }
        }
        GeometryCow::MultiLineString(mls) => {
            for ls in &mls.0 {
                if !ls.0.is_empty() {
                    elems.push(LinearElement::LineString(ls));
                }
            }
        }
        GeometryCow::GeometryCollection(gc) => {
            for g in &gc.0 {
                collect_linear_elements(g, &mut elems);
            }
        }
        _ => {}
    }
    elems
}

fn collect_linear_elements<'g, F: GeoFloat>(
    geom: &'g Geometry<F>,
    elems: &mut Vec<LinearElement<'g, F>>,
) {
    match geom {
        Geometry::Line(l) => elems.push(LinearElement::Line(l)),
        Geometry::LineString(ls) => {
            if !ls.0.is_empty() {
                elems.push(LinearElement::LineString(ls));
            }
        }
        Geometry::MultiLineString(mls) => {
            for ls in &mls.0 {
                if !ls.0.is_empty() {
                    elems.push(LinearElement::LineString(ls));
                }
            }
        }
        Geometry::GeometryCollection(gc) => {
            for g in &gc.0 {
                collect_linear_elements(g, elems);
            }
        }
        _ => {}
    }
}

/// The polygonal elements of a geometry, in document order. Rect and
/// Triangle elements are converted to owned polygons.
enum AreaElement<'g, F: GeoFloat> {
    Polygon(&'g Polygon<F>),
    Owned(Polygon<F>),
}

impl<F: GeoFloat> AreaElement<'_, F> {
    fn polygon(&self) -> &Polygon<F> {
        match self {
            AreaElement::Polygon(p) => p,
            AreaElement::Owned(p) => p,
        }
    }
}

fn area_elements<'g, F: GeoFloat>(geom: &'g GeometryCow<'g, F>) -> Vec<AreaElement<'g, F>> {
    let mut elems = Vec::new();
    match geom {
        GeometryCow::Polygon(p) => elems.push(AreaElement::Polygon(p)),
        GeometryCow::MultiPolygon(mp) => {
            elems.extend(mp.0.iter().map(AreaElement::Polygon));
        }
        GeometryCow::Rect(r) => elems.push(AreaElement::Owned(r.to_polygon())),
        GeometryCow::Triangle(t) => elems.push(AreaElement::Owned(t.to_polygon())),
        GeometryCow::GeometryCollection(gc) => {
            for g in &gc.0 {
                collect_area_elements(g, &mut elems);
            }
        }
        _ => {}
    }
    elems
}

fn collect_area_elements<'g, F: GeoFloat>(
    geom: &'g Geometry<F>,
    elems: &mut Vec<AreaElement<'g, F>>,
) {
    match geom {
        Geometry::Polygon(p) => elems.push(AreaElement::Polygon(p)),
        Geometry::MultiPolygon(mp) => {
            elems.extend(mp.0.iter().map(AreaElement::Polygon));
        }
        Geometry::Rect(r) => elems.push(AreaElement::Owned(r.to_polygon())),
        Geometry::Triangle(t) => elems.push(AreaElement::Owned(t.to_polygon())),
        Geometry::GeometryCollection(gc) => {
            for g in &gc.0 {
                collect_area_elements(g, elems);
            }
        }
        _ => {}
    }
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
        assert_eq!(format!("{im:?}"), "IntersectionMatrix(1F1F00FF2)");
        assert!(im.is_contains());

        let mut contains = super::super::relate_predicate::contains();
        assert!(eval(&cow_mls, &cow_m0, &mut contains));
    }
}
