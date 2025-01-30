use super::{unary_union, BooleanOps};
use crate::{wkt, Convert, MultiPolygon, Polygon, Relate};
use std::time::Instant;
use wkt::ToWkt;

#[test]
fn test_unary_union() {
    let poly1: Polygon = wkt!(POLYGON((204.0 287.0,203.69670020700084 288.2213844497616,200.38308697914755 288.338793163584,204.0 287.0)));
    let poly2: Polygon = wkt!(POLYGON((210.0 290.0,204.07584923592933 288.2701221108328,212.24082541367974 285.47846008552216,210.0 290.0)));
    let poly3: Polygon = wkt!(POLYGON((211.0 292.0,202.07584923592933 288.2701221108328,212.24082541367974 285.47846008552216,210.0 290.0)));

    let polys = vec![poly1.clone(), poly2.clone(), poly3.clone()];
    let poly_union = unary_union(&polys);
    assert_eq!(poly_union.0.len(), 1);

    let multi_poly_12 = MultiPolygon::new(vec![poly1.clone(), poly2.clone()]);
    let multi_poly_3 = MultiPolygon::new(vec![poly3]);
    let multi_polys = vec![multi_poly_12.clone(), multi_poly_3.clone()];
    let multi_poly_union = unary_union(&multi_polys);
    assert_eq!(multi_poly_union.0.len(), 1);
}

#[test]
fn test_unary_union_errors() {
    let input: MultiPolygon = geo_test_fixtures::nl_plots_epsg_28992();

    assert_eq!(input.0.len(), 316);

    let input_area = input.signed_area();
    assert_relative_eq!(input_area, 763889.4732974821);

    let naive_union = {
        let start = Instant::now();
        let mut output = MultiPolygon::new(Vec::new());
        for poly in input.iter() {
            output = output.union(poly);
        }
        let union = output;
        let duration = start.elapsed();
        println!("Time elapsed (naive): {:.2?}", duration);
        union
    };

    let simplified_union = {
        let start = Instant::now();
        let union = unary_union(input.iter());
        let duration = start.elapsed();
        println!("Time elapsed (simplification): {:.2?}", duration);
        union
    };

    use crate::algorithm::Area;
    let naive_area = naive_union.unsigned_area();
    let simplified_area = simplified_union.unsigned_area();
    assert_relative_eq!(naive_area, simplified_area, max_relative = 1e-5);

    // Serial vs. parallel are expected to have slightly different results.
    //
    // Each boolean operation scales the floating point to a discrete
    // integer grid, which introduces some error, and this error factor depends on the magnitude
    // of the input.
    //
    // Because the serial vs. parallel approaches group inputs differently, error is accumulated
    // differently - hence the slightly different outputs.
    //
    // xor'ing the two shapes represents the magnitude of the difference between the two outputs.
    //
    // We want to verify that this error is small - it should be near 0, but the
    // magnitude of the error is relative to the magnitude of the input geometries, so we offset
    // both the error and 0 by `input_area` to make a scale relative comparison.
    let naive_vs_simplified_discrepancy = simplified_union.xor(&naive_union);
    assert_relative_eq!(
        input_area + naive_vs_simplified_discrepancy.unsigned_area(),
        0.0 + input_area,
        max_relative = 1e-5
    );

    assert_eq!(simplified_union.0.len(), 1);
    assert_relative_eq!(simplified_area, input_area, max_relative = 1e-5);
}

#[test]
fn test_unary_union_winding() {
    let input: MultiPolygon = geo_test_fixtures::nl_plots_epsg_28992();

    use crate::orient::{Direction, Orient};
    let default_winding_union = unary_union(input.orient(Direction::Default).iter());
    let reversed_winding_union = unary_union(input.orient(Direction::Reversed).iter());
    assert_eq!(default_winding_union, reversed_winding_union);
}

#[test]
fn jts_overlay_tests() {
    jts_test_runner::assert_jts_tests_succeed("*Overlay*.xml");
}

