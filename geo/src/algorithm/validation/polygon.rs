use super::{CoordIndex, RingRole, Validation, utils};
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::GeometryGraph;
use crate::{Coord, GeoFloat, HasDimensions, Polygon, PreparedGeometry, Relate};

use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;

/// A [`Polygon`] must follow these rules to be valid:
/// - [x] the polygon boundary rings (the exterior shell ring and interior hole rings) are simple (do not cross or self-touch). Because of this a polygon cannnot have cut lines, spikes or loops. This implies that polygon holes must be represented as interior rings, rather than by the exterior ring self-touching (a so-called "inverted hole").
/// - [x] boundary rings do not cross
/// - [x] boundary rings may touch at points but only as a tangent (i.e. not in a line)
/// - [x] interior rings are contained in the exterior ring
/// - [x] the polygon interior is simply connected (i.e. the rings must not touch in a way that splits the polygon into more than one part)
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidPolygon {
    /// A ring must have at least 4 points to be valid. Note that, in order to close the ring, the first and final points will be identical.
    TooFewPointsInRing(RingRole),
    /// A ring has a self-intersection.
    SelfIntersection(RingRole),
    /// One of the Polygon's coordinates is non-finite.
    NonFiniteCoord(RingRole, CoordIndex),
    /// A polygon's interiors must be completely within its exterior.
    InteriorRingNotContainedInExteriorRing(RingRole),
    /// A valid polygon's rings must not intersect one another. In this case, the intersection is 1-dimensional.
    IntersectingRingsOnALine(RingRole, RingRole),
    /// A valid polygon's rings must not intersect one another. In this case, the intersection is 2-dimensional.
    IntersectingRingsOnAnArea(RingRole, RingRole),
    /// The polygon interior is not simply connected because rings touch in a way that
    /// disconnects the interior into multiple regions.
    ///
    /// Per OGC Simple Feature Specification (ISO 19125-1), section 6.1.11.1:
    /// "The interior of every Surface is a connected point set."
    ///
    /// This can occur when:
    /// - Two rings share two or more vertices (creating a "corridor" that cuts through the interior)
    /// - Rings form a cycle of single-vertex touches that encloses part of the interior
    InteriorNotSimplyConnected(RingRole, RingRole),
}

impl fmt::Display for InvalidPolygon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidPolygon::TooFewPointsInRing(ring) => {
                write!(f, "{ring} must have at least 3 distinct points")
            }
            InvalidPolygon::SelfIntersection(ring) => {
                write!(f, "{ring} has a self-intersection")
            }
            InvalidPolygon::NonFiniteCoord(ring, idx) => {
                write!(f, "{ring} has a non-finite coordinate at index {}", idx.0)
            }
            InvalidPolygon::InteriorRingNotContainedInExteriorRing(ring) => {
                write!(f, "{ring} is not contained within the polygon's exterior")
            }
            InvalidPolygon::IntersectingRingsOnALine(ring_1, ring_2) => {
                write!(f, "{ring_1} and {ring_2} intersect on a line")
            }
            InvalidPolygon::IntersectingRingsOnAnArea(ring_1, ring_2) => {
                write!(f, "{ring_1} and {ring_2} intersect on an area")
            }
            InvalidPolygon::InteriorNotSimplyConnected(ring_1, ring_2) => {
                write!(
                    f,
                    "{ring_1} and {ring_2} touch in a way that disconnects the polygon interior"
                )
            }
        }
    }
}

impl std::error::Error for InvalidPolygon {}

impl<F: GeoFloat> Validation for Polygon<F> {
    type Error = InvalidPolygon;

