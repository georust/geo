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
    /// This can occur when:
    /// - Two rings share two or more vertices (creating a "corridor" that cuts through the interior)
    /// - Rings form a cycle of single-vertex touches that encloses part of the interior
    ///
    /// The coordinates indicate the problematic touch point(s) where the disconnection occurs.
    InteriorNotSimplyConnected(Vec<Coord<f64>>),
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
            InvalidPolygon::InteriorNotSimplyConnected(coords) => {
                write!(
                    f,
                    "polygon interior is not simply connected at coordinate(s): {}",
                    coords
                        .iter()
                        .map(|c| format!("({}, {})", c.x, c.y))
                        .collect::<Vec<_>>()
                        .join(", ")
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

        // Use PreparedGeometry for the exterior to cache its graph and R-tree.
        // This avoids rebuilding the graph for each interior containment check.
        let polygon_exterior = Polygon::new(self.exterior().clone(), vec![]);
        let prepared_exterior = PreparedGeometry::from(&polygon_exterior);

        for (interior_1_idx, interior_1) in self.interiors().iter().enumerate() {
            let ring_role_1 = RingRole::Interior(interior_1_idx);
            if interior_1.is_empty() {
                continue;
            }

            // Build polygon from interior ring - used for both exterior check and
            // interior-interior comparisons
            let interior_1_as_poly = Polygon::new(interior_1.clone(), vec![]);
            let prepared_interior_1 = PreparedGeometry::from(&interior_1_as_poly);

            // Check containment: interior ring must be properly inside exterior.
            // We use polygon-to-polygon comparison and check both:
            // - is_contains(): exterior must contain interior
            // - is_equal_topo(): they must NOT be identical (an interior ring equal to
            //   exterior is invalid)
            let exterior_vs_interior = prepared_exterior.relate(&interior_1_as_poly);

            if !exterior_vs_interior.is_contains() || exterior_vs_interior.is_equal_topo() {
                handle_validation_error(InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                    ring_role_1,
                ))?;
            }

            // Interior ring and exterior ring may only touch at point (not as a line)
            // For polygon-to-polygon, check boundary-boundary overlap
            if exterior_vs_interior.get(CoordPos::OnBoundary, CoordPos::OnBoundary)
                == Dimensions::OneDimensional
            {
                handle_validation_error(InvalidPolygon::IntersectingRingsOnALine(
                    RingRole::Exterior,
                    ring_role_1,
                ))?;
            }

            for (interior_2_idx, interior_2) in
                self.interiors().iter().enumerate().skip(interior_1_idx + 1)
            {
                let ring_role_2 = RingRole::Interior(interior_2_idx);
                if interior_2.is_empty() {
                    continue;
                }

                // Compare as polygons to detect 2D overlap.
                let interior_2_as_poly = Polygon::new(interior_2.clone(), vec![]);
                let intersection_matrix = prepared_interior_1.relate(&interior_2_as_poly);

                if intersection_matrix.get(CoordPos::Inside, CoordPos::Inside)
                    == Dimensions::TwoDimensional
                {
                    handle_validation_error(InvalidPolygon::IntersectingRingsOnAnArea(
                        ring_role_1,
                        ring_role_2,
                    ))?;
                }
                if intersection_matrix.get(CoordPos::OnBoundary, CoordPos::OnBoundary)
                    == Dimensions::OneDimensional
                {
                    handle_validation_error(InvalidPolygon::IntersectingRingsOnALine(
                        ring_role_1,
                        ring_role_2,
                    ))?;
                }
            }
        }

        // Check that the interior is simply connected.
        // Build PreparedGeometry for the whole polygon here (deferred from earlier)
        // so we don't pay the cost if validation already failed above.
        let prepared_polygon = PreparedGeometry::from(self);
        if let Some(problem_coords) =
            check_interior_simply_connected_from_graph(&prepared_polygon.geometry_graph)
        {
            handle_validation_error(InvalidPolygon::InteriorNotSimplyConnected(problem_coords))?;
        }

        Ok(())
    }
}

