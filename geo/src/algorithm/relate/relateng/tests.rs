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

// Tests ported from JTS RelateNGRobustnessTest.java: reported cases with
// robustness issues.
mod relate_ng_robustness_test {
    use super::*;

    //--------------------------------------------------------
    //  GeometryCollection semantics
    //--------------------------------------------------------

    // See https://github.com/libgeos/geos/issues/1033
    #[test]
    fn test_geos_1033() {
        check_contains_within(
            "POLYGON((1 0,0 4,2 2,1 0))",
            "GEOMETRYCOLLECTION(POINT(2 2),POINT(1 0),LINESTRING(1 2,1 1))",
            true,
        );
    }

    // https://github.com/libgeos/geos/issues/1027
    #[test]
    fn test_geos_1027() {
        check_covers_covered_by(
            "MULTIPOLYGON (((0 0, 3 0, 3 3, 0 3, 0 0)))",
            "GEOMETRYCOLLECTION ( LINESTRING (1 2, 1 1), POINT (0 0))",
            true,
        );
    }

    // https://github.com/libgeos/geos/issues/1022
    #[test]
    fn test_geos_1022() {
        check_crosses(
            "GEOMETRYCOLLECTION (POINT (7 1), LINESTRING (6 5, 6 4))",
            "POLYGON ((7 1, 1 3, 3 9, 7 1))",
            false,
        );
    }

    // https://github.com/libgeos/geos/issues/1011
    #[test]
    fn test_geos_1011() {
        let a = "LINESTRING(75 15,55 43)";
        let b = "GEOMETRYCOLLECTION(POLYGON EMPTY,LINESTRING(75 15,55 43))";
        check_covers_covered_by(a, b, true);
        check_equals(a, b, true);
    }

    // https://github.com/libgeos/geos/issues/983
    #[test]
    fn test_geos_983() {
        let a = "POINT(0 0)";
        let b = "GEOMETRYCOLLECTION(POINT (1 1), LINESTRING (1 1, 2 2))";
        check_intersects_disjoint(a, b, false);
    }

    // https://github.com/libgeos/geos/issues/982
    #[test]
    fn test_geos_982() {
        let a = "POINT(0 0)";
        let b1 = "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(0 0, 1 0))";
        check_contains_within(b1, a, false);
        check_covers_covered_by(b1, a, true);

        let b2 = "GEOMETRYCOLLECTION(LINESTRING(0 0, 1 0), POINT(0 0))";
        check_contains_within(b2, a, false);
        check_covers_covered_by(b2, a, true);
    }

    // https://github.com/libgeos/geos/issues/981
    #[test]
    fn test_geos_981() {
        let a = "POINT(0 0)";
        let b = "GEOMETRYCOLLECTION(LINESTRING(0 1, 0 0), POINT(0 0))";
        check_relate_matches(b, a, intersection_matrix_pattern::CONTAINS_PROPERLY, false);
    }

    //--------------------------------------------------------
    //  Noding robustness problems
    //--------------------------------------------------------

    // https://github.com/libgeos/geos/issues/1053
    #[test]
    fn test_geos_1053() {
        let a = "MULTILINESTRING((2 4, 10 10),(15 10,10 5,5 10))";
        let b = "MULTILINESTRING((2 4, 10 10))";
        check_relate(a, b, "1F1F00FF2");
    }

    // https://github.com/libgeos/geos/issues/968
    #[test]
    fn test_geos_968() {
        let a2 = "LINESTRING(10 0, 0 20)";
        let b2 = "POINT (9 2)";
        check_covers_covered_by(a2, b2, true);
    }

    // JTS xtestGEOS_968_2 is disabled upstream ("this case doesn't work
    // due to numeric rounding for Orientation test") and is not ported.

    // https://github.com/libgeos/geos/issues/933
    #[test]
    fn test_geos_933() {
        let a = "LINESTRING (0 0, 1 1)";
        let b = "LINESTRING (0.2 0.2, 0.5 0.5)";
        check_covers_covered_by(a, b, true);
    }

    // https://github.com/libgeos/geos/issues/740
    #[test]
    fn test_geos_740() {
        let a = "POLYGON ((1454700 -331500, 1455100 -330700, 1455466.6191038645 -331281.94727476506, 1455467.8182005754 -331293.26796732045, 1454700 -331500))";
        let b = "LINESTRING (1455389.376551584 -331255.3803222172, 1455467.2422460222 -331287.83037053316)";
        check_contains_within(a, b, false);
    }

