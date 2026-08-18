//! End-to-end tests for the RelateNG engine, ported from the JTS test
//! suites (master, ab57bff):
//! `RelateNGTest`, `RelateNGRobustnessTest`, `RelateNGGCTest`, and the
//! Mod-2 cases of `RelateNGBoundaryNodeRuleTest`.
//!
//! Fixtures are the JTS WKT strings verbatim, parsed at runtime. geo
//! cannot represent an empty Point, so `POINT EMPTY` fixtures are omitted
//! where they occur, with a comment.

use wkt::TryFromWkt;

use crate::Geometry;
use crate::geometry_cow::GeometryCow;
use crate::relate::IntersectionMatrix;

use super::im_predicate::RelateMatrixPredicate;
use super::relate_ng::{self, RelateNG};
use super::relate_predicate as pred;
use super::relate_predicate::intersection_matrix_pattern;
use super::topology_predicate::TopologyPredicate;

fn read(wkt_str: &str) -> Geometry<f64> {
    Geometry::try_from_wkt_str(wkt_str)
        .unwrap_or_else(|e| panic!("invalid WKT fixture {wkt_str}: {e}"))
}

fn check_pred(mut predicate: impl TopologyPredicate<f64>, wkta: &str, wktb: &str, expected: bool) {
    let a = read(wkta);
    let b = read(wktb);
    let ca = GeometryCow::from(&a);
    let cb = GeometryCow::from(&b);
    let actual = relate_ng::eval(&ca, &cb, &mut predicate);
    assert_eq!(actual, expected, "{predicate:?}: A = {wkta}, B = {wktb}");
}

fn check_intersects_disjoint(wkta: &str, wktb: &str, expected: bool) {
    check_pred(pred::intersects(), wkta, wktb, expected);
    check_pred(pred::intersects(), wktb, wkta, expected);
    check_pred(pred::disjoint(), wkta, wktb, !expected);
    check_pred(pred::disjoint(), wktb, wkta, !expected);
}

fn check_contains_within(wkta: &str, wktb: &str, expected: bool) {
    check_pred(pred::contains(), wkta, wktb, expected);
    check_pred(pred::within(), wktb, wkta, expected);
}

fn check_covers_covered_by(wkta: &str, wktb: &str, expected: bool) {
    check_pred(pred::covers(), wkta, wktb, expected);
    check_pred(pred::covered_by(), wktb, wkta, expected);
}

fn check_crosses(wkta: &str, wktb: &str, expected: bool) {
    check_pred(pred::crosses(), wkta, wktb, expected);
    check_pred(pred::crosses(), wktb, wkta, expected);
}

fn check_overlaps(wkta: &str, wktb: &str, expected: bool) {
    check_pred(pred::overlaps(), wkta, wktb, expected);
    check_pred(pred::overlaps(), wktb, wkta, expected);
}

fn check_touches(wkta: &str, wktb: &str, expected: bool) {
    check_pred(pred::touches(), wkta, wktb, expected);
    check_pred(pred::touches(), wktb, wkta, expected);
}

fn check_equals(wkta: &str, wktb: &str, expected: bool) {
    check_pred(pred::equals_topo(), wkta, wktb, expected);
    check_pred(pred::equals_topo(), wktb, wkta, expected);
}

fn relate_matrix(wkta: &str, wktb: &str) -> IntersectionMatrix {
    let a = read(wkta);
    let b = read(wktb);
    let ca = GeometryCow::from(&a);
    let cb = GeometryCow::from(&b);
    relate_ng::relate(&ca, &cb)
}

fn check_relate(wkta: &str, wktb: &str, expected: &str) {
    let im = relate_matrix(wkta, wktb);
    let expected: IntersectionMatrix = expected.parse().expect("valid DE-9IM matrix");
    assert_eq!(im, expected, "A = {wkta}, B = {wktb}");
}

fn check_relate_matches(wkta: &str, wktb: &str, pattern: &str, expected: bool) {
    check_pred(
        pred::matches(pattern).expect("valid pattern"),
        wkta,
        wktb,
        expected,
    );
}

