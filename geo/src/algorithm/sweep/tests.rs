use super::*;
use crate::algorithm::line_intersection::line_intersection;
use crate::{GeoFloat, Line};

fn compute_brute_force_intersections<T: GeoFloat>(
    lines: &[Line<T>],
) -> Vec<(Line<T>, Line<T>, LineIntersection<T>)> {
    let mut result = Vec::new();
    for i in 0..lines.len() {
        for j in (i + 1)..lines.len() {
            if let Some(isect) = line_intersection(lines[i], lines[j]) {
                result.push((lines[i], lines[j], isect));
            }
        }
    }
    result
}

/// Helper function to verify that sweep line and brute force find the same intersections
fn verify_intersections(lines: &[Line<f64>]) {
    // Get intersections using both algorithms
    let sweep_intersections: Vec<_> = Intersections::from_iter(lines).collect();
    let brute_force_intersections = compute_brute_force_intersections(lines);

    // Check for same count
    assert_eq!(
        sweep_intersections.len(),
        brute_force_intersections.len(),
        "Sweep algorithm found {} intersections but brute force found {}",
        sweep_intersections.len(),
        brute_force_intersections.len()
    );

    // Check that all brute force intersections are found by sweep
    // and that their intersection details match
    for (bf_line1, bf_line2, bf_intersection) in &brute_force_intersections {
        let matching_intersection = sweep_intersections.iter().find(|(line1, line2, _)| {
            (*line1 == bf_line1 && *line2 == bf_line2) || (*line1 == bf_line2 && *line2 == bf_line1)
        });

        assert!(
            matching_intersection.is_some(),
            "Sweep algorithm missed an intersection between lines"
        );

        // Compare the actual intersection details
        #[allow(clippy::assertions_on_constants)]
        if let Some((_, _, sw_intersection)) = matching_intersection {
            match (bf_intersection, sw_intersection) {
                (
                    LineIntersection::SinglePoint {
                        intersection: bf_point,
                        is_proper: bf_proper,
                    },
                    LineIntersection::SinglePoint {
                        intersection: sw_point,
                        is_proper: sw_proper,
                    },
                ) => {
                    assert_eq!(
                        *bf_proper, *sw_proper,
                        "Proper intersection status doesn't match for the same line pairs"
                    );

                    assert_eq!(bf_point, sw_point,);
                }
                (
                    LineIntersection::Collinear {
                        intersection: bf_line,
                    },
                    LineIntersection::Collinear {
                        intersection: sw_line,
                    },
                ) => {
                    assert!(
                        (*bf_line == *sw_line)
                            || (*bf_line == Line::new(sw_line.end, sw_line.start)),
                        "Collinear intersection lines don't match: brute force: {:?}, sweep: {:?}",
                        bf_line,
                        sw_line
                    );
                }
                _ => {
                    assert!(
                        false,
                        "Intersection types don't match: brute force: {:?}, sweep: {:?}",
                        bf_intersection, sw_intersection
                    );
                }
            }
        }
    }

    // Check that all sweep intersections are found by brute force
    for (sw_line1, sw_line2, _) in sweep_intersections {
        let found = brute_force_intersections.iter().any(|(line1, line2, _)| {
            (line1 == sw_line1 && line2 == sw_line2) || (line1 == sw_line2 && line2 == sw_line1)
        });

        assert!(
            found,
            "Sweep algorithm found an intersection not found by brute force"
        );
    }
}