    //--------------------------------------------------------
    //  Robustness failures (TopologyException in old code)
    //--------------------------------------------------------

    // https://github.com/libgeos/geos/issues/766
    #[test]
    fn test_geos_766() {
        let a = "POLYGON ((26639.240191093646 6039.3615818717535, 26639.240191093646 5889.361620883223, 28000.000095100608 5889.362081553552, 28000.000095100608 6039.361620882992, 28700.00019021402 6039.361620882992, 28700.00019021402 5889.361822800367, 29899.538842431968 5889.362160452064, 32465.59665091549 5889.362882757903, 32969.2837182586 -1313.697771558439, 31715.832811969216 -1489.87008918589, 31681.039836323587 -1242.3030298361555, 32279.3890331618 -1158.210534269224, 32237.63710287376 -861.1301136466199, 32682.89764107368 -802.0828534499739, 32247.445200905553 5439.292852892075, 31797.06861513178 5439.292852892075, 31797.06861513178 5639.36178850523, 29899.538849750803 5639.361268079038, 26167.69458275995 5639.3602445643955, 26379.03654594742 2617.0293071870683, 26778.062167926924 2644.9318977193907, 26792.01346261031 2445.419086759444, 26193.472956813417 2403.5650586598513, 25939.238114175267 6039.361685403233, 26639.240191093646 6039.3615818717535), (32682.89764107368 -802.0828534499738, 32682.89764107378 -802.0828534499669, 32247.445200905655 5439.292852892082, 32247.445200905553 5439.292852892075, 32682.89764107368 -802.0828534499738))";
        let b = "POLYGON ((32450.100392347143 5889.362314133216, 32050.104955691 5891.272957209961, 32100.021071878822 16341.272221116333, 32500.016508656867 16339.361578039587, 32450.100392347143 5889.362314133216))";
        check_intersects_disjoint(a, b, true);
    }

    // https://github.com/libgeos/geos/issues/1026
    #[test]
    fn test_geos_1026() {
        let a = "POLYGON((335645.7810000004 5677846.65,335648.6579999998 5677845.801999999,335650.8630842535 5677845.143617179,335650.77673334075 5677844.7250704905,335642.90299999993 5677847.498,335645.7810000004 5677846.65))";
        let b = "POLYGON((335642.903 5677847.498,335642.894 5677847.459,335645.92 5677846.69,335647.378 5677852.523,335644.403 5677853.285,335644.374 5677853.293,335642.903 5677847.498))";
        check_touches(a, b, false);
    }

    // https://github.com/locationtech/jts/issues/1051
    #[test]
    fn test_jts_1051() {
        let a = "POLYGON ((414188.5999999999 6422867.1, 414193.7 6422866.5, 414205.1 6422859.4, 414223.7 6422846.8, 414229.6 6422843.2, 414235.2 6422835.4, 414224.7 6422837.9, 414219.4 6422842.1, 414210.9 6422849, 414199.2 6422857.6, 414191.1 6422863.4, 414188.5999999999 6422867.1))";
        let b = "LINESTRING (414187.2 6422831.6, 414179 6422836.1, 414182.2 6422841.8, 414176.7 6422844, 414184.5 6422859.5, 414188.6 6422867.1)";
        check_intersects_disjoint(a, b, true);
    }

    // https://trac.osgeo.org/postgis/ticket/5362
    #[test]
    fn test_postgis_5362() {
        let a = "POLYGON ((-707259.66 -1121493.36, -707205.9 -1121605.808, -707310.5388 -1121540.5446, -707318.8200000001 -1121533.21, -707259.66 -1121493.36))";
        let b = "POLYGON ((-707356.18 -1121550.69, -707332.82 -1121536.63, -707318.82 -1121533.21, -707321.72 -1121535.08, -707327.4 -1121539.21, -707356.18 -1121550.69))";
        check_relate(a, b, "2F2101212");
        check_intersects_disjoint(a, b, true);
    }

    //--------------------------------------------------------
    //  Topological Inconsistency
    //--------------------------------------------------------