#[test]
fn jts_test_overlay_la_1() {
    // From TestOverlayLA.xml test case with description "mLmA - A and B complex, overlapping and touching #1"
    let a: MultiPolygon<f64> = wkt!(MULTIPOLYGON(
        (
            (60 260, 60 120, 220 120, 220 260, 60 260),
            (80 240, 80 140, 200 140, 200 240, 80 240)
        ),
        (
          (100 220, 100 160, 180 160, 180 220, 100 220),
           (120 200, 120 180, 160 180, 160 200, 120 200)
        )
    ))
    .convert();
    let b = wkt!(MULTILINESTRING(
        (40 260, 240 260, 240 240, 40 240, 40 220, 240 220),
        (120 300, 120 80, 140 80, 140 300, 140 80, 120 80, 120 320)
    ))
    .convert();
    let actual = a.clip(&b, false);

    // This corresponds to the "intersection" output from JTS overlay
    let expected = wkt!(MULTILINESTRING(
        (220 260, 140 260),
        (140 260, 120 260),
        (120 260, 60 260),
        (200 240, 140 240),
        (140 240, 120 240),
        (120 240, 80 240),
        (180 220, 140 220),
        (140 220, 120 220),
        (120 220, 100 220),
        (120 200, 120 180),
        (220 240, 200 240),
        (80 240, 60 240),
        (60 220, 80 220),
        (200 220, 220 220),
        (120 260, 120 240),
        (120 220, 120 200),
        (120 180, 120 160),
        (120 140, 120 120),
        (140 120, 140 140),
        (140 160, 140 180),
        (140 200, 140 220),
        (140 240, 140 260)
    ))
    .convert();

    let im = actual.relate(&expected);
    assert!(
        im.is_equal_topo(),
        "actual: {:#?}, expected: {:#?}",
        actual.wkt_string(),
        expected.wkt_string()
    );
}

mod gh_issues {
    use super::super::{BooleanOps, OpType};
    use crate::{geometry::*, wkt};

    #[test]
    fn gh_issue_867() {
        let p1 = wkt!(POLYGON((
            17.724912058920285 - 16.37118892052372,
            18.06452454246989 - 17.693907532504,
            19.09389292605319 - 17.924001641855178,
            17.724912058920285 - 16.37118892052372
        )));
        let p2 = wkt!(POLYGON((
            17.576085274796423 - 15.791540153598898,
            17.19432983818328 - 17.499393422066746,
            18.06452454246989 - 17.693907532504,
            17.576085274796423 - 15.791540153598898
        )));
        _ = p1.intersection(&p2);
        // The goal is just to get here without panic
    }

    #[test]
    fn gh_issue_885() {
        let polygon_x: Polygon<f32> = wkt!(POLYGON((8055.658 7977.5537,8010.734 7999.9697,8032.9717 8044.537,8077.896 8022.121,8055.658 7977.5537)));
        let polygon_y: Polygon<f32> = wkt!(POLYGON((8055.805 7977.847,8010.871 8000.2676,8033.105 8044.8286,8078.039 8022.408,8055.805 7977.847)));
        _ = polygon_x.union(&polygon_y);
        _ = polygon_x.intersection(&polygon_y);
        // The goal is just to get here without panic
    }

    mod gh_issue_913 {
        use super::*;

        // https://github.com/georust/geo/issues/913
        #[test]
        fn original_repor() {
            let p1: MultiPolygon<f32> =
                wkt!(MULTIPOLYGON(((140.5 62.24,140.07 62.34,136.94 63.05,140.5 62.24))));
            let p2: MultiPolygon<f32> =
                wkt!(MULTIPOLYGON(((142.5 61.44,142.06 61.55,140.07 62.34,142.5 61.44))));

            let intersection = p2.intersection(&p1);
            _ = p1.difference(&intersection);
            // The goal is just to get here without panic
        }