#[test]
fn should_not_panic() {
    // from https://github.com/georust/geo/issues/1342
    // these segments cause a panic in the existing sweep line impl:
    // segment not found in active-vec-set: 4
    let segments = vec![
        Line::from([(10.0, 6.0), (10.0, 8.0)]),
        Line::from([(10.0, 6.0), (8.0, 10.0)]),
        Line::from([(10.0, 6.0), (8.0, 8.0)]),
        Line::from([(10.0, 6.0), (8.0, 4.0)]),
        Line::from([(10.0, 6.0), (2.0, 8.0)]),
        Line::from([(10.0, 4.0), (10.0, 8.0)]),
        Line::from([(10.0, 8.0), (8.0, 10.0)]),
        Line::from([(8.0, 10.0), (8.0, 8.0)]),
        Line::from([(8.0, 8.0), (4.0, 2.0)]),
        Line::from([(2.0, 2.0), (4.0, 10.0)]),
        Line::from([(2.0, 2.0), (6.0, 10.0)]),
        Line::from([(4.0, 8.0), (4.0, 10.0)]),
        Line::from([(2.0, 6.0), (6.0, 10.0)]),
        Line::from([(6.0, 10.0), (10.0, 2.0)]),
        Line::from([(10.0, 2.0), (10.0, 10.0)]),
        Line::from([(10.0, 10.0), (8.0, 2.0)]),
        Line::from([(8.0, 6.0), (6.0, 8.0)]),
        Line::from([(8.0, 6.0), (6.0, 2.0)]),
        Line::from([(8.0, 6.0), (6.0, 4.0)]),
        Line::from([(8.0, 6.0), (4.0, 6.0)]),
        Line::from([(8.0, 6.0), (2.0, 4.0)]),
        Line::from([(2.0, 4.0), (6.0, 8.0)]),
        Line::from([(6.0, 4.0), (4.0, 6.0)]),
        Line::from([(4.0, 4.0), (2.0, 4.0)]),
        Line::from([(10.0, 6.0), (2.0, 2.0)]),
        Line::from([(2.0, 2.0), (8.0, 6.0)]),
    ];
    verify_intersections(&segments);
}

#[test]
fn test_iterator_behavior() {
    let input = vec![
        Line::from([(1., 0.), (0., 1.)]),
        Line::from([(0., 0.), (1., 1.)]),
    ];

    // They intersect at (0.5, 0.5)
    let intersections: Vec<_> = Intersections::from_iter(&input).collect();

    // There should be one intersection
    assert_eq!(intersections.len(), 1);

    // Check intersection details
    let (_line1, _line2, intersection) = &intersections[0];

    assert!(matches!(
        intersection,
        LineIntersection::SinglePoint {
            intersection: _,
            is_proper: true
        }
    ));
}

#[test]
fn test_multiple_intersections() {
    let input = vec![
        Line::from([(1., 0.), (0., 1.)]),
        Line::from([(0., 0.75), (1., 0.25)]),
        Line::from([(0., 0.25), (1., 0.75)]),
        Line::from([(0., 0.), (1., 1.)]),
    ];

    verify_intersections(&input);
}

#[test]
fn test_brute_force_comparison() {
    // Create a set of lines with various intersections
    let input = vec![
        Line::from([(0., 0.), (10., 10.)]),
        Line::from([(0., 10.), (10., 0.)]),
        Line::from([(5., 0.), (5., 10.)]),
        Line::from([(0., 5.), (10., 5.)]),
    ];

    verify_intersections(&input);
}

/// Generate a grid explicitly with known intersections for easier debugging
fn generate_explicit_grid(size: usize) -> Vec<Line<f64>> {
    let mut lines = Vec::with_capacity(size * 2);

    // Create a grid of "size" horizontal and "size" vertical lines
    // crossing at exact integer coordinates
    for i in 0..size {
        // Convert to float for line coordinates
        let pos = i as f64;

        // Horizontal line at y = pos
        lines.push(Line::from([(-10.0, pos), (10.0, pos)]));

        // Vertical line at x = pos
        lines.push(Line::from([(pos, -10.0), (pos, 10.0)]));
    }

    // Should have exactly size² intersections
    lines
}

/// Test to diagnose and fix the Bentley-Ottmann algorithm for grid patterns
#[test]
fn test_debug_grid_algorithm() {
    // Create a small grid pattern to find the intersections missed by Bentley-Ottmann
    for size in [3, 5, 7, 10] {
        let lines = generate_explicit_grid(size);

        // Expected number of intersections in grid
        let expected_intersections = size * size;

        let sweep_results: Vec<_> = Intersections::from_iter(&lines).collect();
        assert_eq!(
            sweep_results.len(),
            expected_intersections,
            "Sweep algorithm should find {} grid intersections for {}x{} grid",
            expected_intersections,
            size,
            size
        );
    }
}

// Test to diagnose issues with line_intersection for horizontal-vertical pairs
#[test]
fn test_horizontal_vertical_intersection() {
    // Create a grid of 5 horizontal and 5 vertical lines
    let size = 5;
    let mut missing_count = 0;

    // Generate explicit horizontal and vertical lines
    let mut lines = Vec::with_capacity(size * 2);

    // Create horizontal lines
    let mut horizontal_lines = Vec::with_capacity(size);
    for i in 0..size {
        let y = i as f64;
        horizontal_lines.push(Line::from([(-10.0, y), (10.0, y)]));
    }

    // Create vertical lines
    let mut vertical_lines = Vec::with_capacity(size);
    for i in 0..size {
        let x = i as f64;
        vertical_lines.push(Line::from([(x, -10.0), (x, 10.0)]));
    }

    // Add all lines to the main list
    lines.extend(horizontal_lines.iter().cloned());
    lines.extend(vertical_lines.iter().cloned());

    for h_line in horizontal_lines.iter() {
        for v_line in vertical_lines.iter() {
            // Check if these lines should intersect
            let result = line_intersection(*h_line, *v_line);
            // Check if the intersection was found
            if result.is_none() {
                missing_count += 1;
            }
        }
    }
    // Make sure we found all intersections
    assert_eq!(
        missing_count, 0,
        "All horizontal-vertical pairs should intersect. We're missing {} intersections",
        missing_count
    );
}