    // https://github.com/libgeos/geos/issues/1064
    #[test]
    fn test_geos_1064() {
        let a = "LINESTRING (16.330791631988802 68.75635661578073, 16.332533372319826 68.75496886016562)";
        let b =
            "LINESTRING (16.30641253121884 68.75189557630306, 16.33167771310482 68.75565061843871)";
        check_relate(a, b, "F01FF0102");
    }

    // https://github.com/locationtech/jts/issues/396
    #[test]
    fn test_jts_396() {
        let a = "LINESTRING (1 0, 0 2, 0 0, 2 2)";
        let b = "LINESTRING (0 0, 2 2)";
        check_relate(a, b, "101F00FF2");
        check_covers_covered_by(a, b, true);
    }

    // https://github.com/locationtech/jts/issues/270
    #[test]
    fn test_jts_270() {
        let a = "LINESTRING(0.0 0.0, -10.0 1.2246467991473533E-15)";
        let b = "LINESTRING(-9.999143275740073 -0.13089595571333978, -10.0 1.0535676356486768E-13)";
        check_relate(a, b, "FF10F0102");
        check_intersects_disjoint(a, b, true);
    }

    // https://gis.stackexchange.com/questions/484691
    #[test]
    fn test_gisse_484691() {
        let a = "POLYGON ((1.839012980156925 43.169860517728324, 1.838983490127865 43.169860200336274, 1.838898525601717 43.169868281549725, 1.838918565176068 43.1699719478626, 1.838920733577112 43.16998636433192, 1.838978629555589 43.16997979090823, 1.838982586839382 43.169966339940714, 1.838974943184281 43.169918580432174, 1.839020497362873 43.169914572864634, 1.839012980156925 43.169860517728324))";
        let b = "POLYGON ((1.8391355300979277 43.16987802887805, 1.83913336164737 43.16986361241434, 1.8390129801569248 43.169860517728324, 1.8390790978572837 43.16987292371998, 1.8390909520103162 43.16995581178317, 1.8391377530291442 43.16995091801345, 1.8391293863398452 43.16987796276235, 1.8391355300979277 43.16987802887805))";
        check_relate(a, b, "2F2101212");
        check_intersects_disjoint(a, b, true);
    }
}

// Tests ported from JTS RelateNGGCTest.java: GeometryCollection inputs
// with union semantics.
mod relate_ng_gc_test {
    use super::*;

    #[test]
    fn test_dimension_with_empty() {
        let a = "LINESTRING(0 0, 1 1)";
        let b = "GEOMETRYCOLLECTION(POLYGON EMPTY,LINESTRING(0 0, 1 1))";
        check_covers_covered_by(a, b, true);
        check_equals(a, b, true);
    }

