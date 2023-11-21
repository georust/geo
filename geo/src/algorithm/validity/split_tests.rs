use super::*;
use geo_types::*;

use crate::{polygon, CoordsIter, GeoFloat, MapCoords, Winding};
use pretty_env_logger::env_logger::*;

fn one() -> Coord {
    Coord { x: 1.0, y: 1.0 }
}

fn test_consecutive_points_unique<F: GeoFloat>(mp: &MultiPolygon<F>) {
    mp.0.iter().for_each(|p| {
        p.exterior()
            .0
            .windows(2)
            .map(|w| [w[0], w[1]])
            .for_each(|[a, b]| {
                assert_ne!(a, b);
            });
    });
}

#[test]
fn basic_stacked_rectangles_split() {
    _ = try_init();
    let big_rect = Rect::new(Coord::zero(), one() * 3.0);
    let small_rect = Rect::new(one(), one() * 2.0);

    let p = Polygon::new(
        big_rect.to_polygon().exterior().clone(),
        vec![small_rect.to_polygon().exterior().clone()],
    );

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn two_rectangles_inside_separate() {
    _ = try_init();
    let big_rect = Rect::new(Coord::zero(), Coord { x: 10.0, y: 3.0 });
    let small_rect_1 = Rect::new(one(), one() * 2.0);
    let small_rect_2 = Rect::new(Coord { x: 8.0, y: 1.0 }, Coord { x: 9.0, y: 2.0 });

    let p = Polygon::new(
        big_rect.to_polygon().exterior().clone(),
        [small_rect_1, small_rect_2]
            .map(|r| r.to_polygon().exterior().clone())
            .to_vec(),
    );

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 3);
    test_consecutive_points_unique(&split);
}

#[test]
fn two_rectangles_inside_together() {
    _ = try_init();
    let big_rect = Rect::new(Coord::zero(), Coord { x: 10.0, y: 3.0 });
    let small_rect_1 = Rect::new(Coord { x: 4.0, y: 1.0 }, Coord { x: 4.25, y: 2.0 });
    let small_rect_2 = Rect::new(Coord { x: 4.75, y: 1.0 }, Coord { x: 5.0, y: 2.0 });

    let p = Polygon::new(
        big_rect.to_polygon().exterior().clone(),
        [small_rect_1, small_rect_2]
            .map(|r| r.to_polygon().exterior().clone())
            .to_vec(),
    );

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 3);
    test_consecutive_points_unique(&split);
}

#[test]
fn two_rectangles_inside_ext_and_hole_connecting() {
    _ = try_init();
    let big_rect = Rect::new(Coord::zero(), Coord { x: 10.0, y: 3.0 });
    let small_rect_1 = Rect::new(Coord { x: 0.5, y: 1.0 }, Coord { x: 0.75, y: 2.0 });
    let small_rect_2 = Rect::new(Coord { x: 1.0, y: 1.0 }, Coord { x: 2.0, y: 2.0 });

    let p = Polygon::new(
        big_rect.to_polygon().exterior().clone(),
        [small_rect_1, small_rect_2]
            .map(|r| r.to_polygon().exterior().clone())
            .to_vec(),
    );

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 3);
    test_consecutive_points_unique(&split);
}

#[test]
fn funny_case_two_holes() {
    _ = try_init();

    let p = polygon! {
        exterior: [
            (x: 0.0, y: 0.0),
            (x: 8.0, y: 0.0),
            (x: 8.0, y: 9.0),
            (x: -1.0, y: 9.0),
            (x: 2.0, y: 4.0),
        ],
        interiors: [
            [
                (x: 4.0, y: 4.0),
                (x: 6.0, y: 5.0),
                (x: 3.0, y: 5.0),
            ],
            [
                (x: 3.0, y: 6.0),
                (x: 7.0, y: 5.0),
                (x: 7.0, y: 6.0),
            ],
        ]
    }
    .map_coords(|c| Coord { x: c.x, y: -c.y });

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn poly_within_a_poly() {
    _ = try_init();
    let big_rect = Rect::new(Coord::zero(), one() * 10.0);
    let big_rect_hole = Rect::new(one(), one() * 9.0);
    let small_rect = Rect::new(one() * 2.0, one() * 8.0);
    let small_rect_hole = Rect::new(one() * 3.0, one() * 7.0);

    let outer = Polygon::new(
        big_rect.to_polygon().exterior().clone(),
        vec![big_rect_hole.to_polygon().exterior().clone()],
    );
    let inner = Polygon::new(
        small_rect.to_polygon().exterior().clone(),
        vec![small_rect_hole.to_polygon().exterior().clone()],
    );

    let p = MultiPolygon::new(vec![outer, inner]);

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 4);
    test_consecutive_points_unique(&split);
}