fn check_prepared(wkta: &str, wktb: &str) {
    let a = read(wkta);
    let b = read(wktb);
    let ca = GeometryCow::from(&a);
    let cb = GeometryCow::from(&b);
    let prep_a = RelateNG::prepare(&ca);

    macro_rules! check {
        ($factory:expr, $name:literal) => {
            let mut p1 = $factory;
            let mut p2 = $factory;
            assert_eq!(
                prep_a.evaluate(&cb, &mut p1),
                relate_ng::eval(&ca, &cb, &mut p2),
                concat!($name, ": A = {}, B = {}"),
                wkta,
                wktb
            );
        };
    }
    check!(pred::equals_topo(), "equalsTopo");
    check!(pred::intersects(), "intersects");
    check!(pred::disjoint(), "disjoint");
    check!(pred::covers(), "covers");
    check!(pred::covered_by(), "coveredBy");
    check!(pred::within(), "within");
    check!(pred::contains(), "contains");
    check!(pred::crosses(), "crosses");
    check!(pred::touches(), "touches");

    let mut matrix_pred = RelateMatrixPredicate::new();
    prep_a.evaluate(&cb, &mut matrix_pred);
    let prepared_im = matrix_pred.into_im();
    assert_eq!(
        prepared_im,
        relate_ng::relate(&ca, &cb),
        "relate: A = {wkta}, B = {wktb}"
    );
}

fn check_prepared_matches(wkta: &str, wktb: &str, pattern: &str) {
    let a = read(wkta);
    let b = read(wktb);
    let ca = GeometryCow::from(&a);
    let cb = GeometryCow::from(&b);
    let prep_a = RelateNG::prepare(&ca);

    let mut p1 = pred::matches(pattern).expect("valid pattern");
    let mut p2 = pred::matches(pattern).expect("valid pattern");
    assert_eq!(
        prep_a.evaluate(&cb, &mut p1),
        relate_ng::eval(&ca, &cb, &mut p2),
        "matches {pattern}: A = {wkta}, B = {wktb}"
    );
}

// Tests ported from JTS RelateNGTest.java.
mod relate_ng_test {
    use super::*;

    #[test]
    fn test_points_disjoint() {
        let a = "POINT (0 0)";
        let b = "POINT (1 1)";
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
        check_equals(a, b, false);
        check_relate(a, b, "FF0FFF0F2");
    }

    //======= P/P  =============

    #[test]
    fn test_points_contained() {
        let a = "MULTIPOINT (0 0, 1 1, 2 2)";
        let b = "MULTIPOINT (1 1, 2 2)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_equals(a, b, false);
        check_relate(a, b, "0F0FFFFF2");
    }

    #[test]
    fn test_points_equal() {
        let a = "MULTIPOINT (0 0, 1 1, 2 2)";
        let b = "MULTIPOINT (0 0, 1 1, 2 2)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_equals(a, b, true);
    }

    #[test]
    fn test_validate_relate_pp_13() {
        let a = "MULTIPOINT ((80 70), (140 120), (20 20), (200 170))";
        let b = "MULTIPOINT ((80 70), (140 120), (80 170), (200 80))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_contains_within(b, a, false);
        check_covers_covered_by(a, b, false);
        check_overlaps(a, b, true);
        check_touches(a, b, false);
    }

    //======= L/P  =============

    #[test]
    fn test_line_point_contains() {
        let a = "LINESTRING (0 0, 1 1, 2 2)";
        let b = "MULTIPOINT (0 0, 1 1, 2 2)";
        check_relate(a, b, "0F10FFFF2");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_contains_within(b, a, false);
        check_covers_covered_by(a, b, true);
        check_covers_covered_by(b, a, false);
    }

    #[test]
    fn test_line_point_overlaps() {
        let a = "LINESTRING (0 0, 1 1)";
        let b = "MULTIPOINT (0 0, 1 1, 2 2)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_contains_within(b, a, false);
        check_covers_covered_by(a, b, false);
        check_covers_covered_by(b, a, false);
    }

    #[test]
    fn test_zero_length_line_point() {
        let a = "LINESTRING (0 0, 0 0)";
        let b = "POINT (0 0)";
        check_relate(a, b, "0FFFFFFF2");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_contains_within(b, a, true);
        check_covers_covered_by(a, b, true);
        check_covers_covered_by(b, a, true);
        check_equals(a, b, true);
    }

