//! The edge phase of a relate evaluation: finds intersections between the
//! segments of the input geometries and feeds them to the
//! [`TopologyComputer`].
//!
//! Port of JTS `EdgeSegmentIntersector`, plus the candidate-pair sweeps
//! that replace JTS `MCIndexSegmentSetMutualIntersector` (the mutual,
//! base-indexed case) and `EdgeSetIntersector` (the self-noding union
//! case). Where JTS indexes monotone chains in an HPRtree, this port
//! indexes individual segments in an [`rstar::RTree`] – the established
//! pattern of the geomgraph edge-set intersector. A chain-level index is a
//! possible follow-up if benchmarks justify it.

use rstar::{AABB, RTree};

use crate::line_intersection::{LineIntersection, line_intersection};
use crate::relate::geomgraph::index::Segment;
use crate::{Coord, GeoFloat, Intersects, Line, Rect};

use super::relate_segment_string::RelateSegmentString;
use super::topology_computer::TopologyComputer;

/// Tests segments of [`RelateSegmentString`]s and adds any intersections
/// to the [`TopologyComputer`].
pub(crate) struct EdgeSegmentIntersector<'c, 'r, 'a, 'b, F: GeoFloat> {
    topo_computer: &'c mut TopologyComputer<'r, 'a, 'b, F>,
}

impl<'c, 'r, 'a, 'b, F: GeoFloat> EdgeSegmentIntersector<'c, 'r, 'a, 'b, F> {
    pub fn new(topo_computer: &'c mut TopologyComputer<'r, 'a, 'b, F>) -> Self {
        Self { topo_computer }
    }

    /// Whether the sweep can stop because the predicate value is known.
    pub fn is_done(&self) -> bool {
        self.topo_computer.is_result_known()
    }

    /// Processes one candidate segment pair. The caller must not pass a
    /// segment paired with itself.
    pub fn process_intersections(
        &mut self,
        ss0: &RelateSegmentString<F>,
        seg_index0: usize,
        ss1: &RelateSegmentString<F>,
        seg_index1: usize,
    ) {
        // Order the arguments so an A-input string comes first.
        if ss0.is_a() {
            self.add_intersections(ss0, seg_index0, ss1, seg_index1);
        } else {
            self.add_intersections(ss1, seg_index1, ss0, seg_index0);
        }
    }

    fn add_intersections(
        &mut self,
        ss_a: &RelateSegmentString<F>,
        seg_index_a: usize,
        ss_b: &RelateSegmentString<F>,
        seg_index_b: usize,
    ) {
        let a0 = ss_a.coord(seg_index_a);
        let a1 = ss_a.coord(seg_index_a + 1);
        let b0 = ss_b.coord(seg_index_b);
        let b1 = ss_b.coord(seg_index_b + 1);

        match line_intersection(Line::new(a0, a1), Line::new(b0, b1)) {
            None => {}
            Some(LineIntersection::SinglePoint {
                intersection,
                is_proper,
            }) => {
                self.add_intersection_point(
                    ss_a,
                    seg_index_a,
                    ss_b,
                    seg_index_b,
                    intersection,
                    is_proper,
                );
            }
            Some(LineIntersection::Collinear { intersection }) => {
                // A collinear overlap contributes both overlap endpoints,
                // neither of which is proper.
                for int_pt in [intersection.start, intersection.end] {
                    self.add_intersection_point(
                        ss_a,
                        seg_index_a,
                        ss_b,
                        seg_index_b,
                        int_pt,
                        false,
                    );
                }
            }
        }
    }

