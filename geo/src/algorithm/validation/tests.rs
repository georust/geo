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
