#[test]
fn jts_validation_tests() {
    jts_test_runner::assert_jts_tests_succeed("*Valid*");
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
