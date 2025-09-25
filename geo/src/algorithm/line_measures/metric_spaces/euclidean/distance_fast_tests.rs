//! Edge case tests for the fast polygon-polygon distance algorithm
//!
//! This module contains tests for `distance_polygon_polygon_fast`,
//! algorithm that uses project-and-prune for linearly-separable polygons.
//!
//! All test polygons are valid OGC polygons (closed, non-self-intersecting) and are
//! linearly separable (their bounding boxes don't overlap on at least one axis).
//!
//! All tests confirmed to trigger the fast path

// Results can be verified against e.g. PostGIS:
//
// WITH test_cases AS (
//   SELECT
//     'diagonal_separation' AS test_name,
//     'POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))'::geometry AS poly1,
//     'POLYGON((20 20, 30 20, 30 30, 20 30, 20 20))'::geometry AS poly2,
//     14.142135623730951 AS expected_distance
//
//   UNION ALL SELECT
//     'c_shaped_non_convex',
//     'POLYGON((0 0, 5 0, 5 5, 10 5, 10 15, 5 15, 5 10, 0 10, 0 0))'::geometry,
//     'POLYGON((15 7, 20 7, 20 8, 15 8, 15 7))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'star_polygon_fast',
//     'POLYGON((10 0, 11 4, 15 5, 11 6, 10 10, 9 6, 5 5, 9 4, 10 0))'::geometry,
//     'POLYGON((20 0, 25 0, 25 10, 20 10, 20 0))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'barely_separated_bboxes',
//     'POLYGON((0 0, 10 0, 10 10, 0 10, 0 0))'::geometry,
//     'POLYGON((11 5, 20 5, 20 15, 11 15, 11 5))'::geometry,
//     1.0
//
//   UNION ALL SELECT
//     'vertex_vertex_closest',
//     'POLYGON((0 0, 10 0, 5 8, 0 0))'::geometry,
//     'POLYGON((15 0, 20 0, 17 8, 15 0))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'elongated_skinny_polygons',
//     'POLYGON((0 0, 1 0, 1 100, 0 100, 0 0))'::geometry,
//     'POLYGON((10 45, 50 45, 50 55, 10 55, 10 45))'::geometry,
//     9.0
//
//   UNION ALL SELECT
//     'fortyfive_degree_separation',
//     'POLYGON((0 0, 5 0, 5 5, 0 5, 0 0))'::geometry,
//     'POLYGON((10 10, 15 10, 15 15, 10 15, 10 10))'::geometry,
//     7.0710678118654755
//
//   UNION ALL SELECT
//     'edge_edge_parallel',
//     'POLYGON((0 0, 10 0, 10 10, 0 0))'::geometry,
//     'POLYGON((15 0, 25 0, 25 10, 15 0))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'zigzag_non_convex',
//     'POLYGON((0 0, 2 5, 4 0, 6 5, 8 0, 8 10, 0 10, 0 0))'::geometry,
//     'POLYGON((15 2, 20 2, 20 8, 15 8, 15 2))'::geometry,
//     7.0
//
//   UNION ALL SELECT
//     'minimum_vertex_triangles',
//     'POLYGON((0.0 0.0, 5.0 0.0, 2.5 4.0, 0.0 0.0))'::geometry,
//     'POLYGON((10.0 0.0, 15.0 0.0, 12.5 4.0, 10.0 0.0))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'l_shaped_non_convex',
//     'POLYGON((0 0, 5 0, 5 5, 10 5, 10 10, 0 10, 0 0))'::geometry,
//     'POLYGON((15 2, 20 2, 20 8, 15 8, 15 2))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'narrow_gap_complex_shapes',
//     'POLYGON((0 0, 3 0, 3 2, 1 2, 1 4, 3 4, 3 6, 0 6, 0 0))'::geometry,
//     'POLYGON((4 1, 7 1, 7 3, 5 3, 5 5, 7 5, 7 7, 4 7, 4 1))'::geometry,
//     1.0
//
//   UNION ALL SELECT
//     'hexagons_separated',
//     'POLYGON((5.0 0.0, 7.5 2.0, 7.5 6.0, 5.0 8.0, 2.5 6.0, 2.5 2.0, 5.0 0.0))'::geometry,
//     'POLYGON((15.0 0.0, 17.5 2.0, 17.5 6.0, 15.0 8.0, 12.5 6.0, 12.5 2.0, 15.0 0.0))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'vertical_separation_overlap_horizontal',
//     'POLYGON((0 0, 10 0, 10 5, 0 5, 0 0))'::geometry,
//     'POLYGON((3 10, 7 10, 7 15, 3 15, 3 10))'::geometry,
//     5.0
//
//   UNION ALL SELECT
//     'concave_pentagon_vs_triangle',
//     'POLYGON((0 0, 4 0, 4 3, 2 2, 0 3, 0 0))'::geometry,
//     'POLYGON((10.0 1.0, 15.0 1.0, 12.5 4.0, 10.0 1.0))'::geometry,
//     6.0
// )
// SELECT
//   test_name,
//   expected_distance,
//   ST_Distance(poly1, poly2) AS postgis_distance,
//   abs(ST_Distance(poly1, poly2) - expected_distance) AS difference,
//   CASE
//     WHEN abs(ST_Distance(poly1, poly2) - expected_distance) < 1e-10 THEN 'PASS'
//     ELSE 'FAIL'
//   END AS status
// FROM test_cases
// ORDER BY test_name;

