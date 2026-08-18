//! Determines the location of a point which is known to lie on at least
//! one edge of a set of polygons.
//!
//! Port of JTS `AdjacentEdgeLocator`.
//!
//! This provides the union semantics for determining point location in a
//! GeometryCollection, which may have polygons with adjacent edges which
//! are effectively in the interior of the geometry. It is also possible to
//! have adjacent edges which lie on the boundary of the geometry (for
//! example a polygon contained within another polygon, with adjacent
//! edges).

use crate::coordinate_position::CoordPos;
use crate::dimensions::Dimensions;
use crate::winding_order::Winding;
use crate::{Coord, GeoFloat, Intersects, Line, LineString};

use super::node_section::NodeSection;
use super::node_sections::NodeSections;
use super::relate_point_locator::Polygonal;
use super::topology_predicate::InputIndex;

pub(crate) struct AdjacentEdgeLocator<F: GeoFloat> {
    /// Oriented copies of all polygon rings: shells CW, holes CCW.
    rings: Vec<Vec<Coord<F>>>,
}

impl<F: GeoFloat> AdjacentEdgeLocator<F> {
    pub fn new(polygonals: &[Polygonal<'_, F>]) -> Self {
        let mut rings = Vec::new();
        for polygonal in polygonals {
            for polygon in polygonal.polygons() {
                if polygon.exterior().0.is_empty() {
                    continue;
                }
                rings.push(oriented_ring_coords(polygon.exterior(), true));
                for hole in polygon.interiors() {
                    rings.push(oriented_ring_coords(hole, false));
                }
            }
        }
        Self { rings }
    }

    /// Locates a point known to lie on more than one polygon boundary:
    /// the incident ring edges are assembled into a node, and the point is
    /// interior unless some edge still has the exterior on a side.
    pub fn locate(&self, p: Coord<F>) -> CoordPos {
        let mut sections = NodeSections::new(p);
        for ring in &self.rings {
            add_sections(p, ring, &mut sections);
        }
        let node = sections.create_node();
        if node.has_exterior_edge(InputIndex::A) {
            CoordPos::OnBoundary
        } else {
            CoordPos::Inside
        }
    }
}

fn add_sections<F: GeoFloat>(p: Coord<F>, ring: &[Coord<F>], sections: &mut NodeSections<F>) {
    for i in 0..ring.len() - 1 {
        let p0 = ring[i];
        let p_next = ring[i + 1];

        if p == p_next {
            // The final point of a segment is assigned to the next
            // segment.
            continue;
        } else if p == p0 {
            let i_prev = if i > 0 { i - 1 } else { ring.len() - 2 };
            sections.add_node_section(create_section(p, ring[i_prev], p_next));
        } else if Line::new(p0, p_next).intersects(&p) {
            sections.add_node_section(create_section(p, p0, p_next));
        }
    }
}

fn create_section<F: GeoFloat>(p: Coord<F>, prev: Coord<F>, next: Coord<F>) -> NodeSection<F> {
    // The JTS leftover debug print for zero-length section segments is not
    // ported.
    NodeSection::new(
        InputIndex::A,
        Dimensions::TwoDimensional,
        1,
        0,
        None,
        false,
        Some(prev),
        p,
        Some(next),
    )
}

/// A copy of the ring's coordinates, oriented CW (for shells) or CCW (for
/// holes). Port of `RelateGeometry.orient`.
pub(crate) fn oriented_ring_coords<F: GeoFloat>(
    ring: &LineString<F>,
    require_cw: bool,
) -> Vec<Coord<F>> {
    let mut ring = ring.clone();
    if require_cw {
        ring.make_cw_winding();
    } else {
        ring.make_ccw_winding();
    }
    ring.0
}

#[cfg(test)]
mod tests {
    // Tests ported from JTS AdjacentEdgeLocatorTest.java (master, ab57bff).
    use super::super::relate_point_locator::GeometryElements;
    use super::*;
    use crate::geometry_cow::GeometryCow;
    use crate::wkt;