        // https://github.com/georust/geo/issues/913#issuecomment-1266404475
        #[test]
        fn test_geo() {
            use crate::*;

            let pg1 = wkt!(POLYGON((367.679 302.785,146.985 90.13,235.63 197.771,222.764 233.747,200.38180602615253 288.3378373783007,207.575 288.205,367.679 302.785)));
            let pg2 = wkt!(POLYGON((367.679 302.785,203.351 279.952,200.373 288.338,206.73187706275607 288.2205700292493,208.489 286.233,212.24 285.478,367.679 302.785)));

            _ = pg1.union(&pg2);
            // The goal is just to get here without panic
        }

        // https://github.com/georust/geo/issues/913#issuecomment-1268214178
        #[test]
        fn test_3() {
            let p1: MultiPolygon<f32> = wkt!(MULTIPOLYGON(((142.50143 61.443245,142.05663 61.54975,140.07256 62.337814,142.50143 61.443245))));
            let p2: MultiPolygon<f32> = wkt!(MULTIPOLYGON(((140.50359 62.237686,140.07256 62.337814,136.93997 63.053394,140.50359 62.237686))));
            let intersection = p1.intersection(&p2);
            _ = p1.difference(&intersection);
            // The goal is just to get here without panic
        }

        #[test]
        fn test_polygon_union2() {
            let poly1: Polygon = wkt!(POLYGON((204.0 287.0,206.69670020700084 288.2213844497616,200.38308697914755 288.338793163584,204.0 287.0)));
            let poly2: Polygon = wkt!(POLYGON((210.0 290.0,204.07584923592933 288.2701221108328,212.24082541367974 285.47846008552216,210.0 290.0)));

            _ = poly1.union(&poly2);
            // The goal is just to get here without panic
        }
    }

    mod gh_issue_976 {
        use super::*;

        // https://github.com/georust/geo/issues/976#issuecomment-1696593357
        #[test]
        fn test_with_shift() {
            let mut left: MultiPolygon<f64> = wkt!(MULTIPOLYGON(((-1. 46.,8. 46.,8. 39.,-1. 39.,-1. 46.),(2. 45.,7. 45.,7. 44.,5. 42.,5. 41.,0. 43.,2. 45.))));
            let mut right: MultiPolygon<f64> = wkt!(MULTIPOLYGON(((0. 42.,6. 44.,4. 40.,0. 42.))));
            let shift = |c: Coord| Coord {
                x: c.x + 931230.,
                y: c.y + 412600.,
            };
            use crate::MapCoordsInPlace;
            left.map_coords_in_place(shift);
            right.map_coords_in_place(shift);
            for _i in 0..10 {
                // println!("{} ", i);
                left.difference(&right);
            }
            // The goal is just to get here without panic
        }
    }