    fn add_intersection_point(
        &mut self,
        ss_a: &RelateSegmentString<F>,
        seg_index_a: usize,
        ss_b: &RelateSegmentString<F>,
        seg_index_b: usize,
        int_pt: Coord<F>,
        is_proper: bool,
    ) {
        // Ensure endpoint intersections are added once only, for their
        // canonical segments. Proper intersections lie on a unique segment
        // so do not need the check – and must not use it, since roundoff
        // in the computed intersection point makes it unreliable there.
        if is_proper
            || (ss_a.is_containing_segment(seg_index_a, int_pt)
                && ss_b.is_containing_segment(seg_index_b, int_pt))
        {
            let nsa = ss_a.create_node_section(seg_index_a, int_pt);
            let nsb = ss_b.create_node_section(seg_index_b, int_pt);
            self.topo_computer.add_intersection(nsa, nsb);
        }
    }
}

/// Finds intersections between a base set of segment strings (indexed
/// once, reusable across queries) and a query set. Fills the role of JTS
/// `MCIndexSegmentSetMutualIntersector`.
pub(crate) struct MutualSegmentSetIntersector<F: GeoFloat> {
    base: Vec<RelateSegmentString<F>>,
    /// Segment payload: (string index in `base`, segment index).
    tree: RTree<Segment<F, (usize, usize)>>,
}

impl<F: GeoFloat> MutualSegmentSetIntersector<F> {
    /// Builds the index over the base segments, keeping only segments
    /// intersecting the envelope (if given).
    pub fn new(base: Vec<RelateSegmentString<F>>, env: Option<&Rect<F>>) -> Self {
        let mut segments = Vec::new();
        for (string_index, ss) in base.iter().enumerate() {
            for seg_index in 0..ss.size().saturating_sub(1) {
                let p0 = ss.coord(seg_index);
                let p1 = ss.coord(seg_index + 1);
                if env_intersects_segment(env, p0, p1) {
                    segments.push(Segment::new((string_index, seg_index), p0, p1));
                }
            }
        }
        Self {
            base,
            tree: RTree::bulk_load(segments),
        }
    }

    /// Processes each query segment against the indexed base segments,
    /// stopping early once the predicate value is known.
    pub fn process(
        &self,
        query: &[RelateSegmentString<F>],
        intersector: &mut EdgeSegmentIntersector<'_, '_, '_, '_, F>,
    ) {
        for query_string in query {
            for query_seg in 0..query_string.size().saturating_sub(1) {
                let p0 = query_string.coord(query_seg);
                let p1 = query_string.coord(query_seg + 1);
                let env = AABB::from_corners(p0, p1);
                for candidate in self.tree.locate_in_envelope_intersecting(env) {
                    let (base_string, base_seg) = candidate.payload;
                    intersector.process_intersections(
                        &self.base[base_string],
                        base_seg,
                        query_string,
                        query_seg,
                    );
                    if intersector.is_done() {
                        return;
                    }
                }
            }
        }
    }
}

/// Finds all intersections within the union of two segment-string sets,
/// including self-intersections. Fills the role of JTS
/// `EdgeSetIntersector`, used when self-noding is required.
pub(crate) fn intersect_all<F: GeoFloat>(
    edges_a: &[RelateSegmentString<F>],
    edges_b: &[RelateSegmentString<F>],
    env: Option<&Rect<F>>,
    intersector: &mut EdgeSegmentIntersector<'_, '_, '_, '_, F>,
) {
    // Combined string indexing: A strings first, then B strings.
    let string = |index: usize| -> &RelateSegmentString<F> {
        if index < edges_a.len() {
            &edges_a[index]
        } else {
            &edges_b[index - edges_a.len()]
        }
    };
    let string_count = edges_a.len() + edges_b.len();

    let mut segments = Vec::new();
    for string_index in 0..string_count {
        let ss = string(string_index);
        for seg_index in 0..ss.size().saturating_sub(1) {
            let p0 = ss.coord(seg_index);
            let p1 = ss.coord(seg_index + 1);
            if env_intersects_segment(env, p0, p1) {
                segments.push(Segment::new((string_index, seg_index), p0, p1));
            }
        }
    }
    let tree: RTree<Segment<F, (usize, usize)>> = RTree::bulk_load(segments.clone());

    for query in &segments {
        let (query_string, query_seg) = query.payload;
        for candidate in tree.locate_in_envelope_intersecting(query.envelope) {
            let (cand_string, cand_seg) = candidate.payload;
            // Compare each pair once, and never a segment with itself.
            if (cand_string, cand_seg) <= (query_string, query_seg) {
                continue;
            }
            intersector.process_intersections(
                string(query_string),
                query_seg,
                string(cand_string),
                cand_seg,
            );
            if intersector.is_done() {
                return;
            }
        }
    }
}

