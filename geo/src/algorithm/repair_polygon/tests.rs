use approx::assert_relative_eq;

use super::*;
use crate::Area;
use geo_types::wkt;

// ====== Helpers ======

fn assert_valid(mp: &MultiPolygon<f64>) {
    use crate::validation::Validation;
    for (i, poly) in mp.iter().enumerate() {
        assert!(
            poly.is_valid(),
            "polygon {i} is invalid: {:?}",
            poly.validation_errors()
        );
    }
}

fn total_area(mp: &MultiPolygon<f64>) -> f64 {
    mp.unsigned_area()
}

/// Run the same line preparation pipeline as build_cdt (snap + odd-even +
/// split + odd-even) for testing the line preparation logic directly.
fn prepare_lines_for_repair(lines: Vec<Line<f64>>) -> Vec<Line<f64>> {
    use crate::algorithm::triangulate_delaunay::snap_or_register_point;
    use rstar::RTree;

    let snap_radius = 0.0001;
    let mut known_coords: RTree<Coord<f64>> = RTree::new();
    let mut lines: Vec<Line<f64>> = lines
        .into_iter()
        .map(|mut line| {
            line.start = snap_or_register_point(line.start, &mut known_coords, snap_radius);
            line.end = snap_or_register_point(line.end, &mut known_coords, snap_radius);
            line
        })
        .filter(|l| l.start != l.end)
        .collect();
    odd_even_filter(&mut lines);
    let mut lines = split_segments_at_intersections(lines, known_coords, snap_radius).unwrap();
    odd_even_filter(&mut lines);
    lines
}

// ====== Valid input should survive repair ======

#[test]
fn valid_square_unchanged() {
    let square: Polygon<f64> = wkt!(POLYGON((0. 0., 10. 0., 10. 10., 0. 10., 0. 0.)));
    let repaired = square.make_valid().unwrap();
    assert_valid(&repaired);
    assert_eq!(repaired.0.len(), 1);
    assert_relative_eq!(total_area(&repaired), 100.0);
}

#[test]
fn valid_polygon_with_hole() {
    let poly: Polygon<f64> = wkt!(POLYGON(
        (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
        (2. 2., 2. 8., 8. 8., 8. 2., 2. 2.)
    ));
    let repaired = poly.make_valid().unwrap();
    assert_valid(&repaired);
    assert_eq!(repaired.0.len(), 1);
    assert_relative_eq!(total_area(&repaired), 64.0);
}

#[test]
fn valid_triangle() {
    let tri: Polygon<f64> = wkt!(POLYGON((0. 0., 4. 0., 2. 3., 0. 0.)));
    let repaired = tri.make_valid().unwrap();
    assert_valid(&repaired);
    assert_eq!(repaired.0.len(), 1);
    assert_relative_eq!(total_area(&repaired), 6.0);
}

#[test]
fn inverted_polygon() {
    // CW exterior ring (wrong winding) should still repair
    let inverted: Polygon<f64> = wkt!(POLYGON((0. 0., 0. 10., 10. 10., 10. 0., 0. 0.)));
    let repaired = inverted.make_valid().unwrap();
    assert_valid(&repaired);
    assert_eq!(repaired.0.len(), 1);
    assert_relative_eq!(total_area(&repaired), 100.0);
}

#[test]
fn large_polygon_with_many_vertices() {
    let n = 100;
    let coords: Vec<Coord<f64>> = (0..n)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            Coord {
                x: 100.0 * angle.cos(),
                y: 100.0 * angle.sin(),
            }
        })
        .collect();
    let poly = Polygon::new(LineString::new(coords), vec![]);
    let repaired = poly.make_valid().unwrap();
    assert_valid(&repaired);
    assert_eq!(repaired.0.len(), 1);
    let area = total_area(&repaired);
    assert!(
        area > 31000.0 && area < 31500.0,
        "circle-like polygon area {area} out of expected range"
    );
}

// ====== Degenerate input ======