    fn check_location(geom: crate::GeometryCollection<f64>, x: f64, y: f64, expected: CoordPos) {
        let cow = GeometryCow::from(&geom);
        let elements = GeometryElements::extract(&cow);
        let locator = AdjacentEdgeLocator::new(elements.polygonals());
        assert_eq!(locator.locate(Coord { x, y }), expected);
    }

    #[test]
    fn test_adjacent_2() {
        check_location(
            wkt!(GEOMETRYCOLLECTION (
                POLYGON ((1. 9., 5. 9., 5. 1., 1. 1., 1. 9.)),
                POLYGON ((9. 9., 9. 1., 5. 1., 5. 9., 9. 9.))
            )),
            5.,
            5.,
            CoordPos::Inside,
        );
    }

    #[test]
    fn test_non_adjacent() {
        check_location(
            wkt!(GEOMETRYCOLLECTION (
                POLYGON ((1. 9., 4. 9., 5. 1., 1. 1., 1. 9.)),
                POLYGON ((9. 9., 9. 1., 5. 1., 5. 9., 9. 9.))
            )),
            5.,
            5.,
            CoordPos::OnBoundary,
        );
    }

    #[test]
    fn test_adjacent_6_with_filled_holes() {
        check_location(
            wkt!(GEOMETRYCOLLECTION (
                POLYGON ((1. 9., 5. 9., 6. 6., 1. 5., 1. 9.), (2. 6., 4. 8., 6. 6., 2. 6.)),
                POLYGON ((2. 6., 4. 8., 6. 6., 2. 6.)),
                POLYGON ((9. 9., 9. 5., 6. 6., 5. 9., 9. 9.)),
                POLYGON ((9. 1., 5. 1., 6. 6., 9. 5., 9. 1.), (7. 2., 6. 6., 8. 3., 7. 2.)),
                POLYGON ((7. 2., 6. 6., 8. 3., 7. 2.)),
                POLYGON ((1. 1., 1. 5., 6. 6., 5. 1., 1. 1.))
            )),
            6.,
            6.,
            CoordPos::Inside,
        );
    }

    #[test]
    fn test_adjacent_5_with_empty_hole() {
        check_location(
            wkt!(GEOMETRYCOLLECTION (
                POLYGON ((1. 9., 5. 9., 6. 6., 1. 5., 1. 9.), (2. 6., 4. 8., 6. 6., 2. 6.)),
                POLYGON ((2. 6., 4. 8., 6. 6., 2. 6.)),
                POLYGON ((9. 9., 9. 5., 6. 6., 5. 9., 9. 9.)),
                POLYGON ((9. 1., 5. 1., 6. 6., 9. 5., 9. 1.), (7. 2., 6. 6., 8. 3., 7. 2.)),
                POLYGON ((1. 1., 1. 5., 6. 6., 5. 1., 1. 1.))
            )),
            6.,
            6.,
            CoordPos::OnBoundary,
        );
    }

    #[test]
    fn test_contained_and_adjacent() {
        let wkt = wkt!(GEOMETRYCOLLECTION (
            POLYGON ((1. 9., 9. 9., 9. 1., 1. 1., 1. 9.)),
            POLYGON ((9. 2., 2. 2., 2. 8., 9. 8., 9. 2.))
        ));
        check_location(wkt.clone(), 9., 5., CoordPos::OnBoundary);
        check_location(wkt, 9., 8., CoordPos::OnBoundary);
    }

    /// Tests a bug caused by incorrect point-on-segment logic.
    #[test]
    fn test_disjoint_collinear() {
        check_location(
            wkt!(GEOMETRYCOLLECTION (
                MULTIPOLYGON (((1. 4., 4. 4., 4. 1., 1. 1., 1. 4.)), ((5. 4., 8. 4., 8. 1., 5. 1., 5. 4.)))
            )),
            2.,
            4.,
            CoordPos::OnBoundary,
        );
    }
}