fn env_intersects_segment<F: GeoFloat>(env: Option<&Rect<F>>, p0: Coord<F>, p1: Coord<F>) -> bool {
    match env {
        None => true,
        Some(env) => env.intersects(&Rect::new(p0, p1)),
    }
}

#[cfg(test)]
mod tests {
    // Not from JTS: a plumbing test for the edge phase ahead of the
    // driver. Two crossing lines must produce a 0-dimensional
    // interior/interior intersection via node evaluation.
    use super::super::im_predicate::RelateMatrixPredicate;
    use super::super::relate_geometry::RelateGeometry;
    use super::super::topology_computer::TopologyComputer;
    use super::super::topology_predicate::InputIndex;
    use super::*;
    use crate::coordinate_position::CoordPos;
    use crate::dimensions::Dimensions;
    use crate::geometry_cow::GeometryCow;
    use crate::wkt;

    #[test]
    fn crossing_lines_interiors_intersect_zero_dimensional() {
        let a = wkt!(LINESTRING (0. 0., 2. 2.));
        let b = wkt!(LINESTRING (0. 2., 2. 0.));
        let cow_a = GeometryCow::from(&a);
        let cow_b = GeometryCow::from(&b);
        let geom_a = RelateGeometry::new(&cow_a);
        let geom_b = RelateGeometry::new(&cow_b);

        let mut predicate = RelateMatrixPredicate::new();
        {
            let mut computer = TopologyComputer::new(&mut predicate, &geom_a, &geom_b);
            let edges_a = geom_a.extract_segment_strings(InputIndex::A, None);
            let edges_b = geom_b.extract_segment_strings(InputIndex::B, None);

            let mutual = MutualSegmentSetIntersector::new(edges_a, None);
            let mut intersector = EdgeSegmentIntersector::new(&mut computer);
            mutual.process(&edges_b, &mut intersector);

            computer.evaluate_nodes();
            computer.finish();
        }
        let im = predicate.into_im();
        assert_eq!(
            im.get(CoordPos::Inside, CoordPos::Inside),
            Dimensions::ZeroDimensional
        );
    }

    #[test]
    fn self_noding_sweep_finds_the_same_crossing() {
        let a = wkt!(LINESTRING (0. 0., 2. 2.));
        let b = wkt!(LINESTRING (0. 2., 2. 0.));
        let cow_a = GeometryCow::from(&a);
        let cow_b = GeometryCow::from(&b);
        let geom_a = RelateGeometry::new(&cow_a);
        let geom_b = RelateGeometry::new(&cow_b);

        let mut predicate = RelateMatrixPredicate::new();
        {
            let mut computer = TopologyComputer::new(&mut predicate, &geom_a, &geom_b);
            let edges_a = geom_a.extract_segment_strings(InputIndex::A, None);
            let edges_b = geom_b.extract_segment_strings(InputIndex::B, None);

            let mut intersector = EdgeSegmentIntersector::new(&mut computer);
            intersect_all(&edges_a, &edges_b, None, &mut intersector);

            computer.evaluate_nodes();
            computer.finish();
        }
        let im = predicate.into_im();
        assert_eq!(
            im.get(CoordPos::Inside, CoordPos::Inside),
            Dimensions::ZeroDimensional
        );
    }
}
