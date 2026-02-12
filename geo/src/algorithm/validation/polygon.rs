use super::{CoordIndex, RingRole, Validation, utils};
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::GeometryGraph;
use crate::{Coord, GeoFloat, HasDimensions, Polygon, PreparedGeometry, Relate};

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
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
/// Extracts touch-point information from the pre-computed GeometryGraph, whose
/// R-tree intersection detection runs in O((n + k) log n) rather than O(V^2).
///
/// The interior is disconnected when rings touch in a way that creates separate
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
/// Reference: OGC Simple Feature Specification (ISO 19125-1), section 6.1.11.1.
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
        for ei in edge.edge_intersections() {
            let coord = ei.coordinate();
            let is_vertex = edge.coords().contains(&coord);
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

    // Group all intersections sharing the same coordinate and build the
    // ring-pair touch map, skipping pairs the caller already reported.
    let mut ring_pair_touch_coords: HashMap<(usize, usize), Vec<Coord<F>>> = HashMap::new();

    let mut i = 0;
    while i < all_intersections.len() {
        let current_coord = all_intersections[i].coord;

        let mut j = i + 1;
        while j < all_intersections.len() && all_intersections[j].coord == current_coord {
            j += 1;
        }

        let group = &all_intersections[i..j];

        // Deduplicate edges within the group, merging vertex flags.
        let mut unique_edges: Vec<(usize, bool)> = Vec::new();
        for info in group {
            if let Some(last) = unique_edges.last_mut() {
                if last.0 == info.edge_idx {
                    last.1 = last.1 || info.is_vertex;
                    continue;
                }
            }
            unique_edges.push((info.edge_idx, info.is_vertex));
        }

        // A "touch" requires 2+ distinct edges, at least one having the point as a vertex.
        if unique_edges.len() >= 2 && unique_edges.iter().any(|(_, is_v)| *is_v) {
            for (idx_a, &(edge_a, _)) in unique_edges.iter().enumerate() {
                for &(edge_b, _) in unique_edges.iter().skip(idx_a + 1) {
                    let key = (edge_a, edge_b);
                    if !skip_pairs.contains(&key) {
                        ring_pair_touch_coords.entry(key).or_default().push(current_coord);
                    }
                }
            }
        }

        i = j;
    }

    check_ring_touches_disconnect_interior(&ring_pair_touch_coords, edges.len())
}

/// Check if ring touch points disconnect the interior.
///
/// Returns `Some((edge_a, edge_b))` if:
/// 1. Any two rings share 2+ distinct touch coordinates, OR
/// 2. Rings form a cycle in the touch graph through distinct coordinates
///    (each edge in the cycle connects via a different coordinate, so the
///    cycle encloses interior area).
fn check_ring_touches_disconnect_interior<F: GeoFloat>(
    ring_pair_touch_coords: &HashMap<(usize, usize), Vec<Coord<F>>>,
    num_rings: usize,
) -> Option<(usize, usize)> {
    // Case 1: any pair sharing 2+ distinct touch coordinates → disconnected
    for (&(edge_a, edge_b), coords) in ring_pair_touch_coords {
        if coords.len() >= 2 {
            return Some((edge_a, edge_b));
        }
    }

    // Case 2: cycle detection in the single-touch adjacency graph.
    let mut adjacency: Vec<HashSet<usize>> = vec![HashSet::new(); num_rings];
    let mut edge_coords: HashMap<(usize, usize), Coord<F>> = HashMap::new();

    for (&(i, j), coords) in ring_pair_touch_coords {
        if coords.len() == 1 {
            let coord = coords[0];
            adjacency[i].insert(j);
            adjacency[j].insert(i);
            edge_coords.insert((i, j), coord);
            edge_coords.insert((j, i), coord);
        }
    }

    // DFS to find a cycle through DISTINCT coordinates (a same-coordinate
    // cycle doesn't enclose area and is therefore harmless).
    let mut visited = vec![false; num_rings];

    for start in 0..num_rings {
        if visited[start] || adjacency[start].is_empty() {
            continue;
        }

        // (node, parent, coord used to enter node)
        let mut stack: Vec<(usize, Option<usize>, Option<Coord<F>>)> =
            vec![(start, None, None)];
        let mut node_entry_coord: HashMap<usize, Coord<F>> = HashMap::new();

        while let Some((node, parent, entry_coord)) = stack.pop() {
            if visited[node] {
                if let Some(entry) = entry_coord {
                    if let Some(&prev_entry) = node_entry_coord.get(&node) {
                        if entry != prev_entry {
                            // Cycle through distinct coordinates — find
                            // the responsible ring pair from the back-edge.
                            if let Some(p) = parent {
                                let (a, b) = if p < node { (p, node) } else { (node, p) };
                                return Some((a, b));
                            }
                        }
                    }
                }
                continue;
            }

            visited[node] = true;
            if let Some(coord) = entry_coord {
                node_entry_coord.insert(node, coord);
            }

            for &neighbor in &adjacency[node] {
                if Some(neighbor) == parent {
                    continue;
                }

                let edge_coord = edge_coords.get(&(node, neighbor)).copied();

                if visited[neighbor] {
                    if let Some(coord) = edge_coord {
                        if let Some(&neighbor_entry) = node_entry_coord.get(&neighbor) {
                            if coord != neighbor_entry {
                                let (a, b) = if node < neighbor {
                                    (node, neighbor)
                                } else {
                                    (neighbor, node)
                                };
                                return Some((a, b));
                            }
                        }
                    }
                } else {
                    stack.push((neighbor, Some(node), edge_coord));
                }
            }
        }
    }

    None
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