    fn visit_validation<T>(
        &self,
        mut handle_validation_error: Box<dyn FnMut(Self::Error) -> Result<(), T> + '_>,
    ) -> Result<(), T> {
        if self.is_empty() {
            return Ok(());
        }

        for (ring_idx, ring) in std::iter::once(self.exterior())
            .chain(self.interiors().iter())
            .enumerate()
        {
            if ring.is_empty() {
                continue;
            }
            let ring_role = if ring_idx == 0 {
                RingRole::Exterior
            } else {
                RingRole::Interior(ring_idx - 1)
            };

            // Perform the various checks
            if utils::check_too_few_points(ring, true) {
                handle_validation_error(InvalidPolygon::TooFewPointsInRing(ring_role))?;
            }

            if utils::linestring_has_self_intersection(ring) {
                handle_validation_error(InvalidPolygon::SelfIntersection(ring_role))?;
            }

            for (coord_idx, coord) in ring.0.iter().enumerate() {
                if utils::check_coord_is_not_finite(coord) {
                    handle_validation_error(InvalidPolygon::NonFiniteCoord(
                        ring_role,
                        CoordIndex(coord_idx),
                    ))?;
                }
            }
        }

        // Skip interior checks if there are no non-empty interiors
        let has_interiors = self.interiors().iter().any(|ring| !ring.is_empty());
        if !has_interiors {
            return Ok(());
        }

        // Use PreparedGeometry for the exterior to cache its R-tree, avoiding
        // graph reconstruction for each interior containment check.
        let polygon_exterior = Polygon::new(self.exterior().clone(), vec![]);
        let prepared_exterior = PreparedGeometry::from(&polygon_exterior);

        // Track ring pairs that already have intersection errors so the
        // simply-connected check can skip them (avoids duplicate reporting).
        // Keys use GeometryGraph edge indices: 0 = exterior, N = interior N-1.
        let mut errored_edge_pairs: HashSet<(usize, usize)> = HashSet::new();

        for (interior_1_idx, interior_1) in self.interiors().iter().enumerate() {
            let ring_role_1 = RingRole::Interior(interior_1_idx);
            let edge_idx_1 = interior_1_idx + 1;
            if interior_1.is_empty() {
                continue;
            }

            let interior_1_as_poly = Polygon::new(interior_1.clone(), vec![]);
            let prepared_interior_1 = PreparedGeometry::from(&interior_1_as_poly);
            let exterior_vs_interior = prepared_exterior.relate(&prepared_interior_1);

            if !exterior_vs_interior.is_contains() {
                handle_validation_error(InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                    ring_role_1,
                ))?;
            }

            // Interior ring and exterior ring may only touch at point (not as a line)
            if exterior_vs_interior.get(CoordPos::OnBoundary, CoordPos::OnBoundary)
                == Dimensions::OneDimensional
            {
                errored_edge_pairs.insert((0, edge_idx_1));
                handle_validation_error(InvalidPolygon::IntersectingRingsOnALine(
                    RingRole::Exterior,
                    ring_role_1,
                ))?;
            }

            for (interior_2_idx, interior_2) in
                self.interiors().iter().enumerate().skip(interior_1_idx + 1)
            {
                let ring_role_2 = RingRole::Interior(interior_2_idx);
                let edge_idx_2 = interior_2_idx + 1;
                if interior_2.is_empty() {
                    continue;
                }

                let interior_2_as_poly = Polygon::new(interior_2.clone(), vec![]);
                let intersection_matrix = prepared_interior_1.relate(&interior_2_as_poly);

                if intersection_matrix.get(CoordPos::Inside, CoordPos::Inside)
                    == Dimensions::TwoDimensional
                {
                    errored_edge_pairs.insert((edge_idx_1, edge_idx_2));
                    handle_validation_error(InvalidPolygon::IntersectingRingsOnAnArea(
                        ring_role_1,
                        ring_role_2,
                    ))?;
                }
                if intersection_matrix.get(CoordPos::OnBoundary, CoordPos::OnBoundary)
                    == Dimensions::OneDimensional
                {
                    errored_edge_pairs.insert((edge_idx_1, edge_idx_2));
                    handle_validation_error(InvalidPolygon::IntersectingRingsOnALine(
                        ring_role_1,
                        ring_role_2,
                    ))?;
                }
            }
        }

        // Check that the interior is simply connected.
        let prepared_polygon = PreparedGeometry::from(self);
        if let Some((edge_a, edge_b)) = check_interior_simply_connected_from_graph(
            &prepared_polygon.geometry_graph,
            &errored_edge_pairs,
        ) {
            let role_a = edge_index_to_ring_role(edge_a);
            let role_b = edge_index_to_ring_role(edge_b);
            handle_validation_error(InvalidPolygon::InteriorNotSimplyConnected(role_a, role_b))?;
        }