    #[test]
    fn test_zero_length_line_line() {
        let a = "LINESTRING (10 10, 10 10, 10 10)";
        let b = "LINESTRING (10 10, 10 10)";
        check_relate(a, b, "0FFFFFFF2");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_contains_within(b, a, true);
        check_covers_covered_by(a, b, true);
        check_covers_covered_by(b, a, true);
        check_equals(a, b, true);
    }

    // Tests a bug involving checking for non-zero-length lines.
    #[test]
    fn test_non_zero_length_line_point() {
        let a = "LINESTRING (0 0, 0 0, 9 9)";
        let b = "POINT (1 1)";
        check_relate(a, b, "0F1FF0FF2");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_contains_within(b, a, false);
        check_covers_covered_by(a, b, true);
        check_covers_covered_by(b, a, false);
        check_equals(a, b, false);
    }

    #[test]
    fn test_line_point_int_and_ext() {
        let a = "MULTIPOINT((60 60), (100 100))";
        let b = "LINESTRING(40 40, 80 80)";
        check_relate(a, b, "0F0FFF102");
    }

    //======= L/L  =============

    #[test]
    fn test_lines_cross_proper() {
        let a = "LINESTRING (0 0, 9 9)";
        let b = "LINESTRING(0 9, 9 0)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
    }

    #[test]
    fn test_lines_overlap() {
        let a = "LINESTRING (0 0, 5 5)";
        let b = "LINESTRING(3 3, 9 9)";
        check_intersects_disjoint(a, b, true);
        check_touches(a, b, false);
        check_overlaps(a, b, true);
    }

    #[test]
    fn test_lines_cross_vertex() {
        let a = "LINESTRING (0 0, 8 8)";
        let b = "LINESTRING(0 8, 4 4, 8 0)";
        check_intersects_disjoint(a, b, true);
    }

    #[test]
    fn test_lines_touch_vertex() {
        let a = "LINESTRING (0 0, 8 0)";
        let b = "LINESTRING(0 8, 4 0, 8 8)";
        check_intersects_disjoint(a, b, true);
    }

    #[test]
    fn test_lines_disjoint_by_envelope() {
        let a = "LINESTRING (0 0, 9 9)";
        let b = "LINESTRING(10 19, 19 10)";
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
    }

    #[test]
    fn test_lines_disjoint() {
        let a = "LINESTRING (0 0, 9 9)";
        let b = "LINESTRING (4 2, 8 6)";
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
    }

    #[test]
    fn test_lines_closed_empty() {
        let a = "MULTILINESTRING ((0 0, 0 1), (0 1, 1 1, 1 0, 0 0))";
        let b = "LINESTRING EMPTY";
        check_relate(a, b, "FF1FFFFF2");
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
    }

    #[test]
    fn test_lines_ring_touch_at_node() {
        let a = "LINESTRING (5 5, 1 8, 1 1, 5 5)";
        let b = "LINESTRING (5 5, 9 5)";
        check_relate(a, b, "F01FFF102");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_touches(a, b, true);
    }

    #[test]
    fn test_lines_touch_at_bdy() {
        let a = "LINESTRING (5 5, 1 8)";
        let b = "LINESTRING (5 5, 9 5)";
        check_relate(a, b, "FF1F00102");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_touches(a, b, true);
    }

    #[test]
    fn test_lines_overlap_with_disjoint_line() {
        let a = "LINESTRING (1 1, 9 9)";
        let b = "MULTILINESTRING ((2 2, 8 8), (6 2, 8 4))";
        check_relate(a, b, "101FF0102");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_overlaps(a, b, true);
    }

    #[test]
    fn test_lines_disjoint_overlapping_envelopes() {
        let a = "LINESTRING (60 0, 20 80, 100 80, 80 120, 40 140)";
        let b = "LINESTRING (60 40, 140 40, 140 160, 0 160)";
        check_relate(a, b, "FF1FF0102");
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
        check_touches(a, b, false);
    }

