use super::*;
use super::test_helper::*;
use geo_types::*;

// helper

#[test]
fn basic_intersection_compiles() {
    let zero = Coord::zero();
    let one = Coord { x: 1.0, y: 1.0 };
    let rect1 = Rect::new(zero, one * 2.0);
    let rect2 = Rect::new(one, one * 3.0);

    SpadeBoolops::intersection(&rect1.to_polygon(), &rect2.to_polygon()).unwrap();
}

macro_rules! define_test {
    (
        name      = $test_name:ident,
        path      = $path:expr,
        operation = $op:expr,
        results: 
        empty     = $empty:expr,
        num_polys = $num_polys:expr,
        num_holes = $num_holes:expr,
        num_verts = $num_verts:expr,
     ) => {
        #[test]
        fn $test_name() {
            _ = pretty_env_logger::try_init();
            let data = include_str!($path);
            let data = load_wkt(data).unwrap();

            let f = $op;
            let res = f(data);

            if $empty {
                is_multipolygon_empty(&res);
            } else {
                is_multipolygon_nonempty(&res);
            }
            has_num_polygons(&res, $num_polys);
            has_num_holes(&res, $num_holes);
            has_num_vertices(&res, $num_verts);
        }
    };
}

define_test!(
    name      = star_shape_slightly_offset_difference_1,
    path      = "../test_data/star.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let poly1 = &data[0];
        let mut poly2 = poly1.clone();
        poly2.exterior_mut(|ext| {
            ext.coords_mut().skip(1).take(1).for_each(|coord| {
                *coord = *coord + Coord { x: 0.1, y: 0.1 };
            });
        });
        Polygon::difference(poly1, &poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![4],
);

define_test!(
    name      = star_shape_slightly_offset_difference_2,
    path      = "../test_data/star.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let poly1 = &data[0];
        let mut poly2 = poly1.clone();
        poly2.exterior_mut(|ext| {
            ext.coords_mut().skip(1).take(1).for_each(|coord| {
                *coord = *coord + Coord { x: 0.1, y: 0.1 };
            });
        });
        Polygon::difference(&poly2, poly1).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![4],
);

define_test!(
    name      = star_intersects_self_properly,
    path      = "../test_data/star.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let poly1 = &data[0];
        Polygon::intersection(poly1, poly1).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![23],
);

define_test!(
    name      = duplicate_points_intersection_works_1,
    path      = "../test_data/duplicate_points_case_1.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![5],
);

define_test!(
    name      = duplicate_points_intersection_works_2,
    path      = "../test_data/duplicate_points_case_2.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![6],
);

define_test!(
    name      = duplicate_points_difference_works_1,
    path      = "../test_data/duplicate_points_case_3.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::difference(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![9],
);

define_test!(
    name      = collinear_outline_parts_intersection_works,
    path      = "../test_data/collinear_outline_parts.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![5],
);

define_test!(
    name      = missing_triangle_intersection_works_1,
    path      = "../test_data/missing_triangle_case_1.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![5],
);

define_test!(
    name      = missing_triangle_intersection_empty,
    path      = "../test_data/missing_triangle_case_2.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = true,
    num_polys = 0,
    num_holes = vec![],
    num_verts = vec![],
);

define_test!(
    name      = missing_triangle_intersection_works_2,
    path      = "../test_data/missing_triangle_case_3.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![7],
);

define_test!(
    name      = intersection_at_address_works_1,
    path      = "../test_data/intersection_address_001.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![5],
);

define_test!(
    name      = difference_at_address_works_1,
    path      = "../test_data/intersection_address_001.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::difference(poly2, poly1).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![10],
);

define_test!(
    name      = intersection_at_address_works_2,
    path      = "../test_data/intersection_address_002.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![9],
);

define_test!(
    name      = difference_at_address_works_2,
    path      = "../test_data/intersection_address_002.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::difference(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![18],
);

define_test!(
    name      = intersection_doesnt_fail_after_union_fix_1,
    path      = "../test_data/intersection_fail_after_union_fix_1.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![5],
);

define_test!(
    name      = difference_doesnt_fail_after_union_fix_1,
    path      = "../test_data/intersection_fail_after_union_fix_1.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::difference(poly2, poly1).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![10],
);

define_test!(
    name      = intersection_doesnt_fail_after_union_fix_2,
    path      = "../test_data/intersection_fail_after_union_fix_2.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::intersection(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![5],
);

define_test!(
    name      = holes_are_preserved_by_union,
    path      = "../test_data/holes_are_preserved.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let poly1 = &data[0];
        Polygon::union(poly1, poly1).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![10],
);

define_test!(
    name      = holes_are_preserved_by_intersection,
    path      = "../test_data/holes_are_preserved.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let poly1 = &data[0];
        Polygon::intersection(poly1, poly1).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![10],
);

define_test!(
    name      = holes_are_preserved_by_difference,
    path      = "../test_data/holes_are_preserved.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let poly1 = &data[0];
        Polygon::difference(poly1, &empty_poly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![10],
);

define_test!(
    name      = one_hole_after_union,
    path      = "../test_data/hole_after_union.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::union(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![14],
);

define_test!(
    name      = two_holes_after_union,
    path      = "../test_data/two_holes_after_union.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        let [poly1, poly2] = [&data[0], &data[1]];
        Polygon::union(poly1, poly2).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![2],
    num_verts = vec![21],
);

define_test!(
    name      = union_at_address_13_works,
    path      = "../test_data/union_address_013.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        // imprecise inputs lead to hole
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![1],
    num_verts = vec![17],
);

define_test!(
    name      = union_at_address_14_works,
    path      = "../test_data/union_address_014.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![8],
);

define_test!(
    name      = simple_union,
    path      = "../test_data/simple_union.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![7],
);

define_test!(
    name      = multiple_unions,
    path      = "../test_data/multiple_unions.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 2,
    num_holes = vec![0, 0],
    num_verts = vec![7, 7],
);

define_test!(
    name      = union_preserved_intermediate_points_1,
    path      = "../test_data/union_both_intermediate_points_stay.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![7],
);

define_test!(
    name      = union_preserved_intermediate_points_2,
    path      = "../test_data/union_one_intermediate_point_stays.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![7],
);

define_test!(
    name      = union_not_completely_shared_line,
    path      = "../test_data/union_not_completely_shared_line.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![9],
);

define_test!(
    name      = union_at_address_015_works,
    path      = "../test_data/union_address_015.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![20],
);

define_test!(
    name      = union_works_on_overlap_1,
    path      = "../test_data/union_fails_overlap.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![6],
);

define_test!(
    name      = union_works_on_overlap_2,
    path      = "../test_data/union_still_fails_overlap.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![52],
);

define_test!(
    name      = union_at_address_1_works,
    path      = "../test_data/union_address_001.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![69],
);

define_test!(
    name      = union_at_address_2_works,
    path      = "../test_data/union_address_002.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![70],
);

define_test!(
    name      = union_at_address_3_works,
    path      = "../test_data/union_address_003.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 2,
    num_holes = vec![0, 0],
    num_verts = vec![19, 18],
);

define_test!(
    name      = union_at_address_4_works,
    path      = "../test_data/union_address_004.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 2,
    num_holes = vec![0, 0],
    num_verts = vec![4, 24],
);

define_test!(
    name      = union_at_address_5_works,
    path      = "../test_data/union_address_005.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 2,
    num_holes = vec![0, 0],
    num_verts = vec![71, 19],
);

define_test!(
    name      = union_at_address_6_works,
    path      = "../test_data/union_address_006.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 2,
    num_holes = vec![0, 0],
    num_verts = vec![22, 31],
);

define_test!(
    name      = union_at_address_7_works,
    path      = "../test_data/union_address_007.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![14],
);

define_test!(
    name      = union_at_address_8_works,
    path      = "../test_data/union_address_008.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 2,
    num_holes = vec![0, 0],
    num_verts = vec![10, 5],
);

define_test!(
    name      = union_at_address_9_works,
    path      = "../test_data/union_address_009.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 2,
    num_holes = vec![0, 0],
    num_verts = vec![41, 6],
);

define_test!(
    name      = union_at_address_10_works,
    path      = "../test_data/union_address_010.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![28],
);

define_test!(
    name      = union_at_address_11_works,
    path      = "../test_data/union_address_011.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![22],
);

define_test!(
    name      = union_at_address_12_works,
    path      = "../test_data/union_address_012.wkt",
    operation = |data: Vec<Polygon<f32>>| {
        MultiPolygon::union(&multipolygon_from(data), &empty_multipoly()).unwrap()
    },
    results: 
    empty     = false,
    num_polys = 1,
    num_holes = vec![0],
    num_verts = vec![22],
);
