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
    //! - Multiple holes share single vertices in a pattern that disconnects the interior
    //!
    //! The checkerboard pattern is particularly useful for testing because it creates
    //! geometries where 4 holes meet at single vertices, forming a grid pattern that
    //! disconnects the interior even though no two holes share more than one vertex.

    use crate::algorithm::Validation;
    use crate::coord;
    use crate::geometry::{LineString, Polygon};

    /// Create a box as a closed LineString (for use as exterior or hole).
    fn box_ring(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> LineString<f64> {
        LineString::new(vec![
            coord! { x: min_x, y: min_y },
            coord! { x: max_x, y: min_y },
            coord! { x: max_x, y: max_y },
            coord! { x: min_x, y: max_y },
            coord! { x: min_x, y: min_y },
        ])
    }

    /// Generate checkerboard hole positions for a given level.
    ///
    /// The pattern creates holes that share vertices at their corners:
    /// ```text
    ///     0   1   2   3   4   5   6   7
    ///   7 ┌───────────────────────────┐
    ///     │                           │
    ///   6 │   ┌───┐   ┌───┐   ┌───┐   │
    ///     │   │ ░ │   │ ░ │   │ ░ │   │
    ///   5 │   └───●───●───●───●───┘   │
    ///     │       │ ░ │   │ ░ │       │
    ///   4 │   ┌───●───●───●───●───┐   │
    ///     │   │ ░ │   │ ░ │   │ ░ │   │   ░ = holes
    ///   3 │   └───●───●───●───●───┘   │   ● = shared vertices
    ///     │       │ ░ │   │ ░ │       │
    ///   2 │   ┌───●───●───●───●───┐   │
    ///     │   │ ░ │   │ ░ │   │ ░ │   │
    ///   1 │   └───┘   └───┘   └───┘   │
    ///     │                           │
    ///   0 └───────────────────────────┘
    /// ```
    fn checkerboard_holes_at_level(level: usize) -> Vec<LineString<f64>> {
        let base_sz: f64 = 7.0;
        let sz = base_sz.powi(level as i32);

        let mut holes = Vec::new();

        // Diagonal holes: (i, i) for i in 1..6
        for i in 1..6 {
            let fi = i as f64;
            holes.push(box_ring(fi * sz, fi * sz, (fi + 1.0) * sz, (fi + 1.0) * sz));
        }

        // Off-diagonal holes above the diagonal
        for i in 1..4 {
            let fi = i as f64;
            holes.push(box_ring(
                fi * sz,
                (fi + 2.0) * sz,
                (fi + 1.0) * sz,
                (fi + 3.0) * sz,
            ));
        }
        for i in 1..2 {
            let fi = i as f64;
            holes.push(box_ring(
                fi * sz,
                (fi + 4.0) * sz,
                (fi + 1.0) * sz,
                (fi + 5.0) * sz,
            ));
        }

        // Off-diagonal holes below the diagonal
        for i in 1..4 {
            let fi = i as f64;
            holes.push(box_ring(
                (fi + 2.0) * sz,
                fi * sz,
                (fi + 3.0) * sz,
                (fi + 1.0) * sz,
            ));
        }
        for i in 1..2 {
            let fi = i as f64;
            holes.push(box_ring(
                (fi + 4.0) * sz,
                fi * sz,
                (fi + 5.0) * sz,
                (fi + 1.0) * sz,
            ));
        }

        holes
    }

    /// Translate a ring by an offset.
    fn translate_ring(ring: &LineString<f64>, dx: f64, dy: f64) -> LineString<f64> {
        LineString::new(
            ring.coords()
                .map(|c| coord! { x: c.x + dx, y: c.y + dy })
                .collect(),
        )
    }

    /// Create a checkerboard polygon with the given nesting level.
    ///
    /// - Level 0: Simple 7x7 checkerboard with 13 holes
    /// - Level 1: 49x49 with level-0 checkerboard nested inside one of the "solid" squares
    /// - Level 2: 343x343 with level-1 nested inside, etc.
    ///
    /// The Python implementation places nested checkerboards at offset (2*sz, 3*sz) where
    /// sz is the size of the outer checkerboard's unit cell. This places the nested pattern
    /// inside the solid square at grid position (2,3).
    ///
    /// Returns the polygon and its expected area.
    fn create_checkerboard(level: usize) -> (Polygon<f64>, f64) {
        let base_sz: f64 = 7.0;
        let sz = base_sz.powi((level + 1) as i32);

        let exterior = box_ring(0.0, 0.0, sz, sz);
        let exterior_area = sz * sz;

        let mut all_holes: Vec<LineString<f64>> = Vec::new();
        let mut total_hole_area = 0.0;

        // Start with level 0 holes at the outermost scale
        // Then recursively add smaller checkerboards inside one of the "solid" squares
        fn add_holes_recursive(
            all_holes: &mut Vec<LineString<f64>>,
            total_hole_area: &mut f64,
            current_level: usize,
            max_level: usize,
            offset_x: f64,
            offset_y: f64,
            base_sz: f64,
        ) {
            // Size of unit cell at this level
            let unit_sz = base_sz.powi((max_level - current_level) as i32);

            // Add the 13 holes for this checkerboard level
            let holes = checkerboard_holes_at_level(max_level - current_level);
            let hole_area = unit_sz * unit_sz;

            for hole in holes {
                let translated = translate_ring(&hole, offset_x, offset_y);
                all_holes.push(translated);
                *total_hole_area += hole_area;
            }

            // If not at the innermost level, recurse into one of the solid squares
            // The solid square at position (2, 3) in the 7x7 grid
            if current_level < max_level {
                let next_offset_x = offset_x + 2.0 * unit_sz;
                let next_offset_y = offset_y + 3.0 * unit_sz;
                add_holes_recursive(
                    all_holes,
                    total_hole_area,
                    current_level + 1,
                    max_level,
                    next_offset_x,
                    next_offset_y,
                    base_sz,
                );
            }
        }

        add_holes_recursive(
            &mut all_holes,
            &mut total_hole_area,
            0,
            level,
            0.0,
            0.0,
            base_sz,
        );

        let polygon = Polygon::new(exterior, all_holes);
        let expected_area = exterior_area - total_hole_area;

        (polygon, expected_area)
    }

    /// Test case: Two L-shaped holes sharing vertices at (2,2) and (3,3).
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
        assert!(
            !polygon.is_valid(),
            "Polygon with two holes sharing 2 vertices should be invalid (disconnected interior)"
        );
    }

    /// Test case: Checkerboard level 0.
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
        assert!(
            !polygon.is_valid(),
            "Checkerboard level 0 should be invalid (disconnected interior)"
        );
    }

    /// Test case: Checkerboard level 1.
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

        assert!(
            !polygon.is_valid(),
            "Checkerboard level 1 should be invalid (disconnected interior)"
        );
    }

    /// Test case: Valid polygon with holes that touch at single points.
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

    /// Test case: Valid polygon with non-touching holes.
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
}
