use super::{CoordIndex, RingRole, Validation, utils};
use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::relate::geomgraph::GeometryGraph;
use crate::{GeoFloat, HasDimensions, Polygon, PreparedGeometry, Relate};

use total_ord_coord::TotalOrdCoord;
use union_find::UnionFind;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
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

        for (interior_1_idx, interior_1) in self.interiors().iter().enumerate() {
            let ring_role_1 = RingRole::Interior(interior_1_idx);
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
        let geometry_graph = GeometryGraph::new(0, self.into());
        if let Some((edge_a, edge_b)) = check_interior_simply_connected_from_graph(&geometry_graph)
        {
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
) -> Option<(usize, usize)> {
    let rings = graph.edges();
    if rings.len() < 2 {
        return None;
    }

    // Group intersections by coordinate
    //     Coord => (
    //        0: All the rings which intersect at this coordinate
    //        1: If at least one of these intersecting rings has this coordinate as a vertex
    //     )
    let mut all_intersections: BTreeMap<TotalOrdCoord<F>, (BTreeSet<usize>, bool)> =
        BTreeMap::new();
    for (ring_idx, ring_edge) in rings.iter().enumerate() {
        let ring_edge = RefCell::borrow(ring_edge);
        let coords = ring_edge.coords();
        for ei in ring_edge.edge_intersections() {
            let coord = ei.coordinate();
            let is_vertex = coords.contains(&coord);
            let (intersecting_rings, any_is_vertex) = all_intersections
                .entry(TotalOrdCoord(coord))
                .or_insert((BTreeSet::new(), false));
            intersecting_rings.insert(ring_idx);
            *any_is_vertex |= is_vertex;
        }
    }

    // Which rings are connected, even if vicariously, through other rings.
    let mut global = UnionFind::new(rings.len());
    for (_intersection_coord, (intersecting_rings, intersection_is_on_a_vertex)) in
        all_intersections
    {
        if !intersection_is_on_a_vertex {
            continue;
        }
        // Which rings are connected at *this* coordinate
        // let mut local = UnionFind::new(rings.len());

        let mut intersecting_rings = intersecting_rings.into_iter();
        let Some(first) = intersecting_rings.next() else {
            continue;
        };
        for next in intersecting_rings {
            if global.find_root(first) == global.find_root(next)
            // && local.find_root(first) != local.find_root(next)
            {
                // These rings have touched before, we have a disconnected interior.
                return Some((first.min(next), first.max(next)));
            } else {
                global.union_sets(first, next);
                // local.union_sets(first, next);
            }
        }
    }
    None
}

mod total_ord_coord {
    use crate::{Coord, GeoFloat};
    use std::cmp::Ordering;

    #[derive(Debug, Clone, Copy)]
    pub(super) struct TotalOrdCoord<F: GeoFloat>(pub Coord<F>);

    impl<F: GeoFloat> PartialEq<Self> for TotalOrdCoord<F> {
        fn eq(&self, other: &Self) -> bool {
            self.cmp(other) == Ordering::Equal
        }
    }
    impl<F: GeoFloat> Eq for TotalOrdCoord<F> {}
    impl<F: GeoFloat> PartialOrd for TotalOrdCoord<F> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl<F: GeoFloat> Ord for TotalOrdCoord<F> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.0
                .x
                .total_cmp(&other.0.x)
                .then_with(|| self.0.y.total_cmp(&other.0.y))
        }
    }
}

mod union_find {
    use std::cmp::Ordering;

    /// Union Find is a classic algorithm for managing disjoint sets - i.e. which rings are touching.
    ///
    /// Includes Path Compression and Rank counting optimizations.
    pub(super) struct UnionFind {
        parent: Vec<usize>,
        rank: Vec<usize>,
    }

    impl UnionFind {
        pub(crate) fn new(n: usize) -> Self {
            Self {
                parent: (0..n).collect(),
                rank: vec![0; n],
            }
        }

        /// Which set does `x` belong to? (identified by the set parent).
        ///
        /// Rings that are touching will have the same root.
        /// Once all touching rings have been unioned, rings with different roots are not touching.
        pub(crate) fn find_root(&mut self, x: usize) -> usize {
            let mut parent = x;
            // A root node is its own parent
            while parent != self.parent[x] {
                parent = self.parent[self.parent[x]];
                // compress path to make `find_root` faster next time
                self.parent[x] = parent;
            }
            parent
        }

        /// If two rings are touching, union their two "touchable" sets.
        pub(crate) fn union_sets(&mut self, x: usize, y: usize) {
            let root_x = self.find_root(x);
            let root_y = self.find_root(y);
            if root_x == root_y {
                // Already in same set
                return;
            }
            match self.rank[root_x].cmp(&self.rank[root_y]) {
                Ordering::Less => {
                    self.parent[root_x] = root_y;
                }
                Ordering::Greater => {
                    self.parent[root_y] = root_x;
                }
                Ordering::Equal => {
                    self.parent[root_y] = root_x;
                    self.rank[root_x] += 1;
                }
            }
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
            vec![
                InvalidPolygon::IntersectingRingsOnALine(
                    RingRole::Interior(0),
                    RingRole::Interior(1)
                ),
                InvalidPolygon::InteriorNotSimplyConnected(
                    RingRole::Interior(0),
                    RingRole::Interior(1)
                )
            ]
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
            vec![
                InvalidPolygon::IntersectingRingsOnAnArea(
                    RingRole::Interior(0),
                    RingRole::Interior(1)
                ),
                InvalidPolygon::InteriorNotSimplyConnected(
                    RingRole::Interior(0),
                    RingRole::Interior(1)
                )
            ]
        );
    }

    #[test]
    fn test_polygon_invalid_interior_ring_touches_exterior_ring_as_line() {
        // The following polygon contains an interior ring that overlaps
        // the exterior ring along a line.
        let polygon = wkt!(
            POLYGON(
                (0. 0., 4. 0., 4. 4., 0. 4., 0. 0.),
                // First two points are on the exterior ring
                (0. 2., 0. 1., 2. 1., 3. 2., 2. 3., 0. 2.)
            )
        );

        assert_validation_errors!(
            &polygon,
            vec![
                InvalidPolygon::IntersectingRingsOnALine(RingRole::Exterior, RingRole::Interior(0)),
                InvalidPolygon::InteriorNotSimplyConnected(
                    RingRole::Exterior,
                    RingRole::Interior(0)
                )
            ]
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