#[test]
fn empty_polygon_returns_empty() {
    let empty = Polygon::<f64>::new(LineString::new(vec![]), vec![]);
    let repaired = empty.make_valid().unwrap();
    assert!(repaired.0.is_empty());
}

#[test]
fn degenerate_line_returns_empty() {
    let line: Polygon<f64> = wkt!(POLYGON((0. 0., 5. 5., 0. 0.)));
    let repaired = line.make_valid().unwrap();
    assert_relative_eq!(total_area(&repaired), 0.0);
}

// ====== Error handling ======

#[test]
fn make_valid_succeeds_on_valid_input() {
    let square: Polygon<f64> = wkt!(POLYGON((0. 0., 10. 0., 10. 10., 0. 10., 0. 0.)));
    let result = square.make_valid();
    assert!(result.is_ok());
    assert_valid(&result.unwrap());
}

#[test]
fn make_valid_returns_nan_error() {
    let poly = Polygon::new(
        LineString::new(vec![
            Coord {
                x: 0.0,
                y: f64::NAN,
            },
            Coord { x: 1.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
            Coord {
                x: 0.0,
                y: f64::NAN,
            },
        ]),
        vec![],
    );
    let result = poly.make_valid();
    assert_eq!(result.unwrap_err(), RepairPolygonError::CoordinateIsNaN);
}

#[test]
fn make_valid_returns_too_large_error() {
    let huge = 1e300;
    let poly = Polygon::new(
        LineString::new(vec![
            Coord { x: 0.0, y: 0.0 },
            Coord { x: huge, y: 0.0 },
            Coord { x: 0.0, y: huge },
            Coord { x: 0.0, y: 0.0 },
        ]),
        vec![],
    );
    let result = poly.make_valid();
    assert_eq!(result.unwrap_err(), RepairPolygonError::CoordinateTooLarge);
}

// ====== Cross-validation against prepair ======
//
// These tests compare our make_valid output topologically against the WKT
// produced by the reference prepair binary, using relate(...).is_equal_topo().
//
// How the reference WKT was obtained:
//   1. Built prepair (commit 7b4a5c0) from https://github.com/tudelft3d/prepair
//      against CGAL 6.1.1 on macOS (arm64).
//   2. Ran each input WKT through: prepair --wkt '<INPUT>'
//   3. Recorded the verbatim output as the `prepair_wkt` argument below.
//
// The inputs come from two sources:
//   - The prepair README examples:
//     https://github.com/tudelft3d/prepair?tab=readme-ov-file#details
//   - The GDAL geometry validity documentation:
//     https://gdal.org/en/latest/user/geometry_validity.html
//
// Topological equality (is_equal_topo) verifies that our output and prepair's represent
// exactly the same point set, not just the same area or polygon count.

use crate::Relate;
use wkt::TryFromWkt;

fn assert_topo_eq_prepair_polygon(name: &str, input: Polygon<f64>, prepair_wkt: &str) {
    let repaired = input.make_valid().unwrap();
    assert_valid(&repaired);
    let expected = MultiPolygon::<f64>::try_from_wkt_str(prepair_wkt).unwrap_or_else(|_| {
        // POLYGON EMPTY or single polygon -- try parsing as Polygon and wrapping
        let p = Polygon::<f64>::try_from_wkt_str(prepair_wkt).expect("could not parse prepair WKT");
        MultiPolygon::new(vec![p])
    });
    assert!(
        repaired.relate(&expected).is_equal_topo(),
        "{name}: topologically unequal\n  ours:    {repaired:?}\n  prepair: {expected:?}"
    );
}

fn assert_topo_eq_prepair_multi(name: &str, input: MultiPolygon<f64>, prepair_wkt: &str) {
    let repaired = input.make_valid().unwrap();
    assert_valid(&repaired);
    let expected =
        MultiPolygon::<f64>::try_from_wkt_str(prepair_wkt).expect("could not parse prepair WKT");
    assert!(
        repaired.relate(&expected).is_equal_topo(),
        "{name}: topologically unequal\n  ours:    {repaired:?}\n  prepair: {expected:?}"
    );
}

// -- prepair README examples --

#[test]
fn prepair_bowtie() {
    assert_topo_eq_prepair_polygon(
        "bowtie",
        wkt!(POLYGON((0. 0., 0. 10., 10. 0., 10. 10., 0. 0.))),
        "MULTIPOLYGON (((0 0,5 5,0 10,0 0)),((5 5,10 0,10 10,5 5)))",
    );
}

#[test]
fn prepair_inner_ring_sharing_edge_with_outer() {
    assert_topo_eq_prepair_polygon(
        "inner_ring_sharing_edge",
        wkt!(POLYGON(
            (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
            (5. 2., 5. 7., 10. 7., 10. 2., 5. 2.)
        )),
        "POLYGON ((0 0,10 0,10 2,5 2,5 7,10 7,10 10,0 10,0 0))",
    );
}

#[test]
fn prepair_dangling_edge() {
    assert_topo_eq_prepair_polygon(
        "dangling_edge",
        wkt!(POLYGON((0. 0., 10. 0., 15. 5., 10. 0., 10. 10., 0. 10., 0. 0.))),
        "POLYGON ((0 0,10 0,10 10,0 10,0 0))",
    );
}

#[test]
fn prepair_two_adjacent_inner_rings() {
    assert_topo_eq_prepair_polygon(
        "two_adjacent_inner_rings",
        wkt!(POLYGON(
            (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
            (1. 1., 1. 8., 3. 8., 3. 1., 1. 1.),
            (3. 1., 3. 8., 5. 8., 5. 1., 3. 1.)
        )),
        "POLYGON ((0 0,10 0,10 10,0 10,0 0),(1 1,1 8,3 8,5 8,5 1,3 1,1 1))",
    );
}

#[test]
fn prepair_nested_inner_rings() {
    assert_topo_eq_prepair_polygon(
        "nested_inner_rings",
        wkt!(POLYGON(
            (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
            (2. 8., 5. 8., 5. 2., 2. 2., 2. 8.),
            (3. 3., 4. 3., 3. 4., 3. 3.)
        )),
        "MULTIPOLYGON (((0 0,10 0,10 10,0 10,0 0),(2 2,2 8,5 8,5 2,2 2)),((3 3,4 3,3 4,3 3)))",
    );
}

// -- GDAL polygon cases (cross-validated against prepair) --

#[test]
fn gdal_self_intersecting_polygon() {
    assert_topo_eq_prepair_polygon(
        "gdal_self_intersecting",
        wkt!(POLYGON((10. 90., 90. 10., 90. 90., 10. 10., 10. 90.))),
        "MULTIPOLYGON (((50 50,90 10,90 90,50 50)),((10 10,50 50,10 90,10 10)))",
    );
}

#[test]
fn gdal_hole_outside_shell() {
    assert_topo_eq_prepair_polygon(
        "gdal_hole_outside_shell",
        wkt!(POLYGON(
            (10. 90., 50. 90., 50. 10., 10. 10., 10. 90.),
            (60. 80., 90. 80., 90. 20., 60. 20., 60. 80.)
        )),
        "MULTIPOLYGON (((10 10,50 10,50 90,10 90,10 10)),((60 20,90 20,90 80,60 80,60 20)))",
    );
}

#[test]
fn gdal_hole_equal_to_shell() {
    // prepair returns POLYGON EMPTY; our make_valid returns an empty MultiPolygon.
    let poly = wkt!(POLYGON(
        (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.),
        (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.)
    ));
    let repaired = poly.make_valid().unwrap();
    assert!(repaired.0.is_empty(), "hole equal to shell should be empty");
}

#[test]
fn gdal_shell_inside_hole() {
    assert_topo_eq_prepair_polygon(
        "gdal_shell_inside_hole",
        wkt!(POLYGON(
            (30. 70., 70. 70., 70. 30., 30. 30., 30. 70.),
            (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.)
        )),
        "POLYGON ((10 10,90 10,90 90,10 90,10 10),(30 30,30 70,70 70,70 30,30 30))",
    );
}

#[test]
fn gdal_nested_multipolygons() {
    assert_topo_eq_prepair_multi(
        "gdal_nested_multi",
        wkt!(MULTIPOLYGON(
            ((30. 70., 70. 70., 70. 30., 30. 30., 30. 70.)),
            ((10. 90., 90. 90., 90. 10., 10. 10., 10. 90.))
        )),
        "MULTIPOLYGON (((10 10,90 10,90 90,10 90,10 10),(30 30,30 70,70 70,70 30,30 30)))",
    );
}

#[test]
fn gdal_adjacent_multipolygons() {
    assert_topo_eq_prepair_multi(
        "gdal_adjacent_multi",
        wkt!(MULTIPOLYGON(
            ((10. 90., 50. 90., 50. 10., 10. 10., 10. 90.)),
            ((90. 80., 90. 20., 50. 20., 50. 80., 90. 80.))
        )),
        "MULTIPOLYGON (((10 10,50 10,50 20,90 20,90 80,50 80,50 90,10 90,10 10)))",
    );
}

// -- Other cases (cross-validated against prepair) --

#[test]
fn bowtie_becomes_two_triangles() {
    assert_topo_eq_prepair_polygon(
        "small_bowtie",
        wkt!(POLYGON((0. 0., 2. 2., 2. 0., 0. 2., 0. 0.))),
        "MULTIPOLYGON (((0 0,1 1,0 2,0 0)),((1 1,2 0,2 2,1 1)))",
    );
}

#[test]
fn spike_polygon() {
    assert_topo_eq_prepair_polygon(
        "spike",
        wkt!(POLYGON((0. 0., 10. 0., 10. 10., 5. 10., 5. 20., 5. 10., 0. 10., 0. 0.))),
        "POLYGON ((0 0,10 0,10 10,5 10,0 10,0 0))",
    );
}

#[test]
fn touching_rings_at_point() {
    assert_topo_eq_prepair_polygon(
        "touching_rings",
        wkt!(POLYGON(
            (0. 0., 10. 0., 10. 10., 0. 10., 0. 0.),
            (5. 0., 8. 3., 2. 3., 5. 0.)
        )),
        "POLYGON ((0 0,5 0,10 0,10 10,0 10,0 0),(2 3,8 3,5 0,2 3))",
    );
}

#[test]
fn multipolygon_overlapping_squares() {
    assert_topo_eq_prepair_multi(
        "overlapping_squares",
        wkt!(MULTIPOLYGON(
            ((0. 0., 10. 0., 10. 10., 0. 10., 0. 0.)),
            ((5. 5., 15. 5., 15. 15., 5. 15., 5. 5.))
        )),
        "MULTIPOLYGON (((0 0,10 0,10 5,5 5,5 10,0 10,0 0)),((5 10,10 10,10 5,15 5,15 15,5 15,5 10)))",
    );
}

#[test]
fn nested_shells() {
    assert_topo_eq_prepair_polygon(
        "nested_shells",
        wkt!(POLYGON(
            (0. 0., 20. 0., 20. 20., 0. 20., 0. 0.),
            (5. 5., 15. 5., 15. 15., 5. 15., 5. 5.)
        )),
        "POLYGON ((0 0,20 0,20 20,0 20,0 0),(5 5,5 15,15 15,15 5,5 5))",
    );
}

// -- Remaining GDAL cases (cross-validated against prepair) --

#[test]
fn gdal_self_touching_ring() {
    assert_topo_eq_prepair_polygon(
        "gdal_self_touching_ring",
        wkt!(POLYGON((10. 10., 90. 10., 90. 40., 80. 20., 70. 40., 80. 60., 90. 40., 90. 90., 10. 90., 10. 10.))),
        "POLYGON ((10 10,90 10,90 40,90 90,10 90,10 10),(70 40,80 60,90 40,80 20,70 40))",
    );
}

#[test]
fn gdal_hole_partially_outside_shell() {
    assert_topo_eq_prepair_polygon(
        "gdal_hole_partially_outside",
        wkt!(POLYGON(
            (10. 90., 60. 90., 60. 10., 10. 10., 10. 90.),
            (30. 70., 90. 70., 90. 30., 30. 30., 30. 70.)
        )),
        "MULTIPOLYGON (((10 10,60 10,60 30,30 30,30 70,60 70,60 90,10 90,10 10)),((60 30,90 30,90 70,60 70,60 30)))",
    );
}

#[test]
fn gdal_holes_overlap() {
    assert_topo_eq_prepair_polygon(
        "gdal_holes_overlap",
        wkt!(POLYGON(
            (10. 90., 90. 90., 90. 10., 10. 10., 10. 90.),
            (80. 80., 80. 30., 30. 30., 30. 80., 80. 80.),
            (20. 20., 20. 70., 70. 70., 70. 20., 20. 20.)
        )),
        "MULTIPOLYGON (((10 10,90 10,90 90,10 90,10 10),(20 20,20 70,30 70,30 80,80 80,80 30,70 30,70 20,20 20)),((30 30,70 30,70 70,30 70,30 30)))",
    );
}

#[test]
fn gdal_self_crossing_shell() {
    assert_topo_eq_prepair_polygon(
        "gdal_self_crossing_shell",
        wkt!(POLYGON((10. 70., 90. 70., 90. 50., 30. 50., 30. 30., 50. 30., 50. 90., 70. 90., 70. 10., 10. 10., 10. 70.))),
        "MULTIPOLYGON (((10 10,70 10,70 50,50 50,50 70,10 70,10 10),(30 30,30 50,50 50,50 30,30 30)),((70 50,90 50,90 70,70 70,70 50)),((50 70,70 70,70 90,50 90,50 70)))",
    );
}

#[test]
fn gdal_self_overlapping_shell() {
    assert_topo_eq_prepair_polygon(
        "gdal_self_overlapping_shell",
        wkt!(POLYGON((10. 90., 50. 90., 50. 30., 70. 30., 70. 50., 30. 50., 30. 70., 90. 70., 90. 10., 10. 10., 10. 90.))),
        "POLYGON ((10 10,90 10,90 70,50 70,50 90,10 90,10 10),(50 30,50 50,70 50,70 30,50 30),(30 50,30 70,50 70,50 50,30 50))",
    );
}

#[test]
fn gdal_overlapping_multipolygons() {
    assert_topo_eq_prepair_multi(
        "gdal_overlapping_multi",
        wkt!(MULTIPOLYGON(
            ((10. 90., 60. 90., 60. 10., 10. 10., 10. 90.)),
            ((90. 80., 90. 20., 40. 20., 40. 80., 90. 80.))
        )),
        "MULTIPOLYGON (((10 10,60 10,60 20,40 20,40 80,60 80,60 90,10 90,10 10)),((60 20,90 20,90 80,60 80,60 20)))",
    );
}

#[test]
fn gdal_multiple_overlapping_multipolygons() {
    assert_topo_eq_prepair_multi(
        "gdal_multiple_overlapping_multi",
        wkt!(MULTIPOLYGON(
            ((90. 90., 90. 30., 30. 30., 30. 90., 90. 90.)),
            ((20. 20., 20. 80., 80. 80., 80. 20., 20. 20.)),
            ((10. 10., 10. 70., 70. 70., 70. 10., 10. 10.))
        )),
        "MULTIPOLYGON (((70 20,80 20,80 30,70 30,70 20)),((30 80,80 80,80 30,90 30,90 90,30 90,30 80)),((20 70,30 70,30 80,20 80,20 70)),((30 30,70 30,70 70,30 70,30 30)),((10 10,70 10,70 20,20 20,20 70,10 70,10 10)))",
    );
}

// ====== Other end-to-end cases ======

#[test]
fn self_intersecting_quad() {
    let crossed: Polygon<f64> = wkt!(POLYGON((0. 0., 10. 10., 10. 0., 0. 10., 0. 0.)));
    let repaired = crossed.make_valid().unwrap();
    assert_valid(&repaired);
    assert_eq!(repaired.0.len(), 2);
}

#[test]
fn repair_is_idempotent() {
    let bowtie: Polygon<f64> = wkt!(POLYGON((0. 0., 2. 2., 2. 0., 0. 2., 0. 0.)));
    let first = bowtie.make_valid().unwrap();
    let second = first.make_valid().unwrap();
    assert_valid(&second);
    assert_relative_eq!(total_area(&first), total_area(&second));
}

// ====== Pinch-point ring splitting (unit tests) ======

#[test]
fn split_no_pinch_points() {
    let ring = vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 1.0, y: 0.0 },
        Coord { x: 1.0, y: 1.0 },
        Coord { x: 0.0, y: 0.0 },
    ];
    let result = split_ring_at_pinch_points(ring);
    assert_eq!(result.len(), 1, "no pinch points -> one ring");
}

#[test]
fn split_simple_pinch_point() {
    let a = Coord { x: 5.0, y: 0.0 };
    let ring = vec![
        a,
        Coord { x: 8.0, y: 3.0 },
        Coord { x: 2.0, y: 3.0 },
        a,
        Coord { x: 10.0, y: 0.0 },
        Coord { x: 10.0, y: 10.0 },
        a,
    ];
    let result = split_ring_at_pinch_points(ring);
    assert!(
        result.len() >= 2,
        "pinch at shared vertex -> at least 2 rings, got {}",
        result.len()
    );
    for r in &result {
        assert_eq!(r.first(), r.last(), "ring should be closed");
        assert!(
            r.len() >= 4,
            "ring should have at least 3 distinct vertices"
        );
    }
}

#[test]
fn split_degenerate_sub_ring_dropped() {
    let a = Coord { x: 0.0, y: 0.0 };
    let ring = vec![
        a,
        Coord { x: 5.0, y: 0.0 },
        a,
        Coord { x: 10.0, y: 0.0 },
        Coord { x: 10.0, y: 10.0 },
        Coord { x: 0.0, y: 10.0 },
        a,
    ];
    let result = split_ring_at_pinch_points(ring);
    for r in &result {
        assert!(r.len() >= 4, "degenerate sub-ring should be dropped");
    }
}

// ====== Flood-fill face labelling (unit tests) ======

#[test]
fn label_faces_simple_triangle() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 4.0, y: 0.0 }),
        Line::new(Coord { x: 4.0, y: 0.0 }, Coord { x: 2.0, y: 3.0 }),
        Line::new(Coord { x: 2.0, y: 3.0 }, Coord { x: 0.0, y: 0.0 }),
    ];
    let cdt = build_cdt(lines, 0.0001).unwrap();
    let interior = label_faces(&cdt);
    assert_eq!(interior.len(), 1, "triangle should have 1 interior face");
}