use super::*;
use crate::{Convert, Polygon};
use geo_types::{LineString, wkt};

#[test]
fn diagonal_separation() {
    // Two squares separated diagonally (bottom-left vs top-right)
    // Tests when separation occurs on both x and y axes simultaneously
    let square1: Polygon = wkt! { POLYGON((0 0, 10 0, 10 10, 0 10, 0 0)) }.convert();
    let square2: Polygon = wkt! { POLYGON((20 20, 30 20, 30 30, 20 30, 20 20)) }.convert();

    let distance = Euclidean.distance(&square1, &square2);
    // Distance from (10,10) to (20,20) = sqrt((20-10)^2 + (20-10)^2) = sqrt(200) ≈ 14.142
    assert_relative_eq!(distance, 14.142135623730951, epsilon = 1e-10);
}

#[test]
fn c_shaped_non_convex() {
    // C-shape opening towards a rectangle
    // Tests non-convex polygon where concavity faces the other polygon
    let c_shape: Polygon =
        wkt! { POLYGON((0 0, 5 0, 5 5, 10 5, 10 15, 5 15, 5 10, 0 10, 0 0)) }.convert();
    let rect: Polygon = wkt! { POLYGON((15 7, 20 7, 20 8, 15 8, 15 7)) }.convert();

    let distance = Euclidean.distance(&c_shape, &rect);
    // Distance from right edge of C-shape (x=10) to left edge of rect (x=15) at y≈7.5
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn star_polygon_fast() {
    // 8-pointed star vs rectangle
    // Tests polygon with alternating concave/convex points
    let star: Polygon =
        wkt! { POLYGON((10 0, 11 4, 15 5, 11 6, 10 10, 9 6, 5 5, 9 4, 10 0)) }.convert();
    let rect: Polygon = wkt! { POLYGON((20 0, 25 0, 25 10, 20 10, 20 0)) }.convert();

    let distance = Euclidean.distance(&star, &rect);
    // Distance from rightmost point of star (x=15) to left edge of rect (x=20)
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn barely_separated_bboxes() {
    // Bounding boxes separated by exactly 1 unit on x-axis
    let rect1: Polygon = wkt! { POLYGON((0 0, 10 0, 10 10, 0 10, 0 0)) }.convert();
    let rect2: Polygon = wkt! { POLYGON((11 5, 20 5, 20 15, 11 15, 11 5)) }.convert();

    let distance = Euclidean.distance(&rect1, &rect2);
    // Horizontal gap between rectangles
    assert_relative_eq!(distance, 1.0, epsilon = 1e-10);
}

#[test]
fn vertex_vertex_closest() {
    // Two triangles where closest points are vertices (not edges)
    // Tests when the minimum distance occurs between two specific vertices
    let tri1: Polygon = wkt! { POLYGON((0 0, 10 0, 5 8, 0 0)) }.convert();
    let tri2: Polygon = wkt! { POLYGON((15 0, 20 0, 17 8, 15 0)) }.convert();

    let distance = Euclidean.distance(&tri1, &tri2);
    // Distance from right vertex of tri1 (10,0) to left vertex of tri2 (15,0)
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn elongated_skinny_polygons() {
    // Very tall narrow rectangle vs wide flat rectangle
    // Tests extreme aspect ratios and different polygon orientations
    let tall: Polygon = wkt! { POLYGON((0 0, 1 0, 1 100, 0 100, 0 0)) }.convert();
    let wide: Polygon = wkt! { POLYGON((10 45, 50 45, 50 55, 10 55, 10 45)) }.convert();

    let distance = Euclidean.distance(&tall, &wide);
    // Horizontal distance from tall rect (x=1) to wide rect (x=10)
    assert_relative_eq!(distance, 9.0, epsilon = 1e-10);
}

#[test]
fn fortyfive_degree_separation() {
    // Squares separated at 45° angle
    // Tests slope calculation when perpendicular slope is exactly -1
    let square1: Polygon = wkt! { POLYGON((0 0, 5 0, 5 5, 0 5, 0 0)) }.convert();
    let square2: Polygon = wkt! { POLYGON((10 10, 15 10, 15 15, 10 15, 10 10)) }.convert();

    let distance = Euclidean.distance(&square1, &square2);
    // Distance from (5,5) to (10,10) = sqrt((10-5)^2 + (10-5)^2) = sqrt(50) ≈ 7.071
    assert_relative_eq!(distance, 7.0710678118654755, epsilon = 1e-10);
}

#[test]
fn edge_edge_parallel() {
    // Two triangles with parallel edges as the closest features
    // Tests when minimum distance is between two parallel line segments
    let tri1: Polygon = wkt! { POLYGON((0 0, 10 0, 10 10, 0 0)) }.convert();
    let tri2: Polygon = wkt! { POLYGON((15 0, 25 0, 25 10, 15 0)) }.convert();

    let distance = Euclidean.distance(&tri1, &tri2);
    // Distance between vertical edges at x=10 and x=15
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn zigzag_non_convex() {
    // Zigzag polygon vs rectangle
    // Tests complex non-convex boundary with multiple potential closest points
    let zigzag: Polygon = wkt! { POLYGON((0 0, 2 5, 4 0, 6 5, 8 0, 8 10, 0 10, 0 0)) }.convert();
    let rect: Polygon = wkt! { POLYGON((15 2, 20 2, 20 8, 15 8, 15 2)) }.convert();

    let distance = Euclidean.distance(&zigzag, &rect);
    // Distance from rightmost point of zigzag (x=8) to left edge of rect (x=15)
    assert_relative_eq!(distance, 7.0, epsilon = 1e-10);
}

#[test]
fn minimum_vertex_triangles() {
    // Two triangles (minimum valid polygon vertex count)
    // Tests algorithm with simplest possible polygons
    let tri1: Polygon<f64> = wkt! { POLYGON((0.0 0.0, 5.0 0.0, 2.5 4.0, 0.0 0.0)) };
    let tri2: Polygon<f64> = wkt! { POLYGON((10.0 0.0, 15.0 0.0, 12.5 4.0, 10.0 0.0)) };

    let distance = Euclidean.distance(&tri1, &tri2);
    // Distance from rightmost point of tri1 (5,0) to leftmost point of tri2 (10,0)
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn l_shaped_non_convex() {
    // L-shaped polygon vs rectangle
    // Tests another common non-convex shape
    let l_shape: Polygon = wkt! { POLYGON((0 0, 5 0, 5 5, 10 5, 10 10, 0 10, 0 0)) }.convert();
    let rect: Polygon = wkt! { POLYGON((15 2, 20 2, 20 8, 15 8, 15 2)) }.convert();

    let distance = Euclidean.distance(&l_shape, &rect);
    // Distance from rightmost point of L (x=10) to left edge of rect (x=15)
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn narrow_gap_complex_shapes() {
    // Two complex non-convex shapes with narrow gap
    // Tests small distances between complex geometries
    let shape1: Polygon = wkt! { POLYGON((0 0, 3 0, 3 2, 1 2, 1 4, 3 4, 3 6, 0 6, 0 0)) }.convert();
    let shape2: Polygon = wkt! { POLYGON((4 1, 7 1, 7 3, 5 3, 5 5, 7 5, 7 7, 4 7, 4 1)) }.convert();

    let distance = Euclidean.distance(&shape1, &shape2);
    // Minimum horizontal gap between the shapes
    assert_relative_eq!(distance, 1.0, epsilon = 1e-10);
}

#[test]
fn hexagons_separated() {
    // Two regular hexagons separated horizontally
    let hex1: Polygon<f64> =
        wkt! { POLYGON((5.0 0.0, 7.5 2.0, 7.5 6.0, 5.0 8.0, 2.5 6.0, 2.5 2.0, 5.0 0.0)) };
    let hex2: Polygon<f64> =
        wkt! { POLYGON((15.0 0.0, 17.5 2.0, 17.5 6.0, 15.0 8.0, 12.5 6.0, 12.5 2.0, 15.0 0.0)) };

    let distance = Euclidean.distance(&hex1, &hex2);
    // Distance from rightmost point of hex1 to leftmost point of hex2
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn vertical_separation_overlap_horizontal() {
    // Two rectangles: separated vertically, overlapping horizontally
    // Specifically tests y-axis separation with x-axis overlap
    let lower: Polygon = wkt! { POLYGON((0 0, 10 0, 10 5, 0 5, 0 0)) }.convert();
    let upper: Polygon = wkt! { POLYGON((3 10, 7 10, 7 15, 3 15, 3 10)) }.convert();

    let distance = Euclidean.distance(&lower, &upper);
    // Vertical gap between rectangles
    assert_relative_eq!(distance, 5.0, epsilon = 1e-10);
}

#[test]
fn concave_pentagon_vs_triangle() {
    // Concave pentagon vs triangle
    // Tests different vertex counts and concavity
    let pentagon: Polygon = wkt! { POLYGON((0 0, 4 0, 4 3, 2 2, 0 3, 0 0)) }.convert();
    let triangle: Polygon<f64> = wkt! { POLYGON((10.0 1.0, 15.0 1.0, 12.5 4.0, 10.0 1.0)) };

    let distance = Euclidean.distance(&pentagon, &triangle);
    // Distance from rightmost point of pentagon to leftmost point of triangle
    assert_relative_eq!(distance, 6.0, epsilon = 1e-10);
}

// These are also new, but not included in the PostGIS verification

#[test]
fn test_y_axis_separated_polygons() {
    // Two polygons separated along the y-axis (one above the other)
    let poly1: Polygon = wkt! { POLYGON((0 0, 10 0, 10 5, 0 5, 0 0)) }.convert();
    let poly2: Polygon = wkt! { POLYGON((2 10, 8 10, 8 15, 2 15, 2 10)) }.convert();

    // Bounding boxes overlap in x-axis but are separated in y-axis
    // This should now trigger the project-and-sort algorithm
    let distance = Euclidean.distance(&poly1, &poly2);
    assert_relative_eq!(distance, 5.0); // Distance from top of poly1 to bottom of poly2
}

#[test]
fn test_separable_distance() {
    // Sanity check: `a` is a triangle pointing right. `b` is a rectangle strictly right of `a`.
    //             _
    // a->  |>    |_| <-b
    let a: Polygon = wkt! { POLYGON ((160 70, 160 320, 400 200, 160 70)) }.convert();
    let b: Polygon = wkt! { POLYGON ((500 100, 500 300, 700 300, 700 100, 500 100)) }.convert();
    assert_eq!(100.0, Euclidean.distance(&a, &b));

    // Leave the base of `b`, but extend it's upper area to trigger a pathological case in the overlap heuristic.
    // Intuitively, since the closest edge (the left edge of `b`) is still there, it can't be farther than
    // the previous test case.
    //          ____
    //         [_   | <-b
    //           \  |
    //            | |
    //            | |
    // a->  |>    |_|
    let a: Polygon = wkt! { POLYGON ((160 70, 160 320, 400 200, 160 70)) }.convert();
    let b: Polygon =
        wkt! { POLYGON ((450 500, 485 449, 500 400, 500 200, 900 500, 640 520, 450 500)) }
            .convert();
    assert_eq!(100.0, Euclidean.distance(&a, &b));
}

#[test]
fn linestring_linestring_horizontal_separation() {
    // Two horizontal linestrings separated along x-axis
    let ls1 = LineString::from(vec![(0.0, 5.0), (10.0, 5.0), (10.0, 10.0)]);
    let ls2 = LineString::from(vec![(15.0, 5.0), (25.0, 5.0), (25.0, 10.0)]);
    let distance = Euclidean.distance(&ls1, &ls2);
    assert_relative_eq!(distance, 5.0);
}

#[test]
fn linestring_linestring_vertical_separation() {
    // Two vertical linestrings separated along y-axis
    let ls1 = LineString::from(vec![(5.0, 0.0), (5.0, 10.0), (10.0, 10.0)]);
    let ls2 = LineString::from(vec![(5.0, 15.0), (5.0, 25.0), (10.0, 25.0)]);
    let distance = Euclidean.distance(&ls1, &ls2);
    assert_relative_eq!(distance, 5.0);
}

#[test]
fn linestring_linestring_diagonal_separation() {
    // Two linestrings separated diagonally
    let ls1 = LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
    let ls2 = LineString::from(vec![(20.0, 20.0), (30.0, 20.0), (30.0, 30.0)]);
    let distance = Euclidean.distance(&ls1, &ls2);
    // Distance from (10, 10) to (20, 20)
    assert_relative_eq!(distance, (100.0_f64 + 100.0_f64).sqrt());
}

#[test]
fn linestring_linestring_parallel() {
    // Two parallel horizontal linestrings
    let ls1 = LineString::from(vec![(0.0, 0.0), (10.0, 0.0)]);
    let ls2 = LineString::from(vec![(0.0, 5.0), (10.0, 5.0)]);
    let distance = Euclidean.distance(&ls1, &ls2);
    assert_relative_eq!(distance, 5.0);
}

#[test]
fn linestring_linestring_barely_separated() {
    // Two linestrings with bboxes barely separated
    let ls1 = LineString::from(vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)]);
    let ls2 = LineString::from(vec![(11.0, 5.0), (20.0, 5.0), (20.0, 15.0)]);
    let distance = Euclidean.distance(&ls1, &ls2);
    assert_relative_eq!(distance, 1.0);
}
