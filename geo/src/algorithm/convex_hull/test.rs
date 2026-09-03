use super::*;
use crate::geometry::*;
use crate::{coord, line_string, polygon};

#[test]
fn test_zero_points() {
    let mut v: Vec<Coord<i64>> = vec![];
    let correct = vec![];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_zero_points_include_on_hull() {
    let mut v: Vec<Coord<i64>> = vec![];
    let correct = vec![];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_one_point() {
    let mut v = vec![coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_one_point_include_on_hull() {
    let mut v = vec![coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points() {
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 1, y: 1 }];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points_include_on_hull() {
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 1, y: 1 }];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points_duplicated() {
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_two_points_duplicated_include_on_hull() {
    let mut v = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_ccw() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 0 },
        coord! { x: 1, y: 1 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_cw() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 1, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_two_duplicated() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_two_duplicated_include_on_hull() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_duplicated() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![coord! { x: 0, y: 0 }, coord! { x: 0, y: 0 }];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_points_duplicated_include_on_hull() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_collinear_points() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 2, y: 2 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 2, y: 2 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, false);
    assert_eq!(res.0, correct);
}

#[test]
fn test_three_collinear_points_include_on_hull() {
    let mut v = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 2, y: 2 },
    ];
    let correct = vec![
        coord! { x: 0, y: 0 },
        coord! { x: 1, y: 1 },
        coord! { x: 2, y: 2 },
        coord! { x: 0, y: 0 },
    ];
    let res = trivial_hull(&mut v, true);
    assert_eq!(res.0, correct);
}

#[test]
fn convex_hull_multipoint_test() {
    let v = vec![
        Point::new(0, 10),
        Point::new(1, 1),
        Point::new(10, 0),
        Point::new(1, -1),
        Point::new(0, -10),
        Point::new(-1, -1),
        Point::new(-10, 0),
        Point::new(-1, 1),
        Point::new(0, 10),
    ];
    let mp = MultiPoint::new(v);
    let correct = vec![
        Coord::from((0, -10)),
        Coord::from((10, 0)),
        Coord::from((0, 10)),
        Coord::from((-10, 0)),
        Coord::from((0, -10)),
    ];
    let res = mp.convex_hull();
    assert_eq!(res.exterior().0, correct);
}
#[test]
fn convex_hull_linestring_test() {
    let mp = line_string![
        (x: 0.0, y: 10.0),
        (x: 1.0, y: 1.0),
        (x: 10.0, y: 0.0),
        (x: 1.0, y: -1.0),
        (x: 0.0, y: -10.0),
        (x: -1.0, y: -1.0),
        (x: -10.0, y: 0.0),
        (x: -1.0, y: 1.0),
        (x: 0.0, y: 10.0),
    ];
    let correct = vec![
        Coord::from((0.0, -10.0)),
        Coord::from((10.0, 0.0)),
        Coord::from((0.0, 10.0)),
        Coord::from((-10.0, 0.0)),
        Coord::from((0.0, -10.0)),
    ];
    let res = mp.convex_hull();
    assert_eq!(res.exterior().0, correct);
}
#[test]
fn convex_hull_multilinestring_test() {
    let v1 = line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0)];
    let v2 = line_string![(x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 3.0, y: 1.0)];
    let mls = MultiLineString::new(vec![v1, v2]);
    let correct = vec![
        Coord::from((2.0, 0.0)),
        Coord::from((3.0, 1.0)),
        Coord::from((1.0, 10.0)),
        Coord::from((0.0, 0.0)),
        Coord::from((2.0, 0.0)),
    ];
    let res = mls.convex_hull();
    assert_eq!(res.exterior().0, correct);
}
#[test]
fn convex_hull_multipolygon_test() {
    let p1 = polygon![(x: 0.0, y: 0.0), (x: 1.0, y: 10.0), (x: 2.0, y: 0.0), (x: 0.0, y: 0.0)];
    let p2 = polygon![(x: 3.0, y: 0.0), (x: 4.0, y: 10.0), (x: 5.0, y: 0.0), (x: 3.0, y: 0.0)];
    let mp = MultiPolygon::new(vec![p1, p2]);
    let correct = vec![
        Coord::from((5.0, 0.0)),
        Coord::from((4.0, 10.0)),
        Coord::from((1.0, 10.0)),
        Coord::from((0.0, 0.0)),
        Coord::from((5.0, 0.0)),
    ];
    let res = mp.convex_hull();
    assert_eq!(res.exterior().0, correct);
}