#[test]
fn label_faces_square_with_hole() {
    let outer = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
        Line::new(Coord { x: 10.0, y: 0.0 }, Coord { x: 10.0, y: 10.0 }),
        Line::new(Coord { x: 10.0, y: 10.0 }, Coord { x: 0.0, y: 10.0 }),
        Line::new(Coord { x: 0.0, y: 10.0 }, Coord { x: 0.0, y: 0.0 }),
    ];
    let inner = vec![
        Line::new(Coord { x: 3.0, y: 3.0 }, Coord { x: 7.0, y: 3.0 }),
        Line::new(Coord { x: 7.0, y: 3.0 }, Coord { x: 7.0, y: 7.0 }),
        Line::new(Coord { x: 7.0, y: 7.0 }, Coord { x: 3.0, y: 7.0 }),
        Line::new(Coord { x: 3.0, y: 7.0 }, Coord { x: 3.0, y: 3.0 }),
    ];
    let lines: Vec<Line<f64>> = outer.into_iter().chain(inner).collect();
    let cdt = build_cdt(lines, 0.0001).unwrap();
    let interior = label_faces(&cdt);

    let interior_area: f64 = cdt
        .inner_faces()
        .filter(|f| interior.contains(&f.fix()))
        .map(|f| {
            let [a, b, c] = f.positions();
            let cross = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
            f64::abs(cross) / 2.0
        })
        .sum();
    assert_relative_eq!(interior_area, 84.0);
}