        Ok(())
    }
}

/// Convert a GeometryGraph edge index to a [`RingRole`].
///
/// Edge 0 is the exterior ring; edge N (N >= 1) is interior ring N-1.
fn edge_index_to_ring_role(edge_idx: usize) -> RingRole {
    if edge_idx == 0 {
        RingRole::Exterior
    } else {
        RingRole::Interior(edge_idx - 1)
    }
}

/// Check that the polygon interior is simply connected using the GeometryGraph.
///
/// Extracts touch-point information from the pre-computed GeometryGraph. The
/// interior is disconnected when rings touch in a way that creates separate
/// regions. This occurs when:
/// - Two rings share 2+ touch points at different coordinates
/// - Rings form a cycle through distinct single touch points
///   (graph-theoretically: the ring adjacency graph has a cycle whose edges
///   correspond to distinct coordinates, meaning the cycle encloses area)
///
/// Multiple rings meeting at a *single* coordinate does NOT disconnect
/// the interior — the connected regions can still reach each other around
/// the shared point.
///
/// Ring pairs in `skip_pairs` are excluded (they already have intersection
/// errors reported by the caller).
///
/// Returns `None` if the interior is simply connected, or `Some((edge_a, edge_b))`
/// identifying the ring pair that causes disconnection.
fn check_interior_simply_connected_from_graph<F: GeoFloat>(
    graph: &GeometryGraph<F>,
    skip_pairs: &HashSet<(usize, usize)>,
) -> Option<(usize, usize)> {
    let edges = graph.edges();
    if edges.len() < 2 {
        return None;
    }

    // Collect all intersection points with their edge index and vertex status.
    struct IntersectionInfo<F: GeoFloat> {
        coord: Coord<F>,
        edge_idx: usize,
        is_vertex: bool,
    }

    let mut all_intersections: Vec<IntersectionInfo<F>> = Vec::new();
    for (edge_idx, edge) in edges.iter().enumerate() {
        let edge = RefCell::borrow(edge);
        let coords = edge.coords();
        for ei in edge.edge_intersections() {
            let coord = ei.coordinate();
            let is_vertex = coords.contains(&coord);
            all_intersections.push(IntersectionInfo {
                coord,
                edge_idx,
                is_vertex,
            });
        }
    }

    // Sort by (coordinate, edge_idx) using total_cmp for consistent ordering.
    all_intersections.sort_by(|a, b| {
        a.coord
            .x
            .total_cmp(&b.coord.x)
            .then_with(|| a.coord.y.total_cmp(&b.coord.y))
            .then_with(|| a.edge_idx.cmp(&b.edge_idx))
    });

    // Group all intersections sharing the same coordinate and build:
    // - coord_edges: (coord, edge_a, edge_b) for each touch, used by cycle detection
    // - ring_pair_seen: if a pair is seen a second time, it shares 2+ touch
    //   points at different coordinates, which always disconnects the interior
    let mut coord_edges: Vec<(Coord<F>, usize, usize)> = Vec::new();
    let mut ring_pair_seen: HashSet<(usize, usize)> = HashSet::new();
    let mut unique_edges: Vec<(usize, bool)> = Vec::new();
    let mut i = 0;
    while i < all_intersections.len() {
        let current_coord = all_intersections[i].coord;

        // Find the range of intersections sharing this coordinate.
        let mut j = i + 1;
        while j < all_intersections.len() && all_intersections[j].coord == current_coord {
            j += 1;
        }
        let group = &all_intersections[i..j];

        // Deduplicate edges within the group. The same edge can appear multiple
        // times with different vertex flags, so we merge them to ensure we only
        // count distinct edges and know if any of them is a vertex.
        unique_edges.clear();
        for info in group {
            if let Some(last) = unique_edges.last_mut() {
                if last.0 == info.edge_idx {
                    last.1 |= info.is_vertex;
                    continue;
                }
            }
            unique_edges.push((info.edge_idx, info.is_vertex));
        }

        // A "touch" requires 2+ distinct edges, at least one having the point
        // as a vertex. Skip pairs the caller already reported as intersecting.
        if unique_edges.len() >= 2 && unique_edges.iter().any(|(_, is_v)| *is_v) {
            for (idx_a, &(edge_a, _)) in unique_edges.iter().enumerate() {
                for &(edge_b, _) in unique_edges.iter().skip(idx_a + 1) {
                    let key = (edge_a, edge_b);
                    if !skip_pairs.contains(&key) {
                        if !ring_pair_seen.insert(key) {
                            return Some(key);
                        }
                        coord_edges.push((current_coord, edge_a, edge_b));
                    }
                }
            }
        }

        i = j;
    }

    check_ring_touches_disconnect_interior(&mut coord_edges, edges.len())
}