#[test]
fn collection() {
    let collection = GeometryCollection(vec![
        Point::new(0.0, 0.0).into(),
        Triangle::new(
            coord! { x: 1.0, y: 0.0},
            coord! { x: 4.0, y: 0.0},
            coord! { x: 4.0, y: 4.0 },
        )
        .into(),
    ]);

    let convex_hull = collection.convex_hull();
    assert_eq!(
        convex_hull,
        polygon![
            coord! { x: 4.0, y: 0.0 },
            coord! { x: 4.0, y: 4.0 },
            coord! { x: 0.0, y: 0.0 }
        ]
    );
}

#[test]
fn convex_hull_with_nan_does_not_panic() {
    let pts = MultiPoint::new(vec![
        Point::new(0.0, 0.0),
        Point::new(f64::NAN, 1.0),
        Point::new(1.0, 1.0),
    ]);
    let _ = pts.convex_hull();
}

/// Hegel property tests for `ConvexHull`.
///
/// The trait promises "the convex hull of a geometry. The hull is always
/// oriented counter-clockwise", so every input point lies on or to the left of
/// every directed hull edge and consecutive hull vertices never turn clockwise.
/// Both checks use `robust::orient2d` — the exact predicate `RobustKernel`
/// wraps — applied directly to the output, so the oracle cannot share an
/// arithmetic mistake with `quick_hull`.
mod hegel_props {
    use crate::geometry::{Coord, MultiPoint, Point};
    use crate::utils::pbt_gens::{coords, grid_coords};
    use crate::{ConvexHull, CoordsIter, Polygon};
    use hegel::generators::{self, Generator, PrintableGenerator};

    fn orient(a: Coord<f64>, b: Coord<f64>, c: Coord<f64>) -> f64 {
        robust::orient2d(
            robust::Coord { x: a.x, y: a.y },
            robust::Coord { x: b.x, y: b.y },
            robust::Coord { x: c.x, y: c.y },
        )
    }

    fn hull_of(coords: &[Coord<f64>]) -> Polygon<f64> {
        MultiPoint::new(coords.iter().copied().map(Point::from).collect()).convex_hull()
    }

    /// Point sets on the exact integer grid, where every product and difference
    /// `quick_hull` forms is representable in f64. The magnitude-dependent
    /// failures pinned by `hull_at_large_coordinates_leaves_an_input_outside`
    /// cannot arise here, so orientation properties hold on this domain while
    /// #1566 is open.
    fn grid_point_sets() -> impl PrintableGenerator<Vec<Coord<f64>>> {
        generators::vecs(grid_coords()).max_size(64)
    }

    /// Point sets with arbitrary finite coordinates up to `±1e150`. The bound
    /// keeps the products `robust::orient2d` forms internally clear of f64
    /// overflow — beyond it the oracle itself returns NaN — and stays below the
    /// magnitude at which `convex_hull` panics
    /// (`hull_panics_at_large_finite_coordinates`).
    fn point_sets() -> impl PrintableGenerator<Vec<Coord<f64>>> {
        generators::vecs(coords(1e150)).max_size(64)
    }

    #[hegel::test]
    fn every_input_point_is_on_or_left_of_every_hull_edge(tc: hegel::TestCase) {
        let points = tc.draw(grid_point_sets());
        let hull = hull_of(&points);
        for edge in hull.exterior().lines() {
            for &point in &points {
                assert!(
                    orient(edge.start, edge.end, point) >= 0.0,
                    "input point {point:?} lies strictly right of hull edge {edge:?}"
                );
            }
        }
    }

    #[hegel::test]
    fn consecutive_hull_vertices_never_turn_clockwise(tc: hegel::TestCase) {
        let points = tc.draw(grid_point_sets());
        let hull = hull_of(&points);
        let ring = &hull.exterior().0;
        if ring.is_empty() {
            return;
        }
        // The ring is closed, so drop the repeated last coordinate and walk it
        // cyclically.
        let open = &ring[..ring.len() - 1];
        for i in 0..open.len() {
            let (a, b, c) = (
                open[i],
                open[(i + 1) % open.len()],
                open[(i + 2) % open.len()],
            );
            assert!(
                orient(a, b, c) >= 0.0,
                "hull turns clockwise at {b:?} (previous {a:?}, next {c:?})"
            );
        }
    }

    // `quick_hull_indices` expects that "trivial_hull's coord output is a strict
    // subset of the input", and the hull of a point set can only be built from
    // points of that set.
    #[hegel::test]
    fn every_hull_vertex_is_an_input_point(tc: hegel::TestCase) {
        let points = tc.draw(point_sets());
        for vertex in hull_of(&points).exterior() {
            assert!(
                points.contains(vertex),
                "hull vertex {vertex:?} is not one of the input points"
            );
        }
    }