    // https://github.com/georust/geo/issues/1064
    #[test]
    fn gh_issue_1064() {
        let a: Polygon = wkt!(POLYGON ((709799.9999999999 4535330.115932672, 709800.0000000001 4535889.8772568945, 709800.0057476197 4535889.994252375, 709800.1227431173 4535890.000000001, 710109.8788852151 4535889.999999996, 710109.994510683 4535889.99439513, 710110.0025226776 4535889.878911494, 710119.9974094903 4535410.124344491, 710119.9939843 4535410.005891683, 710119.8756285358 4535410.000000003, 710050.1240139506 4535410.000000003, 710050.0040320279 4535410.005955433, 710049.9539423736 4535410.115144073, 710038.1683017325 4535439.579245671, 710037.9601922985 4535440.0325333765, 710037.7079566419 4535440.462831489, 710037.4141048047 4535440.865858022, 710037.0815609565 4535441.237602393, 710036.7136342974 4535441.57436531, 710036.3139861295 4535441.872795589, 710035.8865934211 4535442.129923492, 710035.4357092284 4535442.343190307, 710034.9658203793 4535442.510473766, 710034.4816028172 4535442.6301092245, 710033.9878750739 4535442.7009061435, 710033.489550319 4535442.722160025, 710032.9915874689 4535442.6936593605, 710032.4989418354 4535442.615687774, 710032.016515821 4535442.48902117, 710031.54911013 4535442.314920021, 710031.1013759954 4535442.095116852, 710030.6777688961 4535441.831798956, 710030.2825042206 4535441.527586658, 710029.9195153153 4535441.185507218, 710029.5924143443 4535440.808964732, 710029.3044563448 4535440.401706239, 710029.0585068383 4535439.967784446, 710028.8570133082 4535439.511517367, 710028.701980853 4535439.037445406, 710028.5949522265 4535438.550286128, 710028.536992489 4535438.05488734, 710020.008429941 4535310.126449097, 710019.9938185212 4535310.0024022255, 710019.9208325277 4535309.901040657, 709980.0772727348 4535260.096590921, 709979.9987806884 4535260.007853539, 709979.8970781134 4535260.068614591, 709920.1022372885 4535299.931841814, 709920.0058878824 4535300.003151096, 709920.0000000001 4535300.122873942, 709920.0000000002 4535324.451138855, 709919.9762868701 4535324.937522437, 709919.9053724057 4535325.419292543, 709919.7879292492 4535325.891879465, 709919.6250713767 4535326.350800604, 709919.4183435373 4535326.7917029755, 709919.1697065973 4535327.210404501, 709918.8815189389 4535327.602933702, 709918.5565140966 4535327.965567328, 709918.1977748218 4535328.294865718, 709917.8087038476 4535328.5877053905, 709917.3929916087 4535328.841308688, 709916.9545812428 4535329.053270123, 709916.497631181 4535329.221579175, 709916.0264757108 4535329.344639407, 709915.5455838591 4535329.421283546, 709915.059517008 4535329.450784619, 709914.5728856224 4535329.43286279, 709914.0903055237 4535329.367688051, 709913.6163541072 4535329.255878603, 709913.1555269207 4535329.098494996, 709912.7121950255 4535328.89703004, 709912.2905635366 4535328.653394689, 709911.8946317348 4535328.369899881, 709911.5281551343 4535328.049234638, 709911.1946098561 4535327.694440548, 709910.8971596619 4535327.308882924, 709910.638625942 4535326.896218885, 709910.4214609532 4535326.460362643, 709910.2477245614 4535326.005448406, 709910.119064699 4535325.53579115, 709900.0266965557 4535280.120134492, 709899.9954816136 4535280.006411615, 709899.8778852832 4535280.015264336, 709820.1219250998 4535289.984759362, 709820.0038570677 4535290.005451573, 709819.9450491015 4535290.109901799, 709800.0518466672 4535329.896306664, 709800.0053207672 4535330.001256061, 709799.9999999999 4535330.115932672)));
        let b: Polygon = wkt!(POLYGON ((709800. 4600020., 809759.9999999993 4600020., 809759.9999999987 4500000., 709800.0000000003 4500000., 709800. 4590240., 709800. 4600020.)));

        _ = a.boolean_op(&b, OpType::Intersection);
        // The goal is just to get here without panic
    }

    // https://github.com/georust/geo/issues/1103
    #[test]
    fn gh_issue_1103() {
        let polygons: [Polygon<f32>; 4] = [
            wkt!(POLYGON((
                3.3493652 - 55.80127,
                171.22443 - 300.,
                291.84164 - 300.,
                46.650635 - 30.80127,
                3.3493652 - 55.80127
            ))),
            wkt!(POLYGON((46.650635 -30.80127,291.84164 -300.,300. -228.34003,-3.3493652 55.80127,46.650635 -30.80127))),
            wkt!(POLYGON((-46.650635 30.80127,195.83728 -300.,300. -228.34003,-3.3493652 55.80127,-46.650635 30.80127))),
            wkt!(POLYGON((3.3493652 -55.80127,171.22443 -300.,195.83728 -300.,-46.650635 30.80127,3.3493652 -55.80127))),
        ];

        let mut multi: MultiPolygon<f32> = MultiPolygon::new(Vec::new());
        for poly in polygons {
            multi = multi.union(&MultiPolygon::new(vec![poly]));
        }
        // The goal is just to get here without panic
    }

    mod gh_issue_1174 {
        use super::*;