#[test]
fn label_faces_bowtie() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 2.0, y: 2.0 }),
        Line::new(Coord { x: 2.0, y: 2.0 }, Coord { x: 2.0, y: 0.0 }),
        Line::new(Coord { x: 2.0, y: 0.0 }, Coord { x: 0.0, y: 2.0 }),
        Line::new(Coord { x: 0.0, y: 2.0 }, Coord { x: 0.0, y: 0.0 }),
    ];
    let cdt = build_cdt(lines, 0.0001).unwrap();
    let interior = label_faces(&cdt);

    let interior_area: f64 = cdt
        .inner_faces()
        .filter(|f| interior.contains(&f.fix()))
        .map(|f| {
            let [a, b, c] = f.positions();
            let cross = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
            f64::abs(cross) / 2.0
        })
        .sum();
    assert_relative_eq!(interior_area, 2.0);
}

// ====== Line preparation: sweep + odd-even (unit tests) ======

#[test]
fn sweep_no_intersections() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
        Line::new(Coord { x: 0.0, y: 5.0 }, Coord { x: 10.0, y: 5.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 2);
}

#[test]
fn sweep_single_crossing() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 10.0 }),
        Line::new(Coord { x: 0.0, y: 10.0 }, Coord { x: 10.0, y: 0.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 4, "two crossing lines -> 4 sub-segments");
}