    /// A closed ring rotated to start at its lexicographically least vertex,
    /// with the repeated closing coordinate dropped. The trait fixes the
    /// orientation of the hull but not which vertex it starts from.
    fn canonical_ring(ring: &crate::LineString<f64>) -> Vec<Coord<f64>> {
        if ring.0.is_empty() {
            return Vec::new();
        }
        let open = &ring.0[..ring.0.len() - 1];
        let start = crate::utils::least_index(open);
        open[start..]
            .iter()
            .chain(&open[..start])
            .copied()
            .collect()
    }

    // The hull of a convex set is that set, so hulling twice adds nothing.
    #[hegel::test]
    fn the_hull_of_a_hull_is_the_hull(tc: hegel::TestCase) {
        let points = tc.draw(grid_point_sets());
        let hull = hull_of(&points);
        assert_eq!(
            canonical_ring(hull.convex_hull().exterior()),
            canonical_ring(hull.exterior())
        );
    }

    // The hull is a property of the point set, not of the order the points
    // arrive in.
    #[hegel::test]
    fn the_hull_does_not_depend_on_input_order(tc: hegel::TestCase) {
        let points = tc.draw(grid_point_sets());
        let shuffled = tc.draw(generators::permutations(points.clone()).print_as_debug());
        assert_eq!(
            canonical_ring(hull_of(&shuffled).exterior()),
            canonical_ring(hull_of(&points).exterior())
        );
    }

    // A convex hull is a single filled region: the trait's doc example asserts
    // `res.interiors() == &[]`.
    #[hegel::test]
    fn the_hull_has_no_interior_rings(tc: hegel::TestCase) {
        let points = tc.draw(point_sets());
        assert!(hull_of(&points).interiors().is_empty());
    }

    // `convex_hull_idx` returns "the indices of the input coords (as yielded by
    // `CoordsIter::exterior_coords_iter`) that form the convex hull, in CCW
    // order and closed", so mapping the indices through that iterator must
    // reproduce the hull ring exactly.
    #[hegel::test]
    fn convex_hull_idx_indexes_the_hull_ring(tc: hegel::TestCase) {
        let points = tc.draw(point_sets());
        let multi_point = MultiPoint::new(points.iter().copied().map(Point::from).collect());
        let source: Vec<_> = multi_point.exterior_coords_iter().collect();
        let indexed: Vec<_> = multi_point
            .convex_hull_idx()
            .into_iter()
            .map(|i| source[i])
            .collect();
        assert_eq!(indexed, multi_point.convex_hull().exterior().0);
    }

    // KNOWN FAILURE, #1566 (open): `hull_set` picks its pivot with a
    // naive dot product (`p_orth.x * p_diff.x + p_orth.y * p_diff.y`), so once
    // candidate scores collide after rounding an interior point can be chosen
    // and the returned ring is not a hull. Here (63, 0) ends up strictly right
    // of the edge (0,0) -> (0,1), and the ring visits (0,0) twice.
    #[test]
    #[ignore = "#1566: convex_hull returns a ring that is not a hull at large coordinates"]
    fn hull_at_large_coordinates_leaves_an_input_outside() {
        let points = [
            Coord { x: 0.0, y: 0.0 },
            Coord {
                x: -1.0,
                y: 9150170671525436.0,
            },
            Coord { x: 63.0, y: 0.0 },
            Coord { x: 0.0, y: 1.0 },
        ];
        let hull = hull_of(&points);
        for edge in hull.exterior().lines() {
            for &point in &points {
                assert!(
                    orient(edge.start, edge.end, point) >= 0.0,
                    "input point {point:?} lies strictly right of hull edge {edge:?}"
                );
            }
        }
    }

    // KNOWN FAILURE, same root cause as #1566: the pivot score
    // overflows to infinity for these finite coordinates, `inf - inf` gives
    // NaN, and the `max_by(|a, b| a.partial_cmp(b).unwrap())` in `hull_set`
    // unwraps `None`. Release builds panic too.
    #[test]
    #[ignore = "#1566 (same pivot arithmetic): convex_hull panics at large finite coordinates"]
    fn hull_panics_at_large_finite_coordinates() {
        let points = [
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 1.0, y: 0.0 },
            Coord {
                x: -7.076196536963262e206,
                y: -8.648940011347372e215,
            },
        ];
        assert!(!hull_of(&points).exterior().0.is_empty());
    }
}
