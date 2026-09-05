use super::*;
use crate::geometry::*;
use crate::kernels::Orientation;
use crate::{CoordsIter, GeoNum, Relate, coord, line_string, polygon, wkt};

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

// https://github.com/georust/geo/issues/1566
#[test]
fn convex_hull_at_large_magnitudes() {
    let points = wkt!(MULTIPOINT(0.0 0.0,-1.0 9150170671525436.0,63.0 0.0,0.0 1.0));
    let hull = points.convex_hull();

    // (0, 1) lies inside the triangle formed by the other three.
    let expected =
        wkt!(POLYGON((-1.0 9150170671525436.0,0.0 0.0,63.0 0.0,-1.0 9150170671525436.0)));
    assert!(hull.relate(&expected).is_equal_topo());

    // No input point may lie outside an edge of its own hull.
    for edge in hull.exterior().lines() {
        for point in &points {
            assert_ne!(
                <f64 as GeoNum>::Ker::orient2d(edge.start, edge.end, point.0),
                Orientation::Clockwise
            );
        }
    }

    // The indices agree with the coords they name.
    let coords: Vec<_> = points.exterior_coords_iter().collect();
    let by_index: LineString<f64> = points
        .convex_hull_idx()
        .iter()
        .map(|i| coords[*i])
        .collect();
    assert_eq!(&by_index, hull.exterior());
}