    // See https://github.com/libgeos/geos/issues/1027
    #[test]
    fn test_mp_glp_geos1027() {
        let a = "MULTIPOLYGON (((0 0, 3 0, 3 3, 0 3, 0 0)))";
        let b = "GEOMETRYCOLLECTION ( LINESTRING (1 2, 1 1), POINT (0 0))";
        check_relate(a, b, "1020F1FF2");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, true);
        check_crosses(a, b, false);
        check_equals(a, b, false);
    }

    // See https://github.com/libgeos/geos/issues/1022
    #[test]
    fn test_gpl_a() {
        let a = "GEOMETRYCOLLECTION (POINT (7 1), LINESTRING (6 5, 6 4))";
        let b = "POLYGON ((7 1, 1 3, 3 9, 7 1))";
        check_relate(a, b, "F01FF0212");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_crosses(a, b, false);
        check_touches(a, b, true);
        check_equals(a, b, false);
    }

    // See https://github.com/libgeos/geos/issues/982
    #[test]
    fn test_p_gpl() {
        let a = "POINT(0 0)";
        let b = "GEOMETRYCOLLECTION(POINT(0 0), LINESTRING(0 0, 1 0))";
        check_relate(a, b, "F0FFFF102");
        check_intersects_disjoint(a, b, true);
        check_contains_within(a, b, false);
        check_crosses(a, b, false);
        check_touches(a, b, true);
        check_equals(a, b, false);
    }

    #[test]
    fn test_line_in_overlapping_polygons_touching_interior_edge() {
        let a = "LINESTRING (3 7, 7 3)";
        let b = "GEOMETRYCOLLECTION (POLYGON ((1 9, 7 9, 7 3, 1 3, 1 9)), POLYGON ((9 1, 3 1, 3 7, 9 7, 9 1)))";
        check_relate(a, b, "1FF0FF212");
        check_contains_within(b, a, true);
    }

    #[test]
    fn test_line_in_overlapping_polygons_crossing_interior_edge_at_vertex() {
        let a = "LINESTRING (2 2, 8 8)";
        let b = "GEOMETRYCOLLECTION (POLYGON ((1 1, 1 7, 7 7, 7 1, 1 1)), POLYGON ((9 9, 9 3, 3 3, 3 9, 9 9)))";
        check_relate(a, b, "1FF0FF212");
        check_contains_within(b, a, true);
    }

    #[test]
    fn test_line_in_overlapping_polygons_crossing_interior_edge_proper() {
        let a = "LINESTRING (2 4, 6 8)";
        let b = "GEOMETRYCOLLECTION (POLYGON ((1 1, 1 7, 7 7, 7 1, 1 1)), POLYGON ((9 9, 9 3, 3 3, 3 9, 9 9)))";
        check_relate(a, b, "1FF0FF212");
        check_contains_within(b, a, true);
    }

    #[test]
    fn test_polygon_in_overlapping_polygons_touching_boundaries() {
        let a = "GEOMETRYCOLLECTION (POLYGON ((1 9, 6 9, 6 4, 1 4, 1 9)), POLYGON ((9 1, 4 1, 4 6, 9 6, 9 1)) )";
        let b = "POLYGON ((2 6, 6 2, 8 4, 4 8, 2 6))";
        check_relate(a, b, "212F01FF2");
        check_contains_within(a, b, true);
    }

    #[test]
    fn test_line_in_overlapping_polygons_boundaries() {
        let a = "LINESTRING (1 6, 9 6, 9 1, 1 1, 1 6)";
        let b = "GEOMETRYCOLLECTION (POLYGON ((1 1, 1 6, 6 6, 6 1, 1 1)), POLYGON ((9 1, 4 1, 4 6, 9 6, 9 1)))";
        check_relate(a, b, "F1FFFF2F2");
        check_contains_within(a, b, false);
        check_covers_covered_by(a, b, false);
        check_covers_covered_by(b, a, true);
    }

    #[test]
    fn test_line_covers_overlapping_polygons_boundaries() {
        let a = "LINESTRING (1 6, 9 6, 9 1, 1 1, 1 6)";
        let b = "GEOMETRYCOLLECTION (POLYGON ((1 1, 1 6, 6 6, 6 1, 1 1)), POLYGON ((9 1, 4 1, 4 6, 9 6, 9 1)))";
        check_relate(a, b, "F1FFFF2F2");
        check_contains_within(b, a, false);
        check_covers_covered_by(b, a, true);
    }

    #[test]
    fn test_adjacent_polygons_contained_in_adjacent_polygons() {
        let a = "GEOMETRYCOLLECTION (POLYGON ((2 2, 2 5, 4 5, 4 2, 2 2)), POLYGON ((8 2, 4 3, 4 4, 8 5, 8 2)))";
        let b = "GEOMETRYCOLLECTION (POLYGON ((1 1, 1 6, 4 6, 4 1, 1 1)), POLYGON ((9 1, 4 1, 4 6, 9 6, 9 1)))";
        check_relate(a, b, "2FF1FF212");
        check_contains_within(b, a, true);
        check_covers_covered_by(b, a, true);
    }

    #[test]
    fn test_gc_multi_polygon_intersects_polygon() {
        let a = "POLYGON ((2 5, 3 5, 3 3, 2 3, 2 5))";
        let b = "GEOMETRYCOLLECTION (MULTIPOLYGON (((1 4, 4 4, 4 1, 1 1, 1 4)), ((5 4, 8 4, 8 1, 5 1, 5 4))))";
        check_relate(a, b, "212101212");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(b, a, false);
    }

    #[test]
    fn test_polygon_contains_gc_multi_polygon_element() {
        let a = "POLYGON ((0 5, 4 5, 4 1, 0 1, 0 5))";
        let b = "GEOMETRYCOLLECTION (MULTIPOLYGON (((1 4, 3 4, 3 2, 1 2, 1 4)), ((6 4, 8 4, 8 2, 6 2, 6 4))))";
        check_relate(a, b, "212FF1212");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(b, a, false);
    }

    /// Demonstrates the need for assigning computed nodes to their rings,
    /// so that subsequent point-in-polygon testing can report the node as
    /// being on the ring boundary.
    #[test]
    fn test_polygon_overlapping_gc_polygon() {
        let a = "GEOMETRYCOLLECTION (POLYGON ((18.6 40.8, 16.8825 39.618567, 16.9319 39.5461, 17.10985 39.485133, 16.6143 38.4302, 16.43145 38.313267, 16.2 37.5, 14.8 37.8, 14.96475 40.474933, 18.6 40.8)))";
        let b = "POLYGON ((16.3649953125 38.37219358064516, 16.3649953125 39.545924774193544, 17.949465625000002 39.545924774193544, 17.949465625000002 38.37219358064516, 16.3649953125 38.37219358064516))";
        check_relate(b, a, "212101212");
        check_relate(a, b, "212101212");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, false);
    }

    const WKT_ADJACENT_POLYS: &str = "GEOMETRYCOLLECTION (POLYGON ((5 5, 2 9, 9 9, 9 5, 5 5)), POLYGON ((3 1, 5 5, 9 5, 9 1, 3 1)), POLYGON ((1 9, 2 9, 5 5, 3 1, 1 1, 1 9)))";

    #[test]
    fn test_adj_polygons_cover_polygon_with_endpoint_inside() {
        let a = WKT_ADJACENT_POLYS;
        let b = "POLYGON ((3 7, 7 7, 7 3, 3 3, 3 7))";
        check_relate(b, a, "2FF1FF212");
        check_relate(a, b, "212FF1FF2");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, true);
    }

    #[test]
    fn test_adj_polygons_cover_point_at_node() {
        let a = WKT_ADJACENT_POLYS;
        let b = "POINT (5 5)";
        check_relate(b, a, "0FFFFF212");
        check_relate(a, b, "0F2FF1FF2");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, true);
    }

    #[test]
    fn test_adj_polygons_cover_point_on_edge() {
        let a = WKT_ADJACENT_POLYS;
        let b = "POINT (7 5)";
        check_relate(b, a, "0FFFFF212");
        check_relate(a, b, "0F2FF1FF2");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, true);
    }

    #[test]
    fn test_adj_polygons_containing_polygon_touching_interior_endpoint() {
        let a = WKT_ADJACENT_POLYS;
        let b = "POLYGON ((5 5, 7 5, 7 3, 5 3, 5 5))";
        check_relate(a, b, "212FF1FF2");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, true);
    }

    #[test]
    fn test_adj_polygons_overlapped_by_polygon_with_hole() {
        let a = WKT_ADJACENT_POLYS;
        let b = "POLYGON ((0 10, 10 10, 10 0, 0 0, 0 10), (2 8, 8 8, 8 2, 2 2, 2 8))";
        check_relate(a, b, "2121FF212");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, false);
    }

    #[test]
    fn test_adj_polygons_containing_line() {
        let a = WKT_ADJACENT_POLYS;
        let b = "LINESTRING (5 5, 7 7)";
        check_relate(a, b, "102FF1FF2");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, true);
    }

    #[test]
    fn test_adj_polygons_containing_line_and_point() {
        let a = WKT_ADJACENT_POLYS;
        let b = "GEOMETRYCOLLECTION (POINT (5 5), LINESTRING (5 7, 7 7))";
        check_relate(a, b, "102FF1FF2");
        check_intersects_disjoint(a, b, true);
        check_covers_covered_by(a, b, true);
    }

    // JTS testEmptyMultiPointElements uses MULTIPOINT (EMPTY, (5 5)),
    // which geo cannot represent (a MultiPoint cannot hold an empty
    // Point); the test is not ported.

    #[test]
    fn test_polygon_containing_points_in_boundary() {
        let a = "POLYGON ((0 0, 0 10, 10 10, 10 0, 0 0))";
        let b = "GEOMETRYCOLLECTION (POLYGON ((0 0, 10 0, 10 10, 0 10, 0 0)), MULTIPOINT ((0 2), (0 5)))";
        check_equals(a, b, true);
    }

    #[test]
    fn test_polygon_containing_line_in_boundary() {
        let a = "POLYGON ((0 0, 0 10, 10 10, 10 0, 0 0))";
        let b =
            "GEOMETRYCOLLECTION (POLYGON ((0 0, 10 0, 10 10, 0 10, 0 0)), LINESTRING (0 2, 0 5))";
        check_equals(a, b, true);
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_contains_within(b, a, true);
        check_covers_covered_by(b, a, true);
    }

    #[test]
    fn test_polygon_containing_line_in_boundary_and_interior() {
        let a = "POLYGON ((0 0, 0 10, 10 10, 10 0, 0 0))";
        let b = "GEOMETRYCOLLECTION (POLYGON ((0 0, 10 0, 10 10, 0 10, 0 0)), LINESTRING (0 2, 0 5, 5 5))";
        check_equals(a, b, true);
        check_contains_within(a, b, true);
        check_covers_covered_by(a, b, true);
        check_contains_within(b, a, true);
        check_covers_covered_by(b, a, true);
    }
}