#[test]
fn sweep_multiple_splits_on_one_segment() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 5.0 }, Coord { x: 10.0, y: 5.0 }),
        Line::new(Coord { x: 3.0, y: 0.0 }, Coord { x: 3.0, y: 10.0 }),
        Line::new(Coord { x: 7.0, y: 0.0 }, Coord { x: 7.0, y: 10.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 7);
}

#[test]
fn sweep_split_points_sorted_along_diagonal() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 10.0 }),
        Line::new(Coord { x: 0.0, y: 3.0 }, Coord { x: 10.0, y: 3.0 }),
        Line::new(Coord { x: 0.0, y: 7.0 }, Coord { x: 10.0, y: 7.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 7);
    for line in &result {
        assert_ne!(line.start, line.end, "sub-segment should not be degenerate");
    }
}

#[test]
fn sweep_collinear_overlap() {
    // After splitting: [0,4], [4,6], [4,6], [6,10]
    // Odd-even: [4,6] appears twice (even) -> cancelled
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 6.0, y: 0.0 }),
        Line::new(Coord { x: 4.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(
        result.len(),
        2,
        "overlapping sub-segment cancels under odd-even"
    );
}

#[test]
fn sweep_odd_even_identical_cancel() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 0, "even count should cancel");
}

#[test]
fn sweep_odd_even_reversed_cancel() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
        Line::new(Coord { x: 10.0, y: 0.0 }, Coord { x: 0.0, y: 0.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 0, "reversed pair should cancel");
}

#[test]
fn sweep_odd_even_triple_keeps_one() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 1, "odd count should keep one");
}

#[test]
fn sweep_degenerate_filtered() {
    let lines = vec![
        Line::new(Coord { x: 5.0, y: 5.0 }, Coord { x: 5.0, y: 5.0 }),
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 10.0, y: 0.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(result.len(), 1);
}

#[test]
fn sweep_endpoint_touch_not_split() {
    let lines = vec![
        Line::new(Coord { x: 0.0, y: 0.0 }, Coord { x: 5.0, y: 5.0 }),
        Line::new(Coord { x: 5.0, y: 5.0 }, Coord { x: 10.0, y: 0.0 }),
    ];
    let result = prepare_lines_for_repair(lines);
    assert_eq!(
        result.len(),
        2,
        "shared endpoint should not cause splitting"
    );
}