/// Check that the polygon interior is simply connected using the GeometryGraph.
///
/// This function extracts touch point information from a pre-computed GeometryGraph,
/// which is more efficient than brute-force comparison because the graph's R-tree
/// intersection detection runs in O((n + k) log n) instead of O(V²).
///
/// The interior is disconnected if rings touch in a way that creates separate
/// interior regions. This occurs when:
/// - Two rings share 2+ touch points at different coordinates
/// - Rings form a cycle through distinct single touch points
///
/// Note: Multiple rings meeting at a single coordinate does NOT disconnect
/// the interior. For example, three triangular holes meeting at one point
/// can still have a connected interior around their perimeter.
///
/// Returns `None` if the interior is simply connected, or `Some(coords)` with
/// the problematic touch coordinates if the interior is disconnected.
fn check_interior_simply_connected_from_graph<F: GeoFloat>(
    graph: &GeometryGraph<F>,
) -> Option<Vec<Coord<f64>>> {
    let edges = graph.edges();

    // Need at least 2 edges (rings) to have a touch point
    // Edge 0 = exterior, Edge 1+ = interiors
    if edges.len() < 2 {
        return None;
    }

    // Collect all intersection points with their edge index and whether they're a vertex.
    // A "touch" requires at least one ring to have the point as a vertex.
    struct IntersectionInfo<F: GeoFloat> {
        coord: Coord<F>,
        edge_idx: usize,
        is_vertex: bool,
    }

    let mut all_intersections: Vec<IntersectionInfo<F>> = Vec::new();

    for (edge_idx, edge) in edges.iter().enumerate() {
        let edge = RefCell::borrow(edge);

        // Collect edge intersection coordinates
        for ei in edge.edge_intersections() {
            let coord = ei.coordinate();
            // Check if this intersection point is also a vertex of this edge
            let is_vertex = edge.coords().iter().any(|v| *v == coord);
            all_intersections.push(IntersectionInfo {
                coord,
                edge_idx,
                is_vertex,
            });
        }
    }

    // Sort by (coordinate, edge_idx) using total_cmp for consistent ordering.
    // Sorting by edge_idx within each coordinate group enables efficient deduplication.
    all_intersections.sort_by(|a, b| {
        a.coord
            .x
            .total_cmp(&b.coord.x)
            .then_with(|| a.coord.y.total_cmp(&b.coord.y))
            .then_with(|| a.edge_idx.cmp(&b.edge_idx))
    });

    // Group consecutive coordinates that are equal and build the ring-pair touch map
    let mut ring_pair_touch_coords: HashMap<(usize, usize), Vec<Coord<F>>> = HashMap::new();

    let mut i = 0;
    while i < all_intersections.len() {
        let current_coord = all_intersections[i].coord;

        // Find all entries with the same coordinate
        let mut j = i + 1;
        while j < all_intersections.len() && all_intersections[j].coord == current_coord {
            j += 1;
        }

        // Group spans from i to j (exclusive), sorted by edge_idx within the group
        let group = &all_intersections[i..j];

        // Collect unique edges and track if any is a vertex
        // Since sorted by edge_idx, we can detect duplicates by comparing adjacent entries
        let mut unique_edges: Vec<(usize, bool)> = Vec::new();
        for info in group {
            if let Some(last) = unique_edges.last_mut() {
                if last.0 == info.edge_idx {
                    // Same edge - merge is_vertex flags
                    last.1 = last.1 || info.is_vertex;
                    continue;
                }
            }
            unique_edges.push((info.edge_idx, info.is_vertex));
        }

        // Need at least 2 distinct edges meeting at this point
        if unique_edges.len() >= 2 {
            // Check if at least one edge has this as a vertex
            let has_vertex = unique_edges.iter().any(|(_, is_v)| *is_v);

            if has_vertex {
                // Record this touch point for each pair of distinct edges
                for (idx_a, &(edge_a, _)) in unique_edges.iter().enumerate() {
                    for &(edge_b, _) in unique_edges.iter().skip(idx_a + 1) {
                        let key = (edge_a, edge_b); // Already sorted since edges are sorted
                        ring_pair_touch_coords.entry(key).or_default().push(current_coord);
                    }
                }
            }
        }

        i = j;
    }

    // Check for disconnection using the touch point data
    check_ring_touches_disconnect_interior(&ring_pair_touch_coords, edges.len())
}