// The Mod-2 (OGC SFS) assertions of JTS RelateNGBoundaryNodeRuleTest.java.
// The EndPoint / MonoValent / MultiValent rule assertions are not ported:
// this port supports the Mod-2 rule only.
// testMultiLineStringSelfIntTouchAtEndpoint has only an EndPoint-rule
// assertion and is omitted entirely.
mod relate_ng_boundary_node_rule_test {
    use super::*;

    #[test]
    fn test_line_string_self_int_touch_at_endpoint() {
        let a = "LINESTRING (20 20, 100 100, 100 20, 20 100)";
        let b = "LINESTRING (60 60, 20 60)";
        check_relate(a, b, "F01FF0102");
    }

    #[test]
    fn test_multi_line_string_touch_at_endpoint() {
        let a = "MULTILINESTRING ((0 0, 10 10), (10 10, 20 20))";
        let b = "LINESTRING (10 10, 20 0)";
        // Under Mod-2 the A touch point is not a boundary:
        // A.int / B.bdy = 0.
        check_relate(a, b, "F01FF0102");
    }

    #[test]
    fn test_multi_line_string_closed_touch_at_endpoint() {
        let a = "MULTILINESTRING ((0 0, 10 10), (10 10, 0 20, 0 0))";
        let b = "LINESTRING (10 10, 20 0)";
        // Under Mod-2, A has no boundary: A.int / B.bdy = 0.
        check_relate(a, b, "F01FFF102");
    }

