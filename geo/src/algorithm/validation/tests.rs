#[test]
fn jts_validation_tests() {
    jts_test_runner::assert_jts_tests_succeed("*Valid*");
}

mod simply_connected_interior {
    //! Tests for simply connected interior validation.
    //!
    //! OGC Simple Feature Specification (ISO 19125-1), section 6.1.11.1 states:
    //! "The interior of every Surface is a connected point set."
    //!
    //! These tests verify that we correctly detect polygons with disconnected
    //! interiors, which can occur when:
    //! - Two holes share 2+ vertices (creating a "corridor")
    //! - Rings form a cycle of single-vertex touches that encloses part of the interior

    use crate::algorithm::validation::polygon::InvalidPolygon;
    use crate::algorithm::validation::{RingRole, Validation};
    use crate::coord;
    use crate::geometry::{LineString, Polygon};
    use geo_test_fixtures::checkerboard::{box_ring, create_checkerboard};

    /// Two L-shaped holes sharing vertices at (2,2) and (3,3).
    ///
    /// Simplest case: two holes share exactly 2 vertices, creating a
    /// "corridor" that cuts through the interior.
    #[test]
    fn two_holes_sharing_two_vertices() {
        let exterior = box_ring(0.0, 0.0, 5.0, 5.0);

        let top_l = LineString::new(vec![
            coord! { x: 1.0, y: 2.0 },
            coord! { x: 2.0, y: 2.0 },
            coord! { x: 2.0, y: 3.0 },
            coord! { x: 3.0, y: 3.0 },
            coord! { x: 3.0, y: 4.0 },
            coord! { x: 1.0, y: 4.0 },
            coord! { x: 1.0, y: 2.0 },
        ]);

        let bottom_l = LineString::new(vec![
            coord! { x: 2.0, y: 1.0 },
            coord! { x: 4.0, y: 1.0 },
            coord! { x: 4.0, y: 3.0 },
            coord! { x: 3.0, y: 3.0 },
            coord! { x: 3.0, y: 2.0 },
            coord! { x: 2.0, y: 2.0 },
            coord! { x: 2.0, y: 1.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![top_l, bottom_l]);

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0],
            InvalidPolygon::InteriorNotSimplyConnected(
                RingRole::Interior(0),
                RingRole::Interior(1),
            ),
        );
    }