    /// Case from https://github.com/locationtech/jts/issues/270.
    /// Strictly, the lines cross, since their interiors intersect
    /// according to the orientation predicate; but the computation of the
    /// intersection point is non-robust and reports it as equal to the
    /// endpoint POINT (-10 0.0000000000000012). For consistency the
    /// relate algorithm uses the intersection node topology.
    #[test]
    fn test_lines_cross_jts270() {
        let a = "LINESTRING (0 0, -10 0.0000000000000012)";
        let b = "LINESTRING (-9.999143275740073 -0.1308959557133398, -10 0.0000000000001054)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_crosses(a, b, false);
        check_overlaps(a, b, false);
        check_touches(a, b, true);
    }

    #[test]
    fn test_lines_contained_jts396() {
        let a = "LINESTRING (1 0, 0 2, 0 0, 2 2)";
        let b = "LINESTRING (0 0, 2 2)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_crosses(a, b, false);
        check_overlaps(a, b, false);
        check_touches(a, b, false);
    }

    /// This case shows that lines must be self-noded, so that node
    /// topology is constructed correctly (at least for some predicates).
    #[test]
    fn test_lines_contained_with_self_intersection() {
        let a = "LINESTRING (2 0, 0 2, 0 0, 2 2)";
        let b = "LINESTRING (0 0, 2 2)";
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_crosses(a, b, false);
        check_overlaps(a, b, false);
        check_touches(a, b, false);
    }

    #[test]
    fn test_line_contained_in_ring() {
        let a = "LINESTRING(60 60, 100 100, 140 60)";
        let b = "LINESTRING(100 100, 180 20, 20 20, 100 100)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(b, a, true);
        check_covers_covered_by(b, a, true);
        check_crosses(a, b, false);
        check_overlaps(a, b, false);
        check_touches(a, b, false);
    }

    // See https://github.com/libgeos/geos/issues/933
    #[test]
    fn test_line_line_proper_intersection() {
        let a = "MULTILINESTRING ((0 0, 1 1), (0.5 0.5, 1 0.1, -1 0.1))";
        let b = "LINESTRING (0 0, 1 1)";
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_crosses(a, b, false);
        check_overlaps(a, b, false);
        check_touches(a, b, false);
    }

    #[test]
    fn test_line_self_intersection_collinear() {
        let a = "LINESTRING (9 6, 1 6, 1 0, 5 6, 9 6)";
        let b = "LINESTRING (9 9, 3 1)";
        check_relate(a, b, "0F1FFF102");
    }

    //======= A/P  =============

    #[test]
    fn test_polygon_point_inside() {
        let a = "POLYGON ((0 10, 10 10, 10 0, 0 0, 0 10))";
        let b = "POINT (1 1)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
    }

    #[test]
    fn test_polygon_point_outside() {
        let a = "POLYGON ((10 0, 0 0, 0 10, 10 0))";
        let b = "POINT (8 8)";
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
    }

    #[test]
    fn test_polygon_point_in_boundary() {
        let a = "POLYGON ((10 0, 0 0, 0 10, 10 0))";
        let b = "POINT (1 0)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, true);
    }

    #[test]
    fn test_area_point_in_exterior() {
        let a = "POLYGON ((1 5, 5 5, 5 1, 1 1, 1 5))";
        let b = "POINT (7 7)";
        check_relate(a, b, "FF2FF10F2");
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_touches(a, b, false);
        check_overlaps(a, b, false);
    }

    //======= A/L  =============

