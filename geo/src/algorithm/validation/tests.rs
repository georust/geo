#[test]
fn jts_validation_tests() {
    jts_test_runner::assert_jts_tests_succeed("*Valid*");
}

mod simply_connected_interior {
    //! Tests for simply connected interior validation.
    //!
    //! OGC Simple Feature Specification (ISO 19125-1) states:
    //! "The interior of every Surface is a connected point set."
    //!
    //! These tests verify that we correctly detect polygons with disconnected
    //! interiors, which can occur when:
    //! - Two holes share 2+ vertices (creating a "corridor")
    //! - Three or more rings meet at a single vertex (creating a "pinch point")
    //! - Rings form a cycle of single-vertex touches that encloses part of the interior
    //!
    //! The checkerboard pattern is useful for testing cycle detection: each pair of
    //! adjacent holes shares exactly one vertex, and these single-touch connections
    //! form cycles in the ring adjacency graph, disconnecting the interior.

    use crate::algorithm::Validation;
    use crate::coord;
    use crate::geometry::{LineString, Polygon};
    use geo_test_fixtures::checkerboard::{box_ring, create_checkerboard};

    /// Two L-shaped holes sharing vertices at (2,2) and (3,3).
    ///
    /// This is the simplest case of disconnected interior - two holes that
    /// share exactly 2 vertices, creating a "corridor" between them.
    #[test]
    fn test_two_holes_sharing_two_vertices() {
        // Create exterior 5x5 box
        let exterior = box_ring(0.0, 0.0, 5.0, 5.0);

        // Create two L-shaped holes by specifying their vertices directly
        // Top-left L: covers (1,2)-(2,4) and (2,3)-(3,4)
        let top_l = LineString::new(vec![
            coord! { x: 1.0, y: 2.0 },
            coord! { x: 2.0, y: 2.0 },
            coord! { x: 2.0, y: 3.0 },
            coord! { x: 3.0, y: 3.0 },
            coord! { x: 3.0, y: 4.0 },
            coord! { x: 1.0, y: 4.0 },
            coord! { x: 1.0, y: 2.0 },
        ]);

        // Bottom-right L: covers (2,1)-(4,2) and (3,2)-(4,3)
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

        // The two holes share vertices at (2,2) and (3,3)
        // This disconnects the interior, making it OGC-invalid
        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);