/// Test with nearly parallel lines that have small differences
#[test]
fn test_nearly_parallel_lines() {
    // Set of nearly parallel lines with very small differences in angle
    let lines = vec![
        // Base line
        Line::from([(0.0, 0.0), (10.0, 1.0)]),
        // Nearly parallel lines with small differences
        Line::from([(0.0, 0.0), (10.0, 1.0000001)]), // Extremely small angle diff
        Line::from([(0.0, 0.0), (10.0, 1.00001)]),   // Very small angle diff
        Line::from([(0.0, 0.0), (10.0, 0.99999)]),   // Very small angle diff (opposite)
        Line::from([(0.0, 0.1), (10.0, 1.1)]),       // Parallel but offset
        // Intersecting lines at small angles
        Line::from([(0.1, 0.0), (10.0, 0.9)]), // Intersects baseline
        Line::from([(5.0, 0.0), (5.0, 10.0)]), // Vertical intersector
    ];

    verify_intersections(&lines);
}

/// Test with lines having coordinates near powers of 2
#[test]
fn test_power_of_two_boundaries() {
    // Lines with coordinates near powers of 2, where floating point precision changes
    let lines = vec![
        // Near 2^10 = 1024
        Line::from([(1023.0, 1023.0), (1025.0, 1025.0)]),
        Line::from([(1023.0, 1025.0), (1025.0, 1023.0)]),
        // Near 2^20 = 1,048,576
        Line::from([(1048575.0, 1048575.0), (1048577.0, 1048577.0)]),
        Line::from([(1048575.0, 1048577.0), (1048577.0, 1048575.0)]),
        // Mixed scales
        Line::from([(1024.0, 1024.0), (1048576.0, 1048576.0)]),
        Line::from([(1024.0, 1048576.0), (1048576.0, 1024.0)]),
    ];

    verify_intersections(&lines);
}

/// Test with extremely large coordinate values
#[test]
fn test_large_coordinates() {
    // Lines with very large coordinate values
    let large = 1e15; // 10^15
    let lines = vec![
        Line::from([(large, large), (large + 10.0, large + 10.0)]),
        Line::from([(large, large + 10.0), (large + 10.0, large)]),
        Line::from([(large, large), (large, large + 10.0)]),
        Line::from([(large, large), (large + 10.0, large)]),
    ];

    verify_intersections(&lines);
}

/// Test with extremely small coordinate values and differences
#[test]
fn test_small_coordinates() {
    // Lines with very small coordinate values and differences
    let small = 1e-10; // 10^-10
    let lines = vec![
        Line::from([(small, small), (small * 2.0, small * 2.0)]),
        Line::from([(small, small * 2.0), (small * 2.0, small)]),
        Line::from([(0.0, 0.0), (small, small)]),
        Line::from([(0.0, small), (small, 0.0)]),
        // Very small differences between endpoints
        Line::from([(small, 0.0), (small + 1e-15, 1e-15)]),
        Line::from([(0.0, small), (1e-15, small + 1e-15)]),
    ];

    verify_intersections(&lines);
}

/// Test with collinear or almost-collinear segments
#[test]
fn test_collinear_segments() {
    // Collinear and nearly-collinear segments
    let lines = vec![
        // Two collinear segments that don't overlap
        Line::from([(0.0, 0.0), (1.0, 1.0)]),
        Line::from([(2.0, 2.0), (3.0, 3.0)]),
        // Two collinear segments that overlap
        Line::from([(4.0, 4.0), (6.0, 6.0)]),
        Line::from([(5.0, 5.0), (7.0, 7.0)]),
        // Nearly collinear segments (tiny offset)
        Line::from([(8.0, 8.0), (10.0, 10.0)]),
        Line::from([(9.0, 9.0 + 1e-10), (11.0, 11.0 + 1e-10)]),
    ];

    verify_intersections(&lines);
}