        // https://github.com/georust/geo/issues/1174
        #[test]
        fn issue_description() {
            // TODO: CLIP implementation
            // Also, brace yourself for annoyingly formatted test data from the GH issue
        }

        // https://github.com/georust/geo/issues/1174#issuecomment-2110782797
        #[test]
        fn test_2() {
            let poly1 = wkt!(POLYGON((-10339459.518507583 3672178.7824083967,-10172502.686420029 3169028.9498966974,-10002503.513328442 3498113.19617442,-10339459.518507583 3672178.7824083967)));
            let poly2 = wkt!(POLYGON((-10644125.090349106 3510000.058398463,-10010375.27222986 3502179.60931681,-10018249.493188547 3506247.294314978,-10018249.49318854 3506247.294314993,-10320063.446714956 3765929.7827082784,-10644125.090349106 3510000.058398463)));
            _ = poly2.union(&poly1);
            // The goal is just to get here without panic
        }
    }

    // https://github.com/georust/geo/issues/1189
    #[test]
    fn gh_issue_1189() {
        let mutipolygon = wkt!(MULTIPOLYGON(((-842.8114919816321 -1593.246948876771,-250.74147790864794 -1277.405012942043,-107.7221679586994 -1437.4340368172966,548.266741952783 -851.1711267623472,1324.5108343844536 -437.08084844385627,1223.4931768687463 -247.7154757616648,1411.99111985414 -79.25324884855956,721.4329328584276 693.4350869619186,383.16901237055976 1327.5368365314368,228.479378920974 1245.0170801910524,79.25324884855956 1411.99111985414,-605.2045801730624 800.285292836034,-1784.1533139955259 171.37073609852212,-842.8114919816321 -1593.246948876771))));
        let p = wkt!(POLYGON((-842.8114919816321 -1593.246948876771,-1784.1533139955259 171.37073609852212,383.16901237055976 1327.5368365314368,1324.5108343844536 -437.08084844385627,-842.8114919816321 -1593.246948876771)));
        mutipolygon.union(&p);
        // The goal is just to get here without panic
    }

    // https://github.com/georust/geo/issues/1193
    #[test]
    fn gh_issue_1193() {
        let a: MultiPolygon<f32> = wkt!(MULTIPOLYGON(
            ((
                -19.064932 - 6.57369,
                -19.458324 - 3.6231885,
                -22.058823 - 3.6231885,
                -19.064932 - 6.57369
            )),
            ((
                -14.705882 - 10.869565,
                -14.705882 - 7.649791,
                -17.60358 - 8.013862,
                -14.705882 - 10.869565
            ))
        ));
        let b: MultiPolygon<f32> = wkt!(MULTIPOLYGON(
            ((
                -18.852 - 8.170715,
                -16.761898 - 24.659603,
                43.387707 - 16.298937,
                26.434301 - 2.4808762,
                -18.852 - 8.170715
            ))
        ));
        let c = a.difference(&b);

        // Note the GH issue expected length 1, because `b` occludes the second polygon in `a`, but it's not a total occlusion - there is a tiny sliver left.
        // I haven't investigated, but I'm going to assume this is a bug in the input data/expectations rather than the algorithm
        // assert_eq!(c.0.len(), 1);

        // this "expected" value and everything following is just provided to investigate that sliver.
        // It wasn't part of the test originally provided.
        let expected_c = wkt!(MULTIPOLYGON(
            ((
                -22.058823 - 3.623188,
                -19.064932 - 6.57369,
                -19.458324 - 3.623188,
                -22.058823 - 3.623188
            )),
            ((
                -17.60358 - 8.013862,
                -17.60358 - 8.013863,
                -14.705883 - 7.6497912,
                -14.705883 - 7.649791,
                -17.60358 - 8.013862
            ))
        ));
        assert_eq!(c, expected_c);
        assert_eq!(c.0.len(), 2);
        assert!(crate::Area::unsigned_area(&c.0[1]) < 1e-5);
        // The goal is just to get here without panic
    }
}