        // Verify the error message includes the problematic coordinates
        let error_msg = format!("{}", errors[0]);
        assert!(
            error_msg.contains("(2, 2)") || error_msg.contains("(3, 3)"),
            "Error message should contain problematic coordinates: {}",
            error_msg
        );
    }

    /// Checkerboard level 0.
    ///
    /// 13 holes arranged in a checkerboard pattern where multiple holes
    /// share single vertices at grid intersections.
    #[test]
    fn test_checkerboard_level_0() {
        let (polygon, expected_area) = create_checkerboard(0);

        // Verify the geometry is constructed correctly
        assert_eq!(polygon.interiors().len(), 13);

        // Calculate actual area
        use crate::algorithm::Area;
        let actual_area = polygon.unsigned_area();
        assert!(
            (actual_area - expected_area).abs() < 1e-10,
            "Area mismatch: expected {}, got {}",
            expected_area,
            actual_area
        );

        // This should be invalid due to disconnected interior
        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);

        // The checkerboard has many shared vertices at grid intersections
        // Verify error message contains coordinates (any grid intersection point)
        let error_msg = format!("{}", errors[0]);
        assert!(
            error_msg.contains("polygon interior is not simply connected"),
            "Error message should indicate disconnected interior: {}",
            error_msg
        );
    }

    /// Checkerboard level 1.
    ///
    /// Nested checkerboard with level-0 pattern inside one of the "filled" squares.
    #[test]
    fn test_checkerboard_level_1() {
        let (polygon, expected_area) = create_checkerboard(1);

        // Level 1 has 13 holes at level 0 + 13 holes at level 1 = 26 holes
        assert_eq!(polygon.interiors().len(), 26);

        use crate::algorithm::Area;
        let actual_area = polygon.unsigned_area();
        assert!(
            (actual_area - expected_area).abs() < 1e-10,
            "Area mismatch: expected {}, got {}",
            expected_area,
            actual_area
        );

        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);

        // Verify error message contains coordinates
        let error_msg = format!("{}", errors[0]);
        assert!(
            error_msg.contains("polygon interior is not simply connected"),
            "Error message should indicate disconnected interior: {}",
            error_msg
        );
    }

    /// Valid polygon with holes that touch at single points.
    ///
    /// Two holes sharing exactly ONE vertex is valid (interior remains connected).
    #[test]
    fn test_holes_sharing_one_vertex_is_valid() {
        let exterior = box_ring(0.0, 0.0, 6.0, 6.0);

        // Two square holes that share exactly one vertex at (3, 3)
        let hole1 = box_ring(1.0, 1.0, 3.0, 3.0);
        let hole2 = box_ring(3.0, 3.0, 5.0, 5.0);

        let polygon = Polygon::new(exterior, vec![hole1, hole2]);

        // This should be valid - single shared vertex doesn't disconnect interior
        assert!(
            polygon.is_valid(),
            "Polygon with two holes sharing 1 vertex should be valid"
        );
    }

    /// Valid polygon with non-touching holes.
    #[test]
    fn test_separate_holes_is_valid() {
        let exterior = box_ring(0.0, 0.0, 10.0, 10.0);

        let hole1 = box_ring(1.0, 1.0, 3.0, 3.0);
        let hole2 = box_ring(5.0, 5.0, 7.0, 7.0);
        let hole3 = box_ring(1.0, 6.0, 3.0, 8.0);

        let polygon = Polygon::new(exterior, vec![hole1, hole2, hole3]);

        assert!(
            polygon.is_valid(),
            "Polygon with separate non-touching holes should be valid"
        );
    }

    /// Three triangular holes meeting at a single vertex (valid).
    ///
    /// When three holes share a single vertex but don't share any edges,
    /// the interior can still be connected. This "wedge" pattern creates
    /// three pie-slice holes meeting at the origin, but the interior regions
    /// between them connect around the perimeter.
    ///
    /// ```text
    ///              ● = shared vertex at origin A, B, C = triangular holes                                                                                                 
    /// ████████████████████████████████████████████████████████████████████████████████████████████
    /// █                                                                                          █
    /// █                                                                                          █
    /// █                                                                                          █
    /// █                                                                                          █
    /// █                       ███                                      ███                       █
    /// █                      █████                                    ██ ██                      █
    /// █                     ██   ██                                  ██   ██                     █
    /// █                    ██     ██                                ██     ██                    █
    /// █                   ██       ██                              ██       ██                   █
    /// █                  ██         ██                            ██         ███                 █
    /// █                ███           ██                          ██           ███                █
    /// █               ██              ██                        ██             ███               █
    /// █              ██                ██                      ██               ███              █
    /// █             ███                 ██                    ██                  ██             █
    /// █            ██                    ██                  ██                   ███            █
    /// █           ██                      ███               ██                      ██           █
    /// █          ██                         ██            ███                        ██          █
    /// █         ██           A              ███          ██             B             ██         █
    /// █        ██                            ███         ██                            ██        █
    /// █       ██                               ██      ███                              ██       █
    /// █      ██                                 ██    ██                                 ██      █
    /// █     ██                                   ██  ██                                   ██     █
    /// █    ██                                     █ █                                       █    █
    /// █  ██████████████████████████████████████████●██████████████████████████████████████████   █
    /// █                                           ███                                            █
    /// █                                          ██ ██                                           █
    /// █                                         ██   ██                                          █
    /// █                                       ███     ██                                         █
    /// █                                       ██       ██                                        █
    /// █                                     ███         ███                                      █
    /// █                                    ███           ███                                     █
    /// █                                   ███             ███                                    █
    /// █                                  ███               ███                                   █
    /// █                                 ███                  ██                                  █
    /// █                                ███                    ██                                 █
    /// █                               ██                       ██                                █
    /// █                              ██                         ██                               █
    /// █                             ██            C              ██                              █
    /// █                            ██                             ██                             █
    /// █                           ██                               ██                            █
    /// █                          ██                                 ██                           █
    /// █                         ██                                   ██                          █
    /// █                        ██                                     ██                         █
    /// █                       █ ████████████████████████████████████████ █                       █
    /// █                                                                                          █
    /// █                                                                                          █
    /// █                                                                                          █
    /// ████████████████████████████████████████████████████████████████████████████████████████████
    /// ```
    #[test]
    fn test_three_holes_meeting_at_vertex_valid() {
        let exterior = box_ring(-1.1, -1.1, 1.1, 1.1);

        // Three triangular holes meeting at origin (0, 0)
        // These are the alternating "pie slices" from a 60-degree wedge pattern
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

        // Three holes meeting at one vertex is NOT automatically invalid
        // The interior is still connected around the perimeter
        assert!(
            polygon.is_valid(),
            "Polygon with 3 holes meeting at one vertex (wedge pattern) should be valid"
        );
    }

    /// Four triangular holes forming a cycle through single touches.
    ///
    /// Each pair of holes shares exactly one vertex (no shared edges),
    /// and together they form a cycle in the touch graph: A-B-C-D-A.
    /// This encloses part of the interior, disconnecting it.
    ///
    /// ```text
    ///              ● = shared vertices between holes A, B, C, D
    /// ███████████████████████████████████████████████████████████████████████████████████████████████
    /// ██                                                                                           ██
    /// ██                                                                                           ██
    /// ██                                                                                           ██
    /// ██       ●███████████████████████████████████████████████████████████████████████████●       ██
    /// ██       ██ ███                                                                 ███ ██       ██
    /// ██       ███   ████                                                         █████  ███       ██
    /// ██       ██ ██    █████                                                  ████    ██ ██       ██
    /// ██       ██  ██      █████                     C                     █████      ███ ██       ██
    /// ██       ██   ██         █████                                   █████         ███  ██       ██
    /// ██       ██    ██            █████                           █████             ██   ██       ██
    /// ██       ██     ██               █████                    █████               ██    ██       ██
    /// ██       ██      ██                 █████             █████                  ██     ██       ██
    /// ██       ██      ██                     █████      ████                     ██      ██       ██
    /// ██       ██       ██                        ████████                       ██       ██       ██
    /// ██       ██        ██                                                     ██        ██       ██
    /// ██       ██         ██                                                   ██         ██       ██
    /// ██       ██          ██                                                 ██          ██       ██
    /// ██       ██           ██                                               ███          ██       ██
    /// ██       ██            ██                                             ███           ██       ██
    /// ██       ██             ██                                           ███            ██       ██
    /// ██       ██              ██                                         ███             ██       ██
    /// ██       ██               ██                                       ███              ██       ██
    /// ██       ██                ██                                      ██               ██       ██
    /// ██       ██                 █                                    ███                ██       ██
    /// ██       ██      B          ██                                    █         D       ██       ██
    /// ██       ██                ██                                      ██               ██       ██
    /// ██       ██               ██                                       ██               ██       ██
    /// ██       ██              ██                                         ██              ██       ██
    /// ██       ██             ██                                           ██             ██       ██
    /// ██       ██            ██                                             ██            ██       ██
    /// ██       ██           ██                                               ██           ██       ██
    /// ██       ██          ██                                                 ██          ██       ██
    /// ██       ██         ██                                                   ██         ██       ██
    /// ██       ██        ██                                                     ██        ██       ██
    /// ██       ██       ███                       ███████                        ██       ██       ██
    /// ██       ██      ███                    █████     █████                     ██      ██       ██
    /// ██       ██     ███                  ████             █████                  ██     ██       ██
    /// ██       ██     ██               █████                   █████                ██    ██       ██
    /// ██       ██    ██            █████                           █████            ██    ██       ██
    /// ██       ██   ██         ██████                                 ██████         ██   ██       ██
    /// ██       ██  ██       █████                    A                    █████       ██  ██       ██
    /// ██       ██ ██    █████                                                 █████    ██  █       ██
    /// ██       ███   ████                                                         ████   ███       ██
    /// ██       ██ ███                                                                 ███ ██       ██
    /// ██       ●███████████████████████████████████████████████████████████████████████████●       ██
    /// ██                                                                                           ██
    /// ██                                                                                           ██
    /// ██                                                                                           ██
    /// ███████████████████████████████████████████████████████████████████████████████████████████████
    /// ```
    #[test]
    fn test_four_holes_forming_cycle() {
        let exterior = box_ring(-10.0, -10.0, 10.0, 10.0);

        // Four triangular holes arranged in a cycle around the center
        // Each adjacent pair shares exactly one corner vertex
        // A: bottom - shares (-8,-8) with B, shares (8,-8) with D
        let hole_a = LineString::new(vec![
            coord! { x: -8.0, y: -8.0 },
            coord! { x: 0.0, y: -4.0 },
            coord! { x: 8.0, y: -8.0 },
            coord! { x: -8.0, y: -8.0 },
        ]);

        // B: left - shares (-8,8) with C, shares (-8,-8) with A
        let hole_b = LineString::new(vec![
            coord! { x: -8.0, y: 8.0 },
            coord! { x: -4.0, y: 0.0 },
            coord! { x: -8.0, y: -8.0 },
            coord! { x: -8.0, y: 8.0 },
        ]);

        // C: top - shares (8,8) with D, shares (-8,8) with B
        let hole_c = LineString::new(vec![
            coord! { x: 8.0, y: 8.0 },
            coord! { x: 0.0, y: 4.0 },
            coord! { x: -8.0, y: 8.0 },
            coord! { x: 8.0, y: 8.0 },
        ]);

        // D: right - shares (8,-8) with A, shares (8,8) with C
        let hole_d = LineString::new(vec![
            coord! { x: 8.0, y: -8.0 },
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 8.0, y: 8.0 },
            coord! { x: 8.0, y: -8.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![hole_a, hole_b, hole_c, hole_d]);

        // The four holes form a cycle: A-B at (-8,-8), B-C at (-8,8), C-D at (8,8), D-A at (8,-8)
        // This encloses the center region, disconnecting it from the corners
        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);

        // Verify the error message includes some of the cycle coordinates
        let error_msg = format!("{}", errors[0]);
        assert!(
            error_msg.contains("(-8, -8)")
                || error_msg.contains("(-8, 8)")
                || error_msg.contains("(8, 8)")
                || error_msg.contains("(8, -8)"),
            "Error message should contain cycle coordinates: {}",
            error_msg
        );
    }

    /// Three triangular holes forming a cycle through distinct touch points.
    ///
    /// Each pair of holes shares exactly one vertex at a different coordinate,
    /// forming a cycle: H0↔H1 at (10,5), H1↔H2 at (15,10), H2↔H0 at (5,10).
    /// This disconnects the interior (the center triangle is isolated).
    ///
    /// ```text
    ///     0      5      10      15      20
    ///  15 ┌─────────────────────────────────┐
    ///     │                                 │
    ///     │                                 │
    ///  10 │        ●───────────────●        │
    ///     │       ╱  ╲           ╱  ╲       │
    ///     │      ╱    ╲    H2   ╱    ╲      │
    ///     │     ╱      ╲       ╱      ╲     │
    ///     │    /        ╲     ╱        ╲    │
    ///     │   /   H0     ╲   ╱   H1     ╲   │
    ///     │  /            ╲ ╱            ╲  │
    ///   5 │ ●──────────────●──────────────● │
    ///     │             (10,5)              │
    ///     │          shared vertex          │
    ///     │                                 │
    ///   0 └─────────────────────────────────┘
    ///
    /// ● = shared vertices: (10,5) between H0-H1-H2, (15,10) between H1-H2, (5,10) between H0-H2
    /// ```
    #[test]
    fn test_three_holes_forming_triangle_cycle() {
        let exterior = box_ring(0.0, 0.0, 20.0, 15.0);

        // H0: bottom-left triangle
        let hole_0 = LineString::new(vec![
            coord! { x: 1.0, y: 5.0 },
            coord! { x: 10.0, y: 5.0 },
            coord! { x: 5.0, y: 10.0 },
            coord! { x: 1.0, y: 5.0 },
        ]);

        // H1: bottom-right triangle
        let hole_1 = LineString::new(vec![
            coord! { x: 10.0, y: 5.0 },
            coord! { x: 19.0, y: 5.0 },
            coord! { x: 15.0, y: 10.0 },
            coord! { x: 10.0, y: 5.0 },
        ]);

        // H2: top triangle (inverted, pointing down)
        let hole_2 = LineString::new(vec![
            coord! { x: 5.0, y: 10.0 },
            coord! { x: 15.0, y: 10.0 },
            coord! { x: 10.0, y: 5.0 },
            coord! { x: 5.0, y: 10.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![hole_0, hole_1, hole_2]);

        // H0-H2 share (5,10) and (10,5), H1-H2 share (10,5) and (15,10), H0-H1 share (10,5)
        // Multiple pairs share 2+ vertices → multiple disconnection errors
        let errors = polygon.validation_errors();
        assert!(
            !errors.is_empty(),
            "Should have at least one validation error"
        );

        // Verify at least one error message includes the shared coordinates
        let all_errors: String = errors.iter().map(|e| format!("{}", e)).collect();
        assert!(
            all_errors.contains("(5, 10)")
                || all_errors.contains("(10, 5)")
                || all_errors.contains("(15, 10)"),
            "Error messages should contain shared coordinates: {}",
            all_errors
        );
    }

    /// Two holes forming a chain with vertex-on-edge touches to exterior.
    ///
    /// Two triangular holes that:
    /// - Touch each other at a shared vertex (10, 5)
    /// - Each touches the exterior via vertex-on-edge (not at exterior vertices)
    ///
    /// ```text
    ///     0      5      10      15      20
    ///  15 ┌─────────────────────────────┐
    ///     │                             │
    ///     │                             │
    ///  10 │      ╱╲             ╱╲      │
    ///     │     ╱  ╲           ╱  ╲     │
    ///     │    ╱    ╲         ╱    ╲    │
    ///     │   ╱      ╲       ╱      ╲   │
    ///     │  /        ╲     ╱        ╲  │
    ///     │ /   H0     ╲   ╱   H1     ╲ │
    ///     │/            ╲ ╱            ╲│
    ///   5 ●──────────────●──────────────● (on edge)
    ///     │           (10,5)            │
    ///     │        shared vertex        │
    ///     │                             │
    ///   0 └─────────────────────────────┘
    ///                    
    ///
    /// ● = touch points on exterior edges (not at vertices)
    /// Chain: Exterior(0,5) → H0 → (10,5) → H1 → Exterior(20,5)
    /// ```
    #[test]
    fn test_hole_chain_with_vertex_on_edge_touch() {
        // Exterior: box from (0,0) to (20,15)
        let exterior = box_ring(0.0, 0.0, 20.0, 15.0);

        // H0: triangle touching exterior at (0,5) on left edge, sharing (10,5) with H1
        let hole_0 = LineString::new(vec![
            coord! { x: 0.0, y: 5.0 },
            coord! { x: 10.0, y: 5.0 },
            coord! { x: 5.0, y: 10.0 },
            coord! { x: 0.0, y: 5.0 },
        ]);

        // H1: triangle touching exterior at (20,5) on right edge, sharing (10,5) with H0
        let hole_1 = LineString::new(vec![
            coord! { x: 10.0, y: 5.0 },
            coord! { x: 20.0, y: 5.0 },
            coord! { x: 15.0, y: 10.0 },
            coord! { x: 10.0, y: 5.0 },
        ]);

        let polygon = Polygon::new(exterior, vec![hole_0, hole_1]);

        // Chain: Exterior(0,5) → H0 → (10,5) → H1 → Exterior(20,5)
        // Both (0,5) and (20,5) are on exterior edges (not at vertices)
        // This forms a cycle through the exterior with distinct coordinates
        let errors = polygon.validation_errors();
        assert_eq!(errors.len(), 1);

        // Verify the error message includes the touch coordinates
        let error_msg = format!("{}", errors[0]);
        assert!(
            error_msg.contains("(0, 5)")
                || error_msg.contains("(10, 5)")
                || error_msg.contains("(20, 5)"),
            "Error message should contain touch coordinates: {}",
            error_msg
        );
    }
}