    #[test]
    fn test_line_ring_touch_at_endpoints() {
        let a = "LINESTRING (20 100, 20 220, 120 100, 20 100)";
        let b = "LINESTRING (20 20, 20 100)";
        // Under Mod-2, A has no boundary: A.int / B.bdy = 0.
        check_relate(a, b, "F01FFF102");
    }

    #[test]
    fn test_line_ring_touch_at_endpoint_and_interior() {
        let a = "LINESTRING (20 100, 20 220, 120 100, 20 100)";
        let b = "LINESTRING (20 20, 40 100)";
        check_relate(a, b, "F01FFF102");
    }

    #[test]
    fn test_polygon_empty_ring() {
        let a = "POLYGON EMPTY";
        let b = "LINESTRING (20 100, 20 220, 120 100, 20 100)";
        // A closed line has no boundary under the SFS rule.
        check_relate(a, b, "FFFFFF1F2");
    }

    #[test]
    fn test_polygon_empty_multi_line_string_closed() {
        let a = "POLYGON EMPTY";
        let b = "MULTILINESTRING ((0 0, 0 1), (0 1, 1 1, 1 0, 0 0))";
        check_relate(a, b, "FFFFFF1F2");
    }

    #[test]
    fn test_polygon_equal_rotated() {
        let a = "POLYGON ((0 0, 140 0, 140 140, 0 140, 0 0))";
        let b = "POLYGON ((140 0, 0 0, 0 140, 140 140, 140 0))";
        // The boundary node rule only considers linear endpoints, so the
        // result is the same for all rules.
        check_relate(a, b, "2FFF1FFF2");
    }

    #[test]
    fn test_line_string_interior_touch_multivalent() {
        let a = "POLYGON EMPTY";
        let b = "MULTILINESTRING ((0 0, 0 1), (0 1, 1 1, 1 0, 0 0))";
        check_relate(a, b, "FFFFFF1F2");
    }
}