/// Detect cycles through distinct coordinates in the ring touch graph.
///
/// Each entry in `coord_edges` is `(coord, ring_a, ring_b)` representing a
/// single-touch between two rings at the given coordinate. The caller has
/// already handled the case where any pair shares 2+ touch points.
///
/// A cycle of ring touches only disconnects the interior if the cycle passes
/// through at least two distinct coordinates (enclosing area). A cycle where
/// every edge shares the same coordinate — e.g. three holes all meeting at
/// one point — does not enclose any area and is harmless.
///
/// To distinguish these cases, edges are sorted and grouped by coordinate,
/// then processed with two Union-Find structures:
///
/// - **Global UF**: accumulates connectivity across all coordinate groups
///   processed so far. "Are these two rings connected by ANY path of touches?"
/// - **Local UF**: reset for each coordinate group. "Are these two rings
///   connected through touches at THIS coordinate only?"
///
/// For each edge `(u, v)` in the current coordinate group:
/// - `!global.connected(u, v)`: first connection between these components — no
///   cycle at all. **Safe.**
/// - `global.connected(u, v) && local.connected(u, v)`: the rings are already
///   connected within this coordinate group — a same-coordinate cycle. **Safe.**
/// - `global.connected(u, v) && !local.connected(u, v)`: the rings are
///   reachable through a previously processed (different) coordinate, but not
///   through this one — closing a multi-coordinate cycle. **Invalid.**
fn check_ring_touches_disconnect_interior<F: GeoFloat>(
    coord_edges: &mut [(Coord<F>, usize, usize)],
    num_rings: usize,
) -> Option<(usize, usize)> {
    coord_edges.sort_by(|a, b| {
        a.0.x
            .total_cmp(&b.0.x)
            .then_with(|| a.0.y.total_cmp(&b.0.y))
    });

    let mut global = UnionFind::new(num_rings);
    let mut group_start = 0;
    while group_start < coord_edges.len() {
        let mut group_end = group_start + 1;
        while group_end < coord_edges.len()
            && coord_edges[group_end].0 == coord_edges[group_start].0
        {
            group_end += 1;
        }

        let mut local = UnionFind::new(num_rings);
        for &(_, u, v) in &coord_edges[group_start..group_end] {
            if global.find(u) == global.find(v) && local.find(u) != local.find(v) {
                return Some((u.min(v), u.max(v)));
            }
            global.union(u, v);
            local.union(u, v);
        }

        group_start = group_end;
    }

    None
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, x: usize, y: usize) {
        let rx = self.find(x);
        let ry = self.find(y);
        if rx == ry {
            return;
        }
        if self.rank[rx] < self.rank[ry] {
            self.parent[rx] = ry;
        } else if self.rank[rx] > self.rank[ry] {
            self.parent[ry] = rx;
        } else {
            self.parent[ry] = rx;
            self.rank[rx] += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::validation::{assert_valid, assert_validation_errors};
    use crate::wkt;

    #[test]
    fn test_polygon_valid() {
        // Unclosed rings are automatically closed by geo_types
        // so the following should be valid
        let polygon = wkt!(
            POLYGON((0. 0., 1. 1., 0. 1.))
        );
        assert_valid!(&polygon);
    }

    #[test]
    fn test_polygon_valid_interior_ring_touches_exterior_ring() {
        // The following polygon contains an interior ring that touches
        // the exterior ring on one point.
        // This is valid according to the OGC spec.
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 1., 4. 4.,0. 4.,0. 0.),
                (0. 2., 2. 1., 3. 2., 2. 3., 0. 2.)
            )
        );
        assert_valid!(&polygon);
    }

    #[test]
    fn test_polygon_valid_interior_rings_touch_at_point() {
        // The following polygon contains two interior rings that touch
        // at one point.
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4.,0. 4.,0. 0.),
                (1. 2., 2. 1., 3. 2., 2. 3., 1. 2.),
                (3. 2., 3.5 1., 3.75 2., 3.5 3., 3. 2.)
            )
        );
        assert_valid!(&polygon);
    }

    #[test]
    fn test_polygon_invalid_interior_rings_touch_at_line() {
        // The following polygon contains two interior rings that touch
        // on a line, this is not valid.
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4.,0. 4.,0. 0.),
                (1. 2., 2. 1., 3. 2., 2. 3., 1. 2.),
                (3. 2., 2. 1., 3.5 1., 3.75 2., 3.5 3., 3. 2.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::IntersectingRingsOnALine(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_rings_crosses() {
        // The following polygon contains two interior rings that cross
        // each other (they share some common area), this is not valid.
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0.,  4. 4.,   0. 4.,  0. 0.),
                (1. 2., 2. 1.,  3. 2.,   2. 3.,  1. 2.),
                (2. 2., 2. 1., 3.5 1., 3.75 2., 3.5 3., 3. 2.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::IntersectingRingsOnAnArea(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_ring_touches_exterior_ring_as_line() {
        // The following polygon contains an interior ring that touches
        // the exterior ring on one point.
        // This is valid according to the OGC spec.
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4., 0. 4., 0. 0.),
                // First two points are on the exterior ring
                (0. 2., 0. 1., 2. 1., 3. 2., 2. 3., 0. 2.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::IntersectingRingsOnALine(
                RingRole::Exterior,
                RingRole::Interior(0)
            )]
        );
    }

    #[test]
    fn test_polygon_invalid_too_few_point_exterior_ring() {
        // Unclosed rings are automatically closed by geo_types
        // but there is still two few points in this ring
        // to be a non-empty polygon
        let polygon = wkt!( POLYGON((0. 0., 1. 1.)) );
        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::TooFewPointsInRing(RingRole::Exterior)]
        );
    }

    #[test]
    fn test_polygon_invalid_spike() {
        // The following polygon contains a spike
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4., 2. 4., 2. 6., 2. 4., 0. 4., 0. 0.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    #[test]
    fn test_polygon_invalid_exterior_is_not_simple() {
        // The exterior ring of this polygon is not simple (i.e. it has a self-intersection)
        let polygon = wkt!(
            POLYGON((0. 0., 4. 0., 0. 2., 4. 2., 0. 0.))
        );
        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_not_fully_contained_in_exterior() {
        let polygon = wkt!(
            POLYGON (
                (0.5 0.5, 3.0 0.5, 3.0 2.5, 0.5 2.5, 0.5 0.5),
                (1.0 1.0, 1.0 2.0, 2.5 2.0, 3.5 1.0, 1.0 1.0)
            )
        );
        assert_validation_errors!(
            &polygon,
            vec![InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                RingRole::Interior(0)
            ),]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_ring_contained_in_interior_ring() {
        // The following polygon contains an interior ring that is contained
        // in another interior ring.
        let polygon_1 = wkt!(
            POLYGON(
                (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
                (1. 1.,  1. 9.,  9.  9., 9.  1., 1. 1.),
                (2. 2.,  2. 8.,  8.  8., 8.  2., 2. 2.)
            )
        );

        assert_validation_errors!(
            polygon_1,
            vec![InvalidPolygon::IntersectingRingsOnAnArea(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );

        // Let see if we switch the order of the interior rings
        // (this is still invalid)
        let polygon_2 = wkt!(
            POLYGON(
                (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
                (2. 2.,  2. 8.,  8.  8., 8.  2., 2. 2.),
                (1. 1.,  1. 9.,  9.  9., 9.  1., 1. 1.)
            )
        );

        assert_validation_errors!(
            polygon_2,
            vec![InvalidPolygon::IntersectingRingsOnAnArea(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );
    }
}