    #[test]
    fn test_area_line_contained_at_line_vertex() {
        let a = "POLYGON ((1 5, 5 5, 5 1, 1 1, 1 5))";
        let b = "LINESTRING (2 3, 3 5, 4 3)";
        check_intersects_disjoint(a, b, true);
        check_touches(a, b, false);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_area_line_touch_at_line_vertex() {
        let a = "POLYGON ((1 5, 5 5, 5 1, 1 1, 1 5))";
        let b = "LINESTRING (1 8, 3 5, 5 8)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_touches(a, b, true);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_polygon_line_inside() {
        let a = "POLYGON ((0 10, 10 10, 10 0, 0 0, 0 10))";
        let b = "LINESTRING (1 8, 3 5, 5 8)";
        check_relate(a, b, "102FF1FF2");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
    }

    #[test]
    fn test_polygon_line_outside() {
        let a = "POLYGON ((10 0, 0 0, 0 10, 10 0))";
        let b = "LINESTRING (4 8, 9 3)";
        check_intersects_disjoint(a, b, false);
        check_contains_within(a, b, false);
    }

    #[test]
    fn test_polygon_line_in_boundary() {
        let a = "POLYGON ((10 0, 0 0, 0 10, 10 0))";
        let b = "LINESTRING (1 0, 9 0)";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, true);
        check_touches(a, b, true);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_polygon_line_crossing_contained() {
        let a =
            "MULTIPOLYGON (((20 80, 180 80, 100 0, 20 80)), ((20 160, 180 160, 100 80, 20 160)))";
        let b = "LINESTRING (100 140, 100 40)";
        check_relate(a, b, "1020F1FF2");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_touches(a, b, false);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_validate_relate_la_220() {
        let a = "LINESTRING (90 210, 210 90)";
        let b = "POLYGON ((150 150, 410 150, 280 20, 20 20, 150 150))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_touches(a, b, false);
        check_overlaps(a, b, false);
    }

    /// See RelateLA.xml (line 585).
    #[test]
    fn test_line_crossing_polygon_at_shell_hole_point() {
        let a = "LINESTRING (60 160, 150 70)";
        let b = "POLYGON ((190 190, 360 20, 20 20, 190 190), (110 110, 250 100, 140 30, 110 110))";
        check_relate(a, b, "F01FF0212");
        check_touches(a, b, true);
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_touches(a, b, true);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_line_crossing_polygon_at_non_vertex() {
        let a = "LINESTRING (20 60, 150 60)";
        let b = "POLYGON ((150 150, 410 150, 280 20, 20 20, 150 150))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_touches(a, b, false);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_polygon_lines_contained_collinear_edge() {
        let a = "POLYGON ((110 110, 200 20, 20 20, 110 110))";
        let b =
            "MULTILINESTRING ((110 110, 60 40, 70 20, 150 20, 170 40), (180 30, 40 30, 110 80))";
        check_relate(a, b, "102101FF2");
    }

    //======= A/A  =============

    #[test]
    fn test_polygons_edge_adjacent() {
        let a = "POLYGON ((1 3, 3 3, 3 1, 1 1, 1 3))";
        let b = "POLYGON ((5 3, 5 1, 3 1, 3 3, 5 3))";
        check_overlaps(a, b, false);
        check_touches(a, b, true);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_polygons_edge_adjacent2() {
        let a = "POLYGON ((1 3, 4 3, 3 0, 1 1, 1 3))";
        let b = "POLYGON ((5 3, 5 1, 3 0, 4 3, 5 3))";
        check_overlaps(a, b, false);
        check_touches(a, b, true);
        check_overlaps(a, b, false);
    }

    #[test]
    fn test_polygons_nested() {
        let a = "POLYGON ((1 9, 9 9, 9 1, 1 1, 1 9))";
        let b = "POLYGON ((2 8, 8 8, 8 2, 2 2, 2 8))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_overlaps(a, b, false);
        check_touches(a, b, false);
    }

    #[test]
    fn test_polygons_overlap_proper() {
        let a = "POLYGON ((1 1, 1 7, 7 7, 7 1, 1 1))";
        let b = "POLYGON ((2 8, 8 8, 8 2, 2 2, 2 8))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_overlaps(a, b, true);
        check_touches(a, b, false);
    }

    #[test]
    fn test_polygons_overlap_at_nodes() {
        let a = "POLYGON ((1 5, 5 5, 5 1, 1 1, 1 5))";
        let b = "POLYGON ((7 3, 5 1, 3 3, 5 5, 7 3))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_overlaps(a, b, true);
        check_touches(a, b, false);
    }

    #[test]
    fn test_polygons_contained_at_nodes() {
        let a = "POLYGON ((1 5, 5 5, 6 2, 1 1, 1 5))";
        let b = "POLYGON ((1 1, 5 5, 6 2, 1 1))";
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_overlaps(a, b, false);
        check_touches(a, b, false);
    }

    #[test]
    fn test_polygons_nested_with_hole() {
        let a = "POLYGON ((40 60, 420 60, 420 320, 40 320, 40 60), (200 140, 160 220, 260 200, 200 140))";
        let b = "POLYGON ((80 100, 360 100, 360 280, 80 280, 80 100))";
        check_contains_within(a, b, false);
        check_contains_within(b, a, false);
        check_pred(pred::contains(), a, b, false);
    }

    #[test]
    fn test_polygons_overlapping_with_boundary_inside() {
        let a = "POLYGON ((100 60, 140 100, 100 140, 60 100, 100 60))";
        let b = "MULTIPOLYGON (((80 40, 120 40, 120 80, 80 80, 80 40)), ((120 80, 160 80, 160 120, 120 120, 120 80)), ((80 120, 120 120, 120 160, 80 160, 80 120)), ((40 80, 80 80, 80 120, 40 120, 40 80)))";
        check_relate(a, b, "21210F212");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_contains_within(b, a, false);
        check_covers_covered_by(a, b, false);
        check_overlaps(a, b, true);
        check_touches(a, b, false);
    }

    #[test]
    fn test_polygons_overlap_very_narrow() {
        let a = "POLYGON ((120 100, 120 200, 200 200, 200 100, 120 100))";
        let b = "POLYGON ((100 100, 100000 110, 100000 100, 100 100))";
        check_relate(a, b, "212111212");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_contains_within(b, a, false);
    }

    #[test]
    fn test_validate_relate_aa_86() {
        let a = "POLYGON ((170 120, 300 120, 250 70, 120 70, 170 120))";
        let b = "POLYGON ((150 150, 410 150, 280 20, 20 20, 150 150), (170 120, 330 120, 260 50, 100 50, 170 120))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_overlaps(a, b, false);
        check_pred(pred::within(), a, b, false);
        check_touches(a, b, true);
    }

    #[test]
    fn test_validate_relate_aa_97() {
        let a = "POLYGON ((330 150, 200 110, 150 150, 280 190, 330 150))";
        let b = "MULTIPOLYGON (((140 110, 260 110, 170 20, 50 20, 140 110)), ((300 270, 420 270, 340 190, 220 190, 300 270)))";
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_overlaps(a, b, false);
        check_pred(pred::within(), a, b, false);
        check_touches(a, b, true);
    }

    #[test]
    fn test_adjacent_polygons() {
        let a = "POLYGON ((1 9, 6 9, 6 1, 1 1, 1 9))";
        let b = "POLYGON ((9 9, 9 4, 6 4, 6 9, 9 9))";
        check_relate_matches(a, b, intersection_matrix_pattern::ADJACENT, true);
    }

    #[test]
    fn test_adjacent_polygons_touching_at_point() {
        let a = "POLYGON ((1 9, 6 9, 6 1, 1 1, 1 9))";
        let b = "POLYGON ((9 9, 9 4, 6 4, 7 9, 9 9))";
        check_relate_matches(a, b, intersection_matrix_pattern::ADJACENT, false);
    }

    #[test]
    fn test_adjacent_polygons_overlappping() {
        let a = "POLYGON ((1 9, 6 9, 6 1, 1 1, 1 9))";
        let b = "POLYGON ((9 9, 9 4, 6 4, 5 9, 9 9))";
        check_relate_matches(a, b, intersection_matrix_pattern::ADJACENT, false);
    }

    #[test]
    fn test_contains_properly_polygon_contained() {
        let a = "POLYGON ((1 9, 9 9, 9 1, 1 1, 1 9))";
        let b = "POLYGON ((2 8, 5 8, 5 5, 2 5, 2 8))";
        check_relate_matches(a, b, intersection_matrix_pattern::CONTAINS_PROPERLY, true);
    }

    #[test]
    fn test_contains_properly_polygon_touching() {
        let a = "POLYGON ((1 9, 9 9, 9 1, 1 1, 1 9))";
        let b = "POLYGON ((9 1, 5 1, 5 5, 9 5, 9 1))";
        check_relate_matches(a, b, intersection_matrix_pattern::CONTAINS_PROPERLY, false);
    }

    #[test]
    fn test_contains_properly_polygons_overlapping() {
        let a = "GEOMETRYCOLLECTION (POLYGON ((1 9, 6 9, 6 4, 1 4, 1 9)), POLYGON ((2 4, 6 7, 9 1, 2 4)))";
        let b = "POLYGON ((5 5, 6 5, 6 4, 5 4, 5 5))";
        check_relate_matches(a, b, intersection_matrix_pattern::CONTAINS_PROPERLY, true);
    }

    //================  Repeated Points  ==============

    #[test]
    fn test_repeated_point_ll() {
        let a = "LINESTRING(0 0, 5 5, 5 5, 5 5, 9 9)";
        let b = "LINESTRING(0 9, 5 5, 5 5, 5 5, 9 0)";
        check_relate(a, b, "0F1FF0102");
        check_intersects_disjoint(a, b, true);
    }

    #[test]
    fn test_repeated_point_aa() {
        let a = "POLYGON ((1 9, 9 7, 9 1, 1 3, 1 9))";
        let b = "POLYGON ((1 3, 1 3, 1 3, 3 7, 9 7, 9 7, 1 3))";
        check_relate(a, b, "212F01FF2");
    }

    //================  EMPTY geometries  ==============

    // geo cannot represent POINT EMPTY, so it is omitted from the JTS
    // list of empty fixtures.
    const EMPTIES: [&str; 6] = [
        "LINESTRING EMPTY",
        "POLYGON EMPTY",
        "MULTIPOINT EMPTY",
        "MULTILINESTRING EMPTY",
        "MULTIPOLYGON EMPTY",
        "GEOMETRYCOLLECTION EMPTY",
    ];

    #[test]
    fn test_empty_empty() {
        for a in EMPTIES {
            for b in EMPTIES {
                check_relate(a, b, "FFFFFFFF2");
                // Empty geometries are all topologically equal.
                check_equals(a, b, true);

                check_intersects_disjoint(a, b, false);
                check_contains_within(a, b, false);
            }
        }
    }

    #[test]
    fn test_empty_non_empty() {
        let non_empty_point = "POINT (1 1)";
        let non_empty_line = "LINESTRING (1 1, 2 2)";
        let non_empty_polygon = "POLYGON ((1 1, 1 2, 2 1, 1 1))";

        for empty in EMPTIES {
            check_relate(empty, non_empty_point, "FFFFFF0F2");
            check_relate(non_empty_point, empty, "FF0FFFFF2");

            check_relate(empty, non_empty_line, "FFFFFF102");
            check_relate(non_empty_line, empty, "FF1FF0FF2");

            check_relate(empty, non_empty_polygon, "FFFFFF212");
            check_relate(non_empty_polygon, empty, "FF2FF1FF2");

            check_equals(empty, non_empty_point, false);
            check_equals(empty, non_empty_line, false);
            check_equals(empty, non_empty_polygon, false);

            check_intersects_disjoint(empty, non_empty_point, false);
            check_intersects_disjoint(empty, non_empty_line, false);
            check_intersects_disjoint(empty, non_empty_polygon, false);

            check_contains_within(empty, non_empty_point, false);
            check_contains_within(empty, non_empty_line, false);
            check_contains_within(empty, non_empty_polygon, false);

            check_contains_within(non_empty_point, empty, false);
            check_contains_within(non_empty_line, empty, false);
            check_contains_within(non_empty_polygon, empty, false);
        }
    }

    //================  Prepared Relate  ==============

    #[test]
    fn test_prepared_aa() {
        let a = "POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))";
        let b = "POLYGON((0.5 0.5, 1.5 0.5, 1.5 1.5, 0.5 1.5, 0.5 0.5))";
        check_prepared(a, b);
    }

    #[test]
    fn test_prepared_pa() {
        let a = "POINT (5 5)";
        let b = "POLYGON ((1 9, 9 9, 9 1, 1 1, 1 9))";
        check_prepared(a, b);
        check_prepared(b, a);

        // See https://github.com/libgeos/geos/issues/1275 (not a bug, but
        // a good test to have). The transposed pattern is written out
        // directly.
        check_prepared_matches(a, b, "T*****FF*");
        check_prepared_matches(b, a, "T*F**F***");
    }
}