/// Test with lines forming very shallow angles
#[test]
fn test_shallow_angle_intersections() {
    // Lines that intersect at very shallow angles
    let lines = vec![
        Line::from([(0.0, 0.0), (100.0, 1.0)]),
        Line::from([(0.0, 0.1), (100.0, 0.0)]),
        // Even shallower
        Line::from([(0.0, 0.0), (1000.0, 1.0)]),
        Line::from([(0.0, 0.1), (1000.0, 0.0)]),
        // Extremely shallow
        Line::from([(0.0, 0.0), (10000.0, 1.0)]),
        Line::from([(0.0, 0.001), (10000.0, 0.0)]),
    ];

    verify_intersections(&lines);
}

/// Test with a mix of all previous precision-challenging cases
#[test]
fn test_mixed_precision_challenges() {
    // Combining various challenging cases
    let lines = vec![
        // Nearly parallel with small y offset
        Line::from([(0.0, 0.0), (10.0, 1.0)]),
        Line::from([(0.0, 1e-10), (10.0, 1.0 + 1e-10)]),
        // Power of 2 boundary intersections
        Line::from([(1024.0, 0.0), (1024.0, 10.0)]),
        Line::from([(0.0, 5.0), (2048.0, 5.0)]),
        // Large coordinate values
        Line::from([(1e12, 1e12), (1e12 + 10.0, 1e12 + 10.0)]),
        Line::from([(1e12, 1e12 + 10.0), (1e12 + 10.0, 1e12)]),
        // Small differences and shallow angles
        Line::from([(0.0, 0.0), (10000.0, 1.0)]),
        Line::from([(0.0, 0.5), (10000.0, 0.4)]),
    ];

    verify_intersections(&lines);
}

#[test]
fn test_overlapping_segments() {
    let segments = vec![
        // First polygon set
        Line::from([(10.0, 10.0), (15.0, 10.0)]),
        Line::from([(15.0, 10.0), (15.0, 20.0)]),
        Line::from([(15.0, 20.0), (10.0, 10.0)]),
        Line::from([(10.0, -10.0), (15.0, -10.0)]),
        Line::from([(15.0, -10.0), (15.0, -20.0)]),
        Line::from([(15.0, -20.0), (10.0, -10.0)]),
        Line::from([(20.0, 10.0), (25.0, 10.0)]),
        Line::from([(25.0, 10.0), (25.0, 20.0)]),
        Line::from([(25.0, 20.0), (20.0, 10.0)]),
        Line::from([(20.0, -10.0), (25.0, -10.0)]),
        Line::from([(25.0, -10.0), (25.0, -20.0)]),
        Line::from([(25.0, -20.0), (20.0, -10.0)]),
        // Second polygon set
        Line::from([(10.0, 10.0), (15.0, 10.0)]),
        Line::from([(15.0, 10.0), (15.0, 15.0)]),
        Line::from([(15.0, 15.0), (10.0, 10.0)]),
        Line::from([(10.0, -10.0), (15.0, -10.0)]),
        Line::from([(15.0, -10.0), (15.0, -15.0)]),
        Line::from([(15.0, -15.0), (10.0, -10.0)]),
        Line::from([(20.0, 10.0), (25.0, 10.0)]),
        Line::from([(25.0, 10.0), (25.0, 5.0)]),
        Line::from([(25.0, 5.0), (20.0, 10.0)]),
        Line::from([(20.0, -10.0), (25.0, -10.0)]),
        Line::from([(25.0, -10.0), (25.0, -5.0)]),
        Line::from([(25.0, -5.0), (20.0, -10.0)]),
    ];

    verify_intersections(&segments);
}