/// Check if ring touch points disconnect the interior.
///
/// The interior is disconnected if:
/// 1. Any two rings share 2+ distinct touch coordinates
/// 2. Rings form a cycle in the touch graph through distinct coordinates
fn check_ring_touches_disconnect_interior<F: GeoFloat>(
    ring_pair_touch_coords: &HashMap<(usize, usize), Vec<Coord<F>>>,
    num_rings: usize,
) -> Option<Vec<Coord<f64>>> {
    // Check 1: Any two rings sharing 2+ distinct touch coordinates → disconnected
    for coords in ring_pair_touch_coords.values() {
        if coords.len() >= 2 {
            let problem_coords: Vec<Coord<f64>> = coords
                .iter()
                .map(|c| Coord {
                    x: c.x.to_f64().unwrap_or(f64::NAN),
                    y: c.y.to_f64().unwrap_or(f64::NAN),
                })
                .collect();
            return Some(problem_coords);
        }
    }

    // Check 2: Cycle detection in the touch graph
    // Build adjacency list for rings connected by exactly one touch point
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

    // DFS to detect cycles through DISTINCT coordinates
    let mut visited = vec![false; num_rings];

    for start in 0..num_rings {
        if visited[start] || adjacency[start].is_empty() {
            continue;
        }

        let mut stack = vec![(start, None::<usize>, None::<Coord<F>>)];
        let mut node_entry_coord: HashMap<usize, Coord<F>> = HashMap::new();

        while let Some((node, parent, entry_coord)) = stack.pop() {
            if visited[node] {
                if let Some(entry) = entry_coord {
                    if let Some(&prev_entry) = node_entry_coord.get(&node) {
                        // Check if the coordinates are different (using ==)
                        if entry != prev_entry {
                            return Some(vec![
                                Coord {
                                    x: entry.x.to_f64().unwrap_or(f64::NAN),
                                    y: entry.y.to_f64().unwrap_or(f64::NAN),
                                },
                                Coord {
                                    x: prev_entry.x.to_f64().unwrap_or(f64::NAN),
                                    y: prev_entry.y.to_f64().unwrap_or(f64::NAN),
                                },
                            ]);
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
                                return Some(vec![
                                    Coord {
                                        x: coord.x.to_f64().unwrap_or(f64::NAN),
                                        y: coord.y.to_f64().unwrap_or(f64::NAN),
                                    },
                                    Coord {
                                        x: neighbor_entry.x.to_f64().unwrap_or(f64::NAN),
                                        y: neighbor_entry.y.to_f64().unwrap_or(f64::NAN),
                                    },
                                ]);
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

        // When rings share a line segment, they share multiple vertices,
        // which also disconnects the interior
        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0],
            InvalidPolygon::IntersectingRingsOnALine(RingRole::Interior(0), RingRole::Interior(1))
        );
        // The shared vertices are (2,1) and (3,2) - check that we get InteriorNotSimplyConnected
        assert!(matches!(
            &errors[1],
            InvalidPolygon::InteriorNotSimplyConnected(coords) if coords.len() >= 2
        ));
    }

    #[test]
    fn test_polygon_invalid_interior_rings_crosses() {
        // The following polygon contains two interior rings that cross
        // each other (they share some common area), this is not valid.
        // They also share vertices at (2. 1.) and (3. 2.), which disconnects the interior.
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0.,  4. 4.,   0. 4.,  0. 0.),
                (1. 2., 2. 1.,  3. 2.,   2. 3.,  1. 2.),
                (2. 2., 2. 1., 3.5 1., 3.75 2., 3.5 3., 3. 2.)
            )
        );

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0],
            InvalidPolygon::IntersectingRingsOnAnArea(RingRole::Interior(0), RingRole::Interior(1))
        );
        // The shared vertices are (2,1) and (3,2) - check that we get InteriorNotSimplyConnected
        assert!(matches!(
            &errors[1],
            InvalidPolygon::InteriorNotSimplyConnected(coords) if coords.len() >= 2
        ));
    }

    #[test]
    fn test_polygon_invalid_interior_ring_touches_exterior_ring_as_line() {
        // The following polygon contains an interior ring that shares an
        // edge with the exterior ring (two adjacent vertices on the edge).
        // This is invalid because:
        // 1. Rings intersect on a line (the shared edge)
        // 2. The shared edge also disconnects the interior
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4., 0. 4., 0. 0.),
                // First two points are on the exterior ring's edge at x=0
                (0. 2., 0. 1., 2. 1., 3. 2., 2. 3., 0. 2.)
            )
        );

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 2);
        assert_eq!(
            errors[0],
            InvalidPolygon::IntersectingRingsOnALine(RingRole::Exterior, RingRole::Interior(0))
        );
        // The shared vertices are (0,1) and (0,2) - check that we get InteriorNotSimplyConnected
        assert!(matches!(
            &errors[1],
            InvalidPolygon::InteriorNotSimplyConnected(coords) if coords.len() >= 2
        ));
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