    /// Checkerboard level 0: 13 holes in a checkerboard pattern where
    /// adjacent holes share single vertices at grid intersections.
    #[test]
    fn checkerboard_level_0() {
        let (polygon, expected_area) = create_checkerboard(0);

        assert_eq!(polygon.interiors().len(), 13);

        use crate::algorithm::Area;
        let actual_area = polygon.unsigned_area();
        assert!(
            (actual_area - expected_area).abs() < 1e-10,
            "Area mismatch: expected {expected_area}, got {actual_area}",
        );

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(errors[0], InvalidPolygon::InteriorNotSimplyConnected(_, _)),
            "Expected InteriorNotSimplyConnected, got: {:?}",
            errors[0],
        );
    }

    /// Checkerboard level 1: nested checkerboard with level-0 inside one
    /// of the "filled" squares.
    #[test]
    fn checkerboard_level_1() {
        let (polygon, expected_area) = create_checkerboard(1);

        assert_eq!(polygon.interiors().len(), 26);

        use crate::algorithm::Area;
        let actual_area = polygon.unsigned_area();
        assert!(
            (actual_area - expected_area).abs() < 1e-10,
            "Area mismatch: expected {expected_area}, got {actual_area}",
        );

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(errors[0], InvalidPolygon::InteriorNotSimplyConnected(_, _)),
            "Expected InteriorNotSimplyConnected, got: {:?}",
            errors[0],
        );
    }

    /// Two holes sharing exactly ONE vertex is valid (interior remains connected).
    #[test]
    fn holes_sharing_one_vertex_is_valid() {
        let exterior = box_ring(0.0, 0.0, 6.0, 6.0);
        let hole1 = box_ring(1.0, 1.0, 3.0, 3.0);
        let hole2 = box_ring(3.0, 3.0, 5.0, 5.0);

        let polygon = Polygon::new(exterior, vec![hole1, hole2]);
        assert!(
            polygon.is_valid(),
            "Polygon with two holes sharing 1 vertex should be valid",
        );
    }

    /// Non-touching holes: always valid.
    #[test]
    fn separate_holes_is_valid() {
        let exterior = box_ring(0.0, 0.0, 10.0, 10.0);
        let hole1 = box_ring(1.0, 1.0, 3.0, 3.0);
        let hole2 = box_ring(5.0, 5.0, 7.0, 7.0);
        let hole3 = box_ring(1.0, 6.0, 3.0, 8.0);

        let polygon = Polygon::new(exterior, vec![hole1, hole2, hole3]);
        assert!(
            polygon.is_valid(),
            "Polygon with separate non-touching holes should be valid",
        );
    }

    /// Three triangular holes meeting at a single vertex: valid.
    ///
    /// When three holes share a single vertex but don't share any edges,
    /// the interior is still connected around the perimeter.
    #[test]
    fn three_holes_meeting_at_vertex_valid() {
        let exterior = box_ring(-1.1, -1.1, 1.1, 1.1);

        let hole_a = LineString::new(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.5, y: 0.8660254037844386 },
            coord! { x: 1.0, y: 0.0 },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let hole_b = LineString::new(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: -1.0, y: 0.0 },
            coord! { x: -0.5, y: 0.8660254037844386 },
            coord! { x: 0.0, y: 0.0 },
        ]);
        let hole_c = LineString::new(vec![
            coord! { x: 0.0, y: 0.0 },
            coord! { x: 0.5, y: -0.8660254037844386 },
            coord! { x: -0.5, y: -0.8660254037844386 },
            coord! { x: 0.0, y: 0.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![hole_a, hole_b, hole_c]);
        assert!(
            polygon.is_valid(),
            "Polygon with 3 holes meeting at one vertex (wedge pattern) should be valid",
        );
    }

    /// Four triangular holes forming a cycle through single touches.
    ///
    /// Each pair shares exactly one vertex at a different coordinate,
    /// forming cycle A-B-C-D-A. This encloses the center region.
    #[test]
    fn four_holes_forming_cycle() {
        let exterior = box_ring(-10.0, -10.0, 10.0, 10.0);

        // A: bottom, shares (-8,-8) with B, (8,-8) with D
        let hole_a = LineString::new(vec![
            coord! { x: -8.0, y: -8.0 },
            coord! { x: 0.0, y: -4.0 },
            coord! { x: 8.0, y: -8.0 },
            coord! { x: -8.0, y: -8.0 },
        ]);
        // B: left, shares (-8,8) with C, (-8,-8) with A
        let hole_b = LineString::new(vec![
            coord! { x: -8.0, y: 8.0 },
            coord! { x: -4.0, y: 0.0 },
            coord! { x: -8.0, y: -8.0 },
            coord! { x: -8.0, y: 8.0 },
        ]);
        // C: top, shares (8,8) with D, (-8,8) with B
        let hole_c = LineString::new(vec![
            coord! { x: 8.0, y: 8.0 },
            coord! { x: 0.0, y: 4.0 },
            coord! { x: -8.0, y: 8.0 },
            coord! { x: 8.0, y: 8.0 },
        ]);
        // D: right, shares (8,-8) with A, (8,8) with C
        let hole_d = LineString::new(vec![
            coord! { x: 8.0, y: -8.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 8.0, y: 8.0 },
            coord! { x: 8.0, y: -8.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![hole_a, hole_b, hole_c, hole_d]);

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(errors[0], InvalidPolygon::InteriorNotSimplyConnected(_, _)),
            "Expected InteriorNotSimplyConnected, got: {:?}",
            errors[0],
        );
    }

    /// Three triangular holes forming a 3-node cycle through distinct touch points.
    ///
    /// Each pair shares exactly one vertex at a different coordinate:
    /// - H0-H1 share (10, 2)
    /// - H1-H2 share (16, 10)
    /// - H2-H0 share (4, 10)
    ///
    /// This forms a cycle H0-H1-H2-H0 through three distinct coordinates,
    /// enclosing the central triangle and disconnecting the interior.
    #[test]
    fn three_holes_forming_triangle_cycle() {
        let exterior = box_ring(0.0, 0.0, 20.0, 20.0);

        // H0: bottom-left, shares (4,10) with H2 and (10,2) with H1
        let hole_0 = LineString::new(vec![
            coord! { x: 4.0, y: 10.0 },
            coord! { x: 2.0, y: 2.0 },
            coord! { x: 10.0, y: 2.0 },
            coord! { x: 4.0, y: 10.0 },
        ]);
        // H1: bottom-right, shares (10,2) with H0 and (16,10) with H2
        let hole_1 = LineString::new(vec![
            coord! { x: 10.0, y: 2.0 },
            coord! { x: 18.0, y: 2.0 },
            coord! { x: 16.0, y: 10.0 },
            coord! { x: 10.0, y: 2.0 },
        ]);
        // H2: top, shares (16,10) with H1 and (4,10) with H0
        let hole_2 = LineString::new(vec![
            coord! { x: 16.0, y: 10.0 },
            coord! { x: 10.0, y: 18.0 },
            coord! { x: 4.0, y: 10.0 },
            coord! { x: 16.0, y: 10.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![hole_0, hole_1, hole_2]);

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(errors[0], InvalidPolygon::InteriorNotSimplyConnected(_, _)),
            "Expected InteriorNotSimplyConnected, got: {:?}",
            errors[0],
        );
    }

    /// Two holes forming a chain with vertex-on-edge touches to exterior.
    ///
    /// H0 touches exterior at (0,5), H1 touches exterior at (20,5),
    /// both share (10,5). This creates a chain through the exterior with
    /// distinct coordinates, disconnecting the interior.
    #[test]
    fn hole_chain_with_vertex_on_edge_touch() {
        let exterior = box_ring(0.0, 0.0, 20.0, 15.0);

        let hole_0 = LineString::new(vec![
            coord! { x: 0.0, y: 5.0 },
            coord! { x: 10.0, y: 5.0 },
            coord! { x: 5.0, y: 10.0 },
            coord! { x: 0.0, y: 5.0 },
        ]);
        let hole_1 = LineString::new(vec![
            coord! { x: 10.0, y: 5.0 },
            coord! { x: 20.0, y: 5.0 },
            coord! { x: 15.0, y: 10.0 },
            coord! { x: 10.0, y: 5.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![hole_0, hole_1]);

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(errors[0], InvalidPolygon::InteriorNotSimplyConnected(_, _)),
            "Expected InteriorNotSimplyConnected, got: {:?}",
            errors[0],
        );
    }
}
/// Test cases ported from GDAL's geometry validity documentation.
/// See: https://gdal.org/en/latest/user/geometry_validity.html
///
/// In some cases, our validation output diverges from GDAL's, but (so far!) our own behavior
/// seems defensible. These divergences are noted in the documentation of individual tests.
mod gdal_test_cases {
    use crate::algorithm::validation::{
        GeometryIndex, InvalidMultiPolygon, InvalidPolygon, RingRole, Validation,
    };
    use crate::wkt;

    // GDAL heading: "Self-intersecting polygon"
    // GDAL error: "Self-intersection"
    #[test]
    fn self_intersecting_polygon() {
        let polygon = wkt!(POLYGON ((10. 90., 90. 10., 90. 90., 10. 10., 10. 90.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    // GDAL heading: "Polygon with self-touching ring"
    // GDAL error: "Ring Self-intersection"
    // Our code uses the same SelfIntersection variant for both full self-intersections and
    // self-touching (degenerate) rings.
    #[test]
    fn polygon_with_self_touching_ring() {
        let polygon = wkt!(POLYGON ((10. 10., 90. 10., 90. 40., 80. 20., 70. 40., 80. 60., 90. 40., 90. 90., 10. 90., 10. 10.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    // GDAL heading: "Polygon hole outside shell"
    // GDAL error: "Hole lies outside shell"
    // Our InteriorRingNotContainedInExteriorRing is a broader concept that covers this.
    #[test]
    fn polygon_hole_outside_shell() {
        let polygon = wkt!(POLYGON ((10. 90., 50. 90., 50. 10., 10. 10., 10. 90.), (60. 80., 90. 80., 90. 20., 60. 20., 60. 80.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                RingRole::Interior(0)
            )]
        );
    }

    // GDAL heading: "Hole partially outside polygon shell"
    // GDAL error: "Self-intersection"
    // GDAL sees the crossing exterior/interior ring boundaries as a self-intersection of the
    // polygon boundary. Our code instead reports that the interior ring is not contained within
    // the exterior — a different (and arguably more descriptive) classification.
    #[test]
    fn hole_partially_outside_polygon_shell() {
        let polygon = wkt!(POLYGON ((10. 90., 60. 90., 60. 10., 10. 10., 10. 90.), (30. 70., 90. 70., 90. 30., 30. 30., 30. 70.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                RingRole::Interior(0)
            )]
        );
    }

    // GDAL heading: "Polygon hole equal to shell"
    // GDAL error: "Self-intersection"
    // When the interior ring is identical to the exterior ring, GDAL calls this a
    // self-intersection. Our code reports InteriorRingNotContainedInExteriorRing (the coincident
    // ring is on the exterior boundary, not strictly inside it) followed by
    // IntersectingRingsOnALine (the exterior ring boundary and interior ring share a 1D set).
    #[test]
    fn polygon_hole_equal_to_shell() {
        let polygon = wkt!(POLYGON ((10. 90., 90. 90., 90. 10., 10. 10., 10. 90.), (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![
                InvalidPolygon::InteriorRingNotContainedInExteriorRing(RingRole::Interior(0)),
                InvalidPolygon::IntersectingRingsOnALine(RingRole::Exterior, RingRole::Interior(0))
            ]
        );
    }

    // GDAL heading: "Polygon holes overlap"
    // GDAL error: "Self-intersection"
    // GDAL uses "Self-intersection" broadly; our IntersectingRingsOnAnArea is more precise:
    // the two holes overlap in a 2D area.
    #[test]
    fn polygon_holes_overlap() {
        let polygon = wkt!(POLYGON (
            (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.),
            (80. 80., 80. 30., 30. 30., 30. 80., 80. 80.),
            (20. 20., 20. 70., 70. 70., 70. 20., 20. 20.)
        ));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::IntersectingRingsOnAnArea(
                RingRole::Interior(0),
                RingRole::Interior(1)
            )]
        );
    }

    // GDAL heading: "Polygon shell inside hole"
    // GDAL error: "Hole lies outside shell"
    // The interior ring (the larger square) is not contained within the exterior ring (the
    // smaller square). Our InteriorRingNotContainedInExteriorRing matches GDAL's concept.
    #[test]
    fn polygon_shell_inside_hole() {
        let polygon = wkt!(POLYGON (
            (30. 70., 70. 70., 70. 30., 30. 30., 30. 70.),
            (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.)
        ));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::InteriorRingNotContainedInExteriorRing(
                RingRole::Interior(0)
            )]
        );
    }

    // GDAL heading: "Self-crossing polygon shell"
    // GDAL error: "Self-intersection"
    #[test]
    fn self_crossing_polygon_shell() {
        let polygon = wkt!(POLYGON ((10. 70., 90. 70., 90. 50., 30. 50., 30. 30., 50. 30., 50. 90., 70. 90., 70. 10., 10. 10., 10. 70.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    // GDAL heading: "Self-overlapping polygon shell"
    // GDAL error: "Self-intersection"
    #[test]
    fn self_overlapping_polygon_shell() {
        let polygon = wkt!(POLYGON ((10. 90., 50. 90., 50. 30., 70. 30., 70. 50., 30. 50., 30. 70., 90. 70., 90. 10., 10. 10., 10. 90.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::SelfIntersection(RingRole::Exterior)]
        );
    }

    // GDAL heading: "Nested MultiPolygons"
    // GDAL error: "Nested shells"
    // GDAL uses the specific term "Nested shells" for this case. Our code detects the same
    // geometric invalidity (the inner polygon's interior intersects the outer polygon's interior)
    // but reports it as ElementsOverlaps rather than using nesting-specific terminology.
    #[test]
    fn nested_multipolygons() {
        let mp = wkt!(MULTIPOLYGON (
            ((30. 70., 70. 70., 70. 30., 30. 30., 30. 70.)),
            ((10. 90., 90. 90., 90. 10., 10. 10., 10. 90.))
        ));
        assert_eq!(
            mp.validation_errors(),
            vec![InvalidMultiPolygon::ElementsOverlaps(
                GeometryIndex(0),
                GeometryIndex(1)
            )]
        );
    }

    // GDAL heading: "Overlapping MultiPolygons"
    // GDAL error: "Self-intersection"
    // GDAL uses "Self-intersection" for overlapping MultiPolygon members; our ElementsOverlaps
    // describes the same geometric problem more precisely.
    #[test]
    fn overlapping_multipolygons() {
        let mp = wkt!(MULTIPOLYGON (
            ((10. 90., 60. 90., 60. 10., 10. 10., 10. 90.)),
            ((90. 80., 90. 20., 40. 20., 40. 80., 90. 80.))
        ));
        assert_eq!(
            mp.validation_errors(),
            vec![InvalidMultiPolygon::ElementsOverlaps(
                GeometryIndex(0),
                GeometryIndex(1)
            )]
        );
    }

    // GDAL heading: "MultiPolygon with multiple overlapping Polygons"
    // GDAL error: "Self-intersection"
    // Three mutually-overlapping polygons produce one ElementsOverlaps error per pair.
    #[test]
    fn multipolygon_with_multiple_overlapping_polygons() {
        let mp = wkt!(MULTIPOLYGON (
            ((90. 90., 90. 30., 30. 30., 30. 90., 90. 90.)),
            ((20. 20., 20. 80., 80. 80., 80. 20., 20. 20.)),
            ((10. 10., 10. 70., 70. 70., 70. 10., 10. 10.))
        ));
        assert_eq!(
            mp.validation_errors(),
            vec![
                InvalidMultiPolygon::ElementsOverlaps(GeometryIndex(0), GeometryIndex(1)),
                InvalidMultiPolygon::ElementsOverlaps(GeometryIndex(0), GeometryIndex(2)),
                InvalidMultiPolygon::ElementsOverlaps(GeometryIndex(1), GeometryIndex(2)),
            ]
        );
    }

    // GDAL heading: "MultiPolygon with two adjacent Polygons"
    // GDAL error: "Self-intersection"
    // The two polygons share a full edge (x=50, y=20..80). GDAL reports this as
    // "Self-intersection" because the combined boundary self-intersects along that shared edge.
    // Our code reports ElementsTouchOnALine — a distinct error variant describing boundary
    // contact rather than self-intersection. The semantics differ: GDAL treats edge-adjacency
    // as a form of self-intersection; we have a dedicated "touch on a line" concept.
    #[test]
    fn multipolygon_with_two_adjacent_polygons() {
        let mp = wkt!(MULTIPOLYGON (
            ((10. 90., 50. 90., 50. 10., 10. 10., 10. 90.)),
            ((90. 80., 90. 20., 50. 20., 50. 80., 90. 80.))
        ));
        // GDAL expects "Self-intersection"; we have no SelfIntersection variant for
        // MultiPolygon. Assert the GDAL-expected outcome (invalid due to overlap) which
        // differs from our ElementsTouchOnALine result.
        assert_eq!(
            mp.validation_errors(),
            vec![InvalidMultiPolygon::ElementsTouchOnALine(
                GeometryIndex(0),
                GeometryIndex(1)
            )]
        );
    }

    // GDAL heading: "Single-point polygon"
    // GDAL error: "point array must contain 0 or >1 elements" (GDAL-specific phrasing)
    // geo-types auto-closes the ring, resulting in [(70,30),(70,30)] — one distinct point,
    // which our TooFewPointsInRing check correctly rejects.
    #[test]
    fn single_point_polygon() {
        let polygon = wkt!(POLYGON ((70. 30.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::TooFewPointsInRing(RingRole::Exterior)]
        );
    }

    // GDAL heading: "Two-point polygon"
    // GDAL error: "Points of LinearRing do not form a closed linestring"
    // GDAL rejects this because the ring is not closed. geo-types auto-closes rings, so the
    // ring becomes [(10,10),(90,90),(10,10)] — two distinct points, still too few for a polygon.
    // Both GDAL and we reject it as invalid, though for slightly different stated reasons.
    #[test]
    fn two_point_polygon() {
        let polygon = wkt!(POLYGON ((10. 10., 90. 90.)));
        assert_eq!(
            polygon.validation_errors(),
            vec![InvalidPolygon::TooFewPointsInRing(RingRole::Exterior)]
        );
    }

    // GDAL heading: "Non-closed ring"
    // GDAL error: "Points of LinearRing do not form a closed linestring"
    // GDAL rejects POLYGON ((10 10, 90 10, 90 90, 10 90)) because the ring is not explicitly
    // closed. geo-types automatically closes all rings on construction, turning this into the
    // valid square POLYGON ((10 10, 90 10, 90 90, 10 90, 10 10)). Our code therefore considers
    // this geometry VALID.
    #[test]
    fn non_closed_ring() {
        let polygon = wkt!(POLYGON ((10. 10., 90. 10., 90. 90., 10. 90.)));
        assert!(polygon.exterior().is_closed());
        assert!(polygon.is_valid());
    }
}