#[test]
fn test_intersections_at_endpoints() {
    // This test creates line segments that specifically test endpoint intersections
    let segments = vec![
        // Star-like pattern with intersections at endpoints
        Line::from([(-9.0, 10.0), (9.0, 10.0)]),
        Line::from([(9.0, 10.0), (0.1, 0.1)]),
        Line::from([(0.1, 0.1), (10.0, 9.0)]),
        Line::from([(10.0, 9.0), (10.0, -9.0)]),
        Line::from([(10.0, -9.0), (0.1, -0.1)]),
        Line::from([(0.1, -0.1), (9.0, -10.0)]),
        Line::from([(9.0, -10.0), (-9.0, -10.0)]),
        Line::from([(-9.0, -10.0), (-0.1, -0.1)]),
        Line::from([(-0.1, -0.1), (-10.0, -9.0)]),
        Line::from([(-10.0, -9.0), (-10.0, 9.0)]),
        Line::from([(-10.0, 9.0), (-0.1, 0.1)]),
        Line::from([(-0.1, 0.1), (-9.0, 10.0)]),
        // Same pattern with slight offsets (to test nearly identical points)
        Line::from([(-8.99999, 10.0), (8.99999, 10.0)]),
        Line::from([(8.99999, 10.0), (0.1, 0.1)]),
        Line::from([(0.1, 0.1), (10.0, 8.99999)]),
        Line::from([(10.0, 8.99999), (10.0, -8.99999)]),
        Line::from([(10.0, -8.99999), (0.1, -0.1)]),
        Line::from([(0.1, -0.1), (8.99999, -10.0)]),
        Line::from([(8.99999, -10.0), (-8.99999, -10.0)]),
        Line::from([(-8.99999, -10.0), (-0.1, -0.1)]),
        Line::from([(-0.1, -0.1), (-10.0, -8.99999)]),
        Line::from([(-10.0, -8.99999), (-10.0, 8.99999)]),
        Line::from([(-10.0, 8.99999), (-0.1, 0.1)]),
        Line::from([(-0.1, 0.1), (-8.99999, 10.0)]),
    ];

    verify_intersections(&segments);
}

/// Test with very thin triangles that create nearly parallel lines
#[test]
fn test_thin_triangles() {
    // These thin triangles create segments that are nearly parallel
    // and test the algorithm's handling of precision issues
    let segments = vec![
        // First triangle (very thin)
        Line::from([(0.0, 0.0), (100.0, 0.0)]),
        Line::from([(100.0, 0.0), (50.0, 0.001)]),
        Line::from([(50.0, 0.001), (0.0, 0.0)]),
        // Second triangle (very thin, slightly offset)
        Line::from([(0.0, 0.1), (100.0, 0.1)]),
        Line::from([(100.0, 0.1), (50.0, 0.101)]),
        Line::from([(50.0, 0.101), (0.0, 0.1)]),
        // Intersector lines
        Line::from([(25.0, -1.0), (25.0, 2.0)]),
        Line::from([(75.0, -1.0), (75.0, 2.0)]),
    ];

    verify_intersections(&segments);
}

#[test]
fn test_checkerboard_pattern() {
    let mut segments = Vec::new();

    // Create a grid of squares (3x3 checkerboard)
    let size = 3;
    let square_size = 10.0;

    for row in 0..size {
        for col in 0..size {
            let x = col as f64 * square_size;
            let y = row as f64 * square_size;

            // Add the square
            segments.push(Line::from([(x, y), (x + square_size, y)]));
            segments.push(Line::from([
                (x + square_size, y),
                (x + square_size, y + square_size),
            ]));
            segments.push(Line::from([
                (x + square_size, y + square_size),
                (x, y + square_size),
            ]));
            segments.push(Line::from([(x, y + square_size), (x, y)]));

            // Add diagonal lines
            if (row + col) % 2 == 0 {
                segments.push(Line::from([(x, y), (x + square_size, y + square_size)]));
            } else {
                segments.push(Line::from([(x + square_size, y), (x, y + square_size)]));
            }
        }
    }

    verify_intersections(&segments);
}

/// Test inspired by the "hourglasses" shape
#[test]
fn test_hourglasses() {
    // Create hourglass shapes that test overlapping segments
    let segments = vec![
        // First hourglass
        Line::from([(0.0, 0.0), (10.0, 10.0)]),
        Line::from([(10.0, 10.0), (0.0, 20.0)]),
        Line::from([(0.0, 20.0), (10.0, 30.0)]),
        Line::from([(10.0, 30.0), (0.0, 40.0)]),
        Line::from([(0.0, 0.0), (0.0, 40.0)]),
        // Second hourglass (overlapping with the first)
        Line::from([(5.0, 5.0), (15.0, 15.0)]),
        Line::from([(15.0, 15.0), (5.0, 25.0)]),
        Line::from([(5.0, 25.0), (15.0, 35.0)]),
        Line::from([(15.0, 35.0), (5.0, 45.0)]),
        Line::from([(5.0, 5.0), (5.0, 45.0)]),
        // Connecting lines to create more intersections
        Line::from([(0.0, 10.0), (15.0, 10.0)]),
        Line::from([(0.0, 30.0), (15.0, 30.0)]),
    ];

    verify_intersections(&segments);
}
