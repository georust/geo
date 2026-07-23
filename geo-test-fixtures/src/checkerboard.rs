//! Checkerboard polygon generator for testing simply-connected interior validation.
//!
//! The checkerboard pattern is useful for testing cycle detection: each pair of
//! adjacent holes shares exactly one vertex, and these single-touch connections
//! form cycles in the ring adjacency graph, disconnecting the interior.
//!
//! # Pattern Structure
//!
//! ```text
//!     0   1   2   3   4   5   6   7
//!   7 +-------------------------------+
//!     |                               |
//!   6 |   +---+   +---+   +---+       |
//!     |   | # |   | # |   | # |       |
//!   5 |   +---*---*---*---*---+       |
//!     |       | # |   | # |           |
//!   4 |   +---*---*---*---*---+       |   # = holes
//!     |   | # |   | # |   | # |       |   * = shared vertices
//!   3 |   +---*---*---*---*---+       |
//!     |       | # |   | # |           |
//!   2 |   +---*---*---*---*---+       |
//!     |   | # |   | # |   | # |       |
//!   1 |   +---+   +---+   +---+       |
//!     |                               |
//!   0 +-------------------------------+
//! ```

use geo::Translate;
use geo_types::{coord, LineString, Polygon};

/// Create a box as a closed [`LineString`] (for use as exterior or hole).
pub fn box_ring(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> LineString<f64> {
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
/// The pattern creates holes that share vertices at their corners.
/// Level 0 uses unit size 1, level 1 uses unit size 7, etc.
pub fn checkerboard_holes_at_level(level: usize) -> Vec<LineString<f64>> {
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

/// Create a checkerboard polygon with the given nesting level.
///
/// - Level 0: Simple 7x7 checkerboard with 13 holes
/// - Level 1: 49x49 with level-0 checkerboard nested inside one of the "solid" squares
/// - Level 2: 343x343 with level-1 nested inside, etc.
///
/// The nested checkerboards are placed at offset `(2*sz, 3*sz)` where
/// `sz` is the size of the outer checkerboard's unit cell. This places
/// the nested pattern inside the solid square at grid position (2,3).
///
/// Returns the polygon and its expected area.
pub fn create_checkerboard(level: usize) -> (Polygon<f64>, f64) {
    let base_sz: f64 = 7.0;
    let sz = base_sz.powi((level + 1) as i32);

    let exterior = box_ring(0.0, 0.0, sz, sz);
    let exterior_area = sz * sz;

    let mut all_holes: Vec<LineString<f64>> = Vec::new();
    let mut total_hole_area = 0.0;

    fn add_holes_recursive(
        all_holes: &mut Vec<LineString<f64>>,
        total_hole_area: &mut f64,
        current_level: usize,
        max_level: usize,
        offset_x: f64,
        offset_y: f64,
        base_sz: f64,
    ) {
        let unit_sz = base_sz.powi((max_level - current_level) as i32);

        let holes = checkerboard_holes_at_level(max_level - current_level);
        let hole_area = unit_sz * unit_sz;

        for hole in holes {
            all_holes.push(hole.translate(offset_x, offset_y));
            *total_hole_area += hole_area;
        }

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

/// Create a checkerboard polygon without computing expected area.
///
/// Convenience wrapper for benchmarking where only the polygon is needed.
pub fn create_checkerboard_polygon(level: usize) -> Polygon<f64> {
    create_checkerboard(level).0
}