#[test]
fn poly_within_a_poly_within_a_poly() {
    _ = try_init();
    let big_rect = Rect::new(Coord::zero(), one() * 10.0);
    let big_rect_hole = Rect::new(one(), one() * 9.0);
    let middle_rect = Rect::new(one() * 2.0, one() * 8.0);
    let middle_rect_hole = Rect::new(one() * 3.0, one() * 7.0);
    let small_rect = Rect::new(one() * 4.0, one() * 6.0);
    let small_rect_hole = Rect::new(one() * 4.5, one() * 5.5);

    let outer = Polygon::new(
        big_rect.to_polygon().exterior().clone(),
        vec![big_rect_hole.to_polygon().exterior().clone()],
    );
    let middle = Polygon::new(
        middle_rect.to_polygon().exterior().clone(),
        vec![middle_rect_hole.to_polygon().exterior().clone()],
    );
    let inner = Polygon::new(
        small_rect.to_polygon().exterior().clone(),
        vec![small_rect_hole.to_polygon().exterior().clone()],
    );

    let p = MultiPolygon::new(vec![outer, middle, inner]);

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 6);
    test_consecutive_points_unique(&split);
}

// if this were the case, this test would create a banana polygon which is basically not
// hole-less
#[test]
fn connection_to_exterior_isnt_same_point() {
    _ = try_init();
    let big_rect = Rect::new(Coord::zero(), one() * 100.0);
    let tri1 = Triangle::new(
        Coord { x: 49.0, y: 1.0 },
        Coord { x: 49.5, y: 1.0 },
        Coord { x: 49.5, y: 25.0 },
    );
    let tri2 = Triangle::new(
        Coord { y: 49.0, x: 1.0 },
        Coord { y: 49.5, x: 1.0 },
        Coord { y: 49.5, x: 25.0 },
    );

    let p = Polygon::new(
        big_rect.to_polygon().exterior().clone(),
        [tri1, tri2]
            .map(|t| t.to_polygon().exterior().clone())
            .to_vec(),
    );

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn connection_to_exterior_isnt_same_point_real_data() {
    let p = polygon! {
        exterior: [
            (x: -32.24332, y: -20.182356),
            (x: -9.921326, y: -43.82522),
            (x: -2.090634, y: -10.18567),
            (x: -24.399603, y: -6.548216),
            (x: -32.24332, y: -20.182356)
        ],
        interiors: [
            [
                (x: -15.338705, y: -25.178802),
                (x: -16.906475, y: -22.304386),
                (x: -14.203756, y: -21.644667),
                (x: -15.338705, y: -25.178802),
            ],
            [
                (x: -14.332268, y: -16.889647),
                (x: -25.32022, y: -16.11856),
                (x: -18.89452, y: -13.0395775),
                (x: -14.332268, y: -16.889647),
            ],
        ],
    };

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn connecting_line_doesnt_intersect_other_line() {
    let p = polygon! {
        exterior: [
            Coord { x: -16.599487, y: -15.243347, },
            Coord { x: 22.438654, y: -20.59876, },
            Coord { x: 13.0709915, y: 11.69747, },
            Coord { x: -18.707924, y: 4.3485994, },
            Coord { x: -18.472523, y: -10.658596, },
            Coord { x: -16.599487, y: -15.243347, },
        ],
        interiors: [
            [
                Coord { x: -14.979336, y: -7.160144, },
                Coord { x: -6.7293396, y: -0.86014414, },
                Coord { x: 1.4206578, y: -5.460142, },
                Coord { x: -14.979336, y: -7.160144, },
            ],
            [
                Coord { x: -12.000986, y: -11.615011, },
                Coord { x: 1.9206573, y: -8.860142, },
                Coord { x: -2.417613, y: -14.433651, },
                Coord { x: -12.000986, y: -11.615011, },
            ],
        ],
    };

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn winding_sanity_check() {
    // I think this caused a bug and if not we don't need to prove it again

    let mut t1 = LineString::new(
        Triangle::from([
            Coord {
                x: 8.237429,
                y: 9.415966,
            },
            Coord {
                x: -4.9125605,
                y: 8.0659685,
            },
            Coord {
                x: -0.6625643,
                y: 11.465963,
            },
        ])
        .to_array()
        .to_vec(),
    );
    let mut t2 = LineString::new(
        Triangle::from([
            Coord {
                x: -10.012558,
                y: 1.6159716,
            },
            Coord {
                x: 4.0338764,
                y: 1.6316788,
            },
            Coord {
                x: -1.9625626,
                y: -1.4340262,
            },
        ])
        .to_array()
        .to_vec(),
    );

    t1.close();
    t1.make_cw_winding();
    t2.close();
    t2.make_cw_winding();

    let ls = t1.coords_iter().chain(t2.coords_iter()).collect::<Vec<_>>();

    let p = Polygon::new(LineString::new(ls), vec![]);

    let mut ls = p.exterior().clone();
    assert!(!ls.is_cw());
    ls.make_cw_winding();
    // !!! 😮 !!!
    assert!(
        !ls.is_cw(),
        "!!! IF THIS TEST FAILS THAT'S ACTUALLY A GOOD SIGN SINCE THE ASSERTION IS WEIRD !!!"
    );
}

#[test]
fn holes_arent_used_twice() {
    _ = try_init();
    let p = polygon! {
        exterior: [
            Coord { x: -15.516099, y: 0.4316782, },
            Coord { x: -10.416106, y: -6.7683125, },
            Coord { x: 11.183868, y: -2.5683184, },
            Coord { x: 16.634357, y: 16.267822, },
            Coord { x: -13.116101, y: 14.781662, },
            Coord { x: -15.516099, y: 0.4316782, },
        ],
        interiors: [
            [
                Coord { x: 8.237429, y: 9.415966, },
                Coord { x: -4.9125605, y: 8.0659685, },
                Coord { x: -0.6625643, y: 11.465963, },
                Coord { x: 8.237429, y: 9.415966, },
            ],
            [
                Coord { x: -10.012558, y: 1.6159716, },
                Coord { x: 4.0338764, y: 1.6316788, },
                Coord { x: -1.9625626, y: -1.4340262, },
                Coord { x: -10.012558, y: 1.6159716, },
            ],
        ],
    };

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn no_crossing_connecting_lines() {
    _ = try_init();
    let p = polygon! {
        exterior: [
            Coord { x: 30.180992, y: 131.80675, },
            Coord { x: -406.34613, y: -18.371037, },
            Coord { x: -30.24865, y: -56.683075, },
            Coord { x: 43.076523, y: -34.146957, },
            Coord { x: 43.471027, y: 12.403208, },
            Coord { x: 30.180992, y: 131.80675, },
        ],
        interiors: [[
            Coord { x: -149.56546, y: -4.3648252, },
            Coord { x: -57.746937, y: 21.313248, },
            Coord { x: -59.631237, y: -8.495655, },
            Coord { x: -149.56546, y: -4.3648252, },
        ]],
    };

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn interior_one_vertex_on_exterior() {
    _ = try_init();
    let p = polygon!(
        exterior: [
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 50.0, y: 0.0 },
            Coord { x: 100.0, y: 0.0 },
            Coord { x: 50.0, y: 100.0 }
        ],
        interiors: [[
            Coord { x: 50.0, y: 0.0 },
            Coord { x: 52.5, y: 2.5 },
            Coord { x: 47.5, y: 2.5 },
        ]],
    );

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}

#[test]
fn point_and_line_connection() {
    _ = try_init();
    let p = polygon! {
        Coord { x: -17.5161, y: 5.790348, },
        Coord { x: -3.7623396, y: -3.2200394, },
        Coord { x: -17.965897, y: -24.153437, },
        Coord { x: 25.049488, y: -16.02196, },
        Coord { x: 29.76677, y: 8.099451, },
        Coord { x: 5.395004, y: 8.099451, },
        Coord { x: 5.616444, y: 4.0554075, },
        Coord { x: 10.43572, y: -0.892385, },
        Coord { x: 3.3031864, y: -4.8763237, },
        Coord { x: 4.5291767, y: -8.794404, },
        Coord { x: 10.440827, y: -12.521313, },
        Coord { x: -5.6928024, y: -16.63536, },
        Coord { x: 4.5291767, y: -8.794404, },
        Coord { x: 3.3031864, y: -4.8763237, },
        Coord { x: -3.7623396, y: -3.2200394, },
        Coord { x: 5.616444, y: 4.0554075, },
        Coord { x: 5.395004, y: 8.099451, },
        Coord { x: -17.5161, y: 5.790348, },
    };

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 3);
    test_consecutive_points_unique(&split);
}

// this was found but we didn't really care about this case at the time. The issue here is that
// the algo splits the polygon into two polygons, which is good. But one of the return polygons
// is banana like which is kind of invalid and trips up algorithms
#[test]
fn interior_one_vertex_on_exterior_found_acidentally() {
    _ = try_init();
    let p = polygon!(
        exterior: [
            Coord { x: 0.0, y: 0.0 },
            Coord { x: 55.0, y: 0.0 },
            Coord { x: 100.0, y: 0.0 },
            Coord { x: 50.0, y: 100.0 }
        ],
        interiors: [[
            Coord { x: 50.0, y: 0.0 },
            Coord { x: 52.5, y: 2.5 },
            Coord { x: 47.5, y: 2.5 },
        ]],
    );

    let split = p.split_into_valid();

    assert_eq!(split.0.len(), 2);
    test_consecutive_points_unique(&split);
}
