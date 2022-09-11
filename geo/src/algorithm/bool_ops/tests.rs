use crate::{Coordinate, LineString, MultiPolygon, Polygon};
use log::{error, info};

use std::{
    error::Error,
    panic::{catch_unwind, resume_unwind},
};
use wkt::{ToWkt, TryFromWkt};

pub(super) fn init_log() {
    use pretty_env_logger::env_logger;
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                record.level(),
                record.module_path().unwrap(),
                record.args()
            )
        })
        .try_init();
}

use super::*;
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn check_sweep(wkt1: &str, wkt2: &str, ty: OpType) -> Result<MultiPolygon<f64>> {
    init_log();
    eprintln!("input: 1: {wkt1}");
    eprintln!("input: 2: {wkt2}");
    let poly1 = MultiPolygon::<f64>::try_from_wkt_str(wkt1)
        .or_else(|_| Polygon::<f64>::try_from_wkt_str(wkt1).map(MultiPolygon::from))
        .unwrap();
    let poly2 = MultiPolygon::try_from_wkt_str(wkt2)
        .or_else(|_| Polygon::<f64>::try_from_wkt_str(wkt2).map(MultiPolygon::from))
        .unwrap();
    let mut bop = Proc::new(BoolOp::from(ty), 0);
    bop.add_multi_polygon(&poly1, 0);
    bop.add_multi_polygon(&poly2, 1);

    let geom = bop.sweep();

    info!("Got {n} rings", n = geom.0.len());
    info!("{wkt}", wkt = geom.to_wkt());

    Ok(geom)
}

#[test]
fn test_rect_overlapping() -> Result<()> {
    // Two rects that overlap
    let wkt1 = "POLYGON((0 0,1 0,1 1,0 1,0 0))";
    let wkt2 = "POLYGON((0.5 1,2 1,2 2,0.5 2,0.5 1))";

    let wkt_union = "MULTIPOLYGON(((1 0,1 1,2 1,2 2,0.5 2,0.5 1,0 1,0 0,1 0)))";
    let output = check_sweep(wkt1, wkt2, OpType::Union)?;
    assert_eq!(output, MultiPolygon::try_from_wkt_str(wkt_union).unwrap());
    Ok(())
}

#[test]
fn test_ext_in_hole() -> Result<()> {
    // A union which outputs a ring inside a hole inside a ext.
    let wkt1 = "POLYGON((0 0, 40 0, 40 40, 0 40, 0 0), (10 10, 30 10, 30 30, 10 30, 10 10))";
    let wkt2 = "POLYGON((11 11, 29 11, 29 29, 11 29, 11 11), (15 15, 25 15, 25 25, 15 25, 15 15))";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn test_invalid_simple() -> Result<()> {
    // Polygon with holes and invalid
    let wkt1 = "POLYGON((0 0, 2 2, 2 0, 0 0), (1 1, 2 1, 1 0))";
    let wkt2 = "POLYGON EMPTY";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn test_invalid_loops() -> Result<()> {
    let wkt1 = "POLYGON((0 0, 2 2, 0 4, -2 2, 0 0, 1 2, 0 3, -1 2, 0 0))";
    let wkt2 = "POLYGON EMPTY";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}
#[test]
fn test_complex_rects() -> Result<()> {
    let wkt1 = "MULTIPOLYGON(((-1 -2,-1.0000000000000002 2,-0.8823529411764707 2,-0.8823529411764706 -2,-1 -2)),((-0.7647058823529411 -2,-0.7647058823529412 2,-0.6470588235294118 2,-0.6470588235294118 -2,-0.7647058823529411 -2)),((-0.5294117647058824 -2,-0.5294117647058825 2,-0.41176470588235287 2,-0.4117647058823529 -2,-0.5294117647058824 -2)),((-0.2941176470588236 -2,-0.2941176470588236 2,-0.17647058823529418 2,-0.17647058823529416 -2,-0.2941176470588236 -2)),((-0.05882352941176472 -2,-0.05882352941176472 2,0.05882352941176472 2,0.05882352941176472 -2,-0.05882352941176472 -2)),((0.17647058823529416 -2,0.17647058823529416 2,0.29411764705882365 2,0.2941176470588236 -2,0.17647058823529416 -2)),((0.4117647058823528 -2,0.41176470588235287 2,0.5294117647058821 2,0.5294117647058822 -2,0.4117647058823528 -2)),((0.6470588235294117 -2,0.6470588235294118 2,0.7647058823529411 2,0.7647058823529411 -2,0.6470588235294117 -2)),((0.8823529411764706 -2,0.8823529411764707 2,1.0000000000000002 2,1 -2,0.8823529411764706 -2)))";
    let wkt2 = "MULTIPOLYGON(((-2 -1,2 -1.0000000000000002,2 -0.8823529411764707,-2 -0.8823529411764706,-2 -1)),((-2 -0.7647058823529411,2 -0.7647058823529412,2 -0.6470588235294118,-2 -0.6470588235294118,-2 -0.7647058823529411)),((-2 -0.5294117647058824,2 -0.5294117647058825,2 -0.41176470588235287,-2 -0.4117647058823529,-2 -0.5294117647058824)),((-2 -0.2941176470588236,2 -0.2941176470588236,2 -0.17647058823529418,-2 -0.17647058823529416,-2 -0.2941176470588236)),((-2 -0.05882352941176472,2 -0.05882352941176472,2 0.05882352941176472,-2 0.05882352941176472,-2 -0.05882352941176472)),((-2 0.17647058823529416,2 0.17647058823529416,2 0.29411764705882365,-2 0.2941176470588236,-2 0.17647058823529416)),((-2 0.4117647058823528,2 0.41176470588235287,2 0.5294117647058821,-2 0.5294117647058822,-2 0.4117647058823528)),((-2 0.6470588235294117,2 0.6470588235294118,2 0.7647058823529411,-2 0.7647058823529411,-2 0.6470588235294117)),((-2 0.8823529411764706,2 0.8823529411764707,2 1.0000000000000002,-2 1,-2 0.8823529411764706)))";

    let mp1 = MultiPolygon::<f64>::try_from_wkt_str(wkt1)?;
    let mp2 = MultiPolygon::<f64>::try_from_wkt_str(wkt2)?;

    for p1 in mp1.0.iter() {
        let p1 = MultiPolygon::from(p1.clone());
        for p2 in mp2.0.iter() {
            let p2 = MultiPolygon::from(p2.clone());
            let result = catch_unwind(|| -> Result<()> {
                check_sweep(&p1.wkt_string(), &p2.wkt_string(), OpType::Union)?;
                Ok(())
            });
            if let Err(ee) = result {
                error!("p1: {wkt}", wkt = p1.wkt_string());
                error!("p2: {wkt}", wkt = p2.wkt_string());
                resume_unwind(ee);
            }
        }
    }
    Ok(())
}
#[test]
fn test_complex_rects1() -> Result<()> {
    let wkt1 = "MULTIPOLYGON(((-1 -2,-1.0000000000000002 2,-0.8823529411764707 2,-0.8823529411764706 -2,-1 -2)))";
    let wkt2 = "MULTIPOLYGON(((-2 -1,2 -1.0000000000000002,2 -0.8823529411764707,-2 -0.8823529411764706,-2 -1)))";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn test_overlap_issue_867() -> Result<()> {
    let wkt1 = "POLYGON ((17.724912058920285 -16.37118892052372, 18.06452454246989 -17.693907532504, 19.09389292605319 -17.924001641855178, 17.724912058920285 -16.37118892052372))";
    let wkt2 = "POLYGON ((17.576085274796423 -15.791540153598898, 17.19432983818328 -17.499393422066746, 18.06452454246989 -17.693907532504, 17.576085274796423 -15.791540153598898))";
    check_sweep(wkt1, wkt2, OpType::Intersection)?;
    Ok(())

    // Instructive example of line-intersection fp considerations.
    //     let l1 = Line::new(
    //         coord!(x: 17.576085274796423, y: -15.791540153598898),
    //         coord!(x: 18.06452454246989, y: -17.693907532504),
    //     );

    //     let l2 = Line::new(
    //         coord!(x: 17.724912058920285, y: -16.37118892052372),
    //         coord!(x: 18.06452454246989, y: -17.693907532504),
    //     );
    //     let l3 = Line::new(
    //         coord!(x: 17.724912058920285, y: -16.37118892052372),
    //         coord!(x: 19.09389292605319, y: -17.924001641855178),
    //     );

    //     let i12 = line_intersection(l1, l2);
    //     eprintln!("l1 x l2 = {i12:?}");
    //     let i13 = line_intersection(l1, l3);
    //     eprintln!("l1 x l3 = {i13:?}");
}

#[test]
fn test_issue_865() -> Result<()> {
    // Simplified example
    let wkt1 = "POLYGON((0 0, 1 1, 10 0, 0 0))";
    let wkt2 = "POLYGON((1 1, 8 2, 4 1, 1 1))";
    let wkt2d = "POLYGON((1 1, 4 2, 8 1, 1 1))";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    check_sweep(wkt1, wkt2d, OpType::Union)?;

    // Original Example
    let wkt1 = "POLYGON((-640 -360,640 -360,640 360,-640 360,-640 -360))";
    let wkt2 = "POLYGON((313.276 359.999,213.319 359.999,50 60,-50 60,-50 110,-8.817 360,-93.151 360,-85.597 225.618,-114.48 359.999,-117.017 360,-85 215,-85 155,-115 155,-154.161 360,-640 360,-640 -360,640 -360,640 360,313.277 360,313.276 359.999))";
    check_sweep(wkt1, wkt2, OpType::Difference)?;
    Ok(())
}

#[test]
fn test_clip_adhoc() -> Result<()> {
    let wkt1 = "POLYGON ((20 0, 20 160, 200 160, 200 0, 20 0))";
    let wkt2 = "LINESTRING (0 0, 100 100, 240 100)";

    let poly1 = MultiPolygon::<f64>::try_from_wkt_str(wkt1)
        .or_else(|_| Polygon::<f64>::try_from_wkt_str(wkt1).map(MultiPolygon::from))
        .unwrap();
    let mls = MultiLineString::try_from_wkt_str(wkt2)
        .or_else(|_| LineString::<f64>::try_from_wkt_str(wkt2).map(MultiLineString::from))
        .unwrap();
    let output = poly1.clip(&mls, true);
    eprintln!("{wkt}", wkt = output.to_wkt());
    Ok(())
}

#[test]
fn test_issue_885() -> Result<()> {
    init_log();
    type Polygon = geo_types::Polygon<f32>;
    let p1 = Polygon::new(
        LineString(vec![
            Coordinate {
                x: 8055.658,
                y: 7977.5537,
            },
            Coordinate {
                x: 8010.734,
                y: 7999.9697,
            },
            Coordinate {
                x: 8032.9717,
                y: 8044.537,
            },
            Coordinate {
                x: 8077.896,
                y: 8022.121,
            },
            Coordinate {
                x: 8055.658,
                y: 7977.5537,
            },
        ]),
        vec![],
    );
    let p2 = Polygon::new(
        LineString(vec![
            Coordinate {
                x: 8055.805,
                y: 7977.847,
            },
            Coordinate {
                x: 8010.871,
                y: 8000.2676,
            },
            Coordinate {
                x: 8033.105,
                y: 8044.8286,
            },
            Coordinate {
                x: 8078.039,
                y: 8022.408,
            },
            Coordinate {
                x: 8055.805,
                y: 7977.847,
            },
        ]),
        vec![],
    );
    eprintln!("{p1}", p1 = p1.to_wkt());
    eprintln!("{p2}", p2 = p2.to_wkt());
    let _union = p1.union(&p2);
    let _isect = p1.intersection(&p2);
    Ok(())
}

#[test]
#[ignore]
fn test_issue_885_big() {
    init_log();
    let a = MultiPolygon::new(vec![Polygon::new(
        LineString::new(vec![
            Coordinate {
                x: 256.0,
                y: -256.0,
            },
            Coordinate {
                x: -256.0,
                y: -256.0,
            },
            Coordinate {
                x: -256.0,
                y: 256.0,
            },
            Coordinate { x: 256.0, y: 256.0 },
            Coordinate {
                x: 256.0,
                y: -256.0,
            },
        ]),
        vec![
            LineString::new(vec![
                Coordinate {
                    x: 21.427018453144548,
                    y: 100.2247496417564,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 97.449462890625,
                },
                Coordinate {
                    x: 21.255590787413905,
                    y: 95.86452640008609,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 92.449462890625,
                },
                Coordinate {
                    x: 21.255635468249327,
                    y: 90.86460378956318,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 87.44970703125,
                },
                Coordinate {
                    x: 21.255590787413905,
                    y: 85.86477054071109,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 82.44970703125,
                },
                Coordinate {
                    x: 21.255590787413905,
                    y: 80.86477054071109,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 77.44970703125,
                },
                Coordinate {
                    x: 21.255635468249327,
                    y: 75.86484793018818,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 72.449951171875,
                },
                Coordinate {
                    x: 21.255590787413905,
                    y: 70.86501468133609,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 67.449951171875,
                },
                Coordinate {
                    x: 21.255590787413905,
                    y: 65.86501468133609,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 62.449951171875,
                },
                Coordinate {
                    x: 21.255635468249327,
                    y: 60.865092070813176,
                },
                Coordinate {
                    x: 22.170654296875,
                    y: 57.4501953125,
                },
                Coordinate {
                    x: 20.964468977514716,
                    y: 55.3610210560243,
                },
                Coordinate {
                    x: 21.7110595703125,
                    y: 52.57470703125,
                },
                Coordinate {
                    x: 6.7110595703125036,
                    y: 26.593944917716843,
                },
                Coordinate {
                    x: 4.94307055353958,
                    y: 26.120213688445443,
                },
                Coordinate {
                    x: 3.1754150390625036,
                    y: 23.058544527091843,
                },
                Coordinate {
                    x: 1.4077371841677144,
                    y: 22.584896673394404,
                },
                Coordinate {
                    x: -0.36010742187499645,
                    y: 19.522899995841843,
                },
                Coordinate {
                    x: -2.127952027917708,
                    y: 19.04920746130898,
                },
                Coordinate {
                    x: -3.8956298828124964,
                    y: 15.987499605216843,
                },
                Coordinate {
                    x: -5.663474488855209,
                    y: 15.513807070683983,
                },
                Coordinate {
                    x: -7.4311523437499964,
                    y: 12.452099214591843,
                },
                Coordinate {
                    x: -9.199141360522919,
                    y: 11.978367985320444,
                },
                Coordinate {
                    x: -10.966796874999996,
                    y: 8.916698823966843,
                },
                Coordinate {
                    x: -12.734474729894785,
                    y: 8.443050970269406,
                },
                Coordinate {
                    x: -14.502319335937496,
                    y: 5.381054292716843,
                },
                Coordinate {
                    x: -29.502319335937507,
                    y: 1.36181640625,
                },
                Coordinate {
                    x: -44.502319335937514,
                    y: 5.381054292716847,
                },
                Coordinate {
                    x: -55.48308144947066,
                    y: 16.361816406249996,
                },
                Coordinate {
                    x: -59.5023193359375,
                    y: 31.361816406250004,
                },
                Coordinate {
                    x: -55.48308144947066,
                    y: 46.36181640625,
                },
                Coordinate {
                    x: -51.94760366936858,
                    y: 49.89729418635208,
                },
                Coordinate {
                    x: -51.94755898853316,
                    y: 49.8974609375,
                },
                Coordinate {
                    x: -40.96679687499999,
                    y: 60.87822305103316,
                },
                Coordinate {
                    x: -40.96646337270415,
                    y: 60.878312412704005,
                },
                Coordinate {
                    x: -37.43115234374999,
                    y: 64.41362344165816,
                },
                Coordinate {
                    x: -37.43098559260209,
                    y: 64.41366812249358,
                },
                Coordinate {
                    x: -37.25638533366225,
                    y: 64.58826838143341,
                },
                Coordinate {
                    x: -37.15947272204719,
                    y: 64.949951171875,
                },
                Coordinate {
                    x: -37.829345703125,
                    y: 67.449951171875,
                },
                Coordinate {
                    x: -37.15947272204719,
                    y: 69.949951171875,
                },
                Coordinate {
                    x: -37.829345703125,
                    y: 72.449951171875,
                },
                Coordinate {
                    x: -37.159505430688846,
                    y: 74.9498291015625,
                },
                Coordinate {
                    x: -37.829345703125,
                    y: 77.44970703125,
                },
                Coordinate {
                    x: -37.15947272204719,
                    y: 79.94970703125,
                },
                Coordinate {
                    x: -37.829345703125,
                    y: 82.44970703125,
                },
                Coordinate {
                    x: -37.15947272204719,
                    y: 84.94970703125,
                },
                Coordinate {
                    x: -37.829345703125,
                    y: 87.44970703125,
                },
                Coordinate {
                    x: -37.6218793727004,
                    y: 88.22398191725448,
                },
                Coordinate {
                    x: -43.845336914062514,
                    y: 89.89155233959184,
                },
                Coordinate {
                    x: -54.82609902759566,
                    y: 100.872314453125,
                },
                Coordinate {
                    x: -58.8453369140625,
                    y: 115.872314453125,
                },
                Coordinate {
                    x: -54.82609902759566,
                    y: 130.872314453125,
                },
                Coordinate {
                    x: -43.84533691406249,
                    y: 141.85307656665816,
                },
                Coordinate {
                    x: -28.845336914062496,
                    y: 145.872314453125,
                },
                Coordinate {
                    x: -26.345275878906246,
                    y: 145.20242511772636,
                },
                Coordinate {
                    x: -23.845214843749996,
                    y: 145.872314453125,
                },
                Coordinate {
                    x: -21.345153808593746,
                    y: 145.20242511772636,
                },
                Coordinate {
                    x: -18.845092773437496,
                    y: 145.872314453125,
                },
                Coordinate {
                    x: -16.345092773437496,
                    y: 145.20244147204718,
                },
                Coordinate {
                    x: -13.845092773437498,
                    y: 145.872314453125,
                },
                Coordinate {
                    x: -11.345092773437498,
                    y: 145.20244147204718,
                },
                Coordinate {
                    x: -8.845092773437498,
                    y: 145.872314453125,
                },
                Coordinate {
                    x: -7.241160801189184,
                    y: 145.4425421764466,
                },
                Coordinate {
                    x: -4.753417968749998,
                    y: 146.109130859375,
                },
                Coordinate {
                    x: -3.857349940998309,
                    y: 145.86903015497558,
                },
                Coordinate {
                    x: -3.8450927734374982,
                    y: 145.872314453125,
                },
                Coordinate {
                    x: -2.5414695567288628,
                    y: 145.52300966497347,
                },
                Coordinate {
                    x: -0.04333496093749816,
                    y: 146.1923828125,
                },
                Coordinate {
                    x: 14.956665039062504,
                    y: 142.17314492603316,
                },
                Coordinate {
                    x: 14.96700942550945,
                    y: 142.16280053958621,
                },
                Coordinate {
                    x: 15.549804687500004,
                    y: 142.00664101978316,
                },
                Coordinate {
                    x: 26.53056680103316,
                    y: 131.02587890625,
                },
                Coordinate {
                    x: 30.5498046875,
                    y: 116.02587890625,
                },
                Coordinate {
                    x: 21.427018453144548,
                    y: 100.2247496417564,
                },
            ]),
            LineString::new(vec![
                Coordinate {
                    x: 15.9573974609375,
                    y: 218.888916015625,
                },
                Coordinate {
                    x: 11.93815957447066,
                    y: 233.888916015625,
                },
                Coordinate {
                    x: 0.9573974609375036,
                    y: 244.86967812915816,
                },
                Coordinate {
                    x: -14.042602539062498,
                    y: 248.888916015625,
                },
                Coordinate {
                    x: -29.042602539062493,
                    y: 244.86967812915816,
                },
                Coordinate {
                    x: -32.57795824885207,
                    y: 241.3343224193686,
                },
                Coordinate {
                    x: -32.57812499999999,
                    y: 241.33427773853316,
                },
                Coordinate {
                    x: -36.11348070978957,
                    y: 237.7989220287436,
                },
                Coordinate {
                    x: -36.11364746093749,
                    y: 237.79887734790816,
                },
                Coordinate {
                    x: -39.64895848989165,
                    y: 234.26356631895402,
                },
                Coordinate {
                    x: -39.64929199218749,
                    y: 234.26347695728316,
                },
                Coordinate {
                    x: -50.63005410572066,
                    y: 223.28271484375,
                },
                Coordinate {
                    x: -50.63009878655608,
                    y: 223.28254809260207,
                },
                Coordinate {
                    x: -54.16557656665816,
                    y: 219.7470703125,
                },
                Coordinate {
                    x: -58.184814453125,
                    y: 204.7470703125,
                },
                Coordinate {
                    x: -54.16557656665816,
                    y: 189.7470703125,
                },
                Coordinate {
                    x: -43.184814453125014,
                    y: 178.76630819896684,
                },
                Coordinate {
                    x: -28.184814453125007,
                    y: 174.7470703125,
                },
                Coordinate {
                    x: -13.184814453124996,
                    y: 178.76630819896684,
                },
                Coordinate {
                    x: -11.416969847082285,
                    y: 181.8283048765194,
                },
                Coordinate {
                    x: -9.649291992187496,
                    y: 182.30195273021684,
                },
                Coordinate {
                    x: -7.88163647771042,
                    y: 185.36362189157043,
                },
                Coordinate {
                    x: -6.1136474609374964,
                    y: 185.83735312084184,
                },
                Coordinate {
                    x: -4.345969606042708,
                    y: 188.89906097693398,
                },
                Coordinate {
                    x: -2.5781249999999964,
                    y: 189.37275351146684,
                },
                Coordinate {
                    x: -0.8104471451052081,
                    y: 192.43446136755898,
                },
                Coordinate {
                    x: 0.9573974609375036,
                    y: 192.90815390209184,
                },
                Coordinate {
                    x: 15.9573974609375,
                    y: 218.888916015625,
                },
            ]),
        ],
    )]);
    let b = Polygon::new(
        LineString::new(vec![
            Coordinate {
                x: 19.492919921875,
                y: 222.424560546875,
            },
            Coordinate {
                x: 15.47368203540816,
                y: 237.424560546875,
            },
            Coordinate {
                x: 4.4929199218750036,
                y: 248.40532266040816,
            },
            Coordinate {
                x: -10.507080078124998,
                y: 252.424560546875,
            },
            Coordinate {
                x: -25.507080078124993,
                y: 248.40532266040816,
            },
            Coordinate {
                x: -36.48784219165816,
                y: 237.424560546875,
            },
            Coordinate {
                x: -40.507080078125,
                y: 222.424560546875,
            },
            Coordinate {
                x: -36.48784219165816,
                y: 207.424560546875,
            },
            Coordinate {
                x: -25.507080078125014,
                y: 196.44379843334184,
            },
            Coordinate {
                x: -10.507080078125005,
                y: 192.424560546875,
            },
            Coordinate {
                x: 4.4929199218750036,
                y: 196.44379843334184,
            },
            Coordinate {
                x: 19.492919921875,
                y: 222.424560546875,
            },
        ]),
        vec![],
    );

    eprintln!("input: {p1}", p1 = a.to_wkt());
    eprintln!("input: {p2}", p2 = b.to_wkt());
    a.difference(&MultiPolygon::new(vec![b]));
}

#[test]
fn test_issue_885_big_simplified() -> Result<()> {
    let wkt1 = r#"MULTIPOLYGON(((256.0 -256.0,-256.0 -256.0,-256.0 256.0,256.0 256.0,256.0 -256.0), (15.9573974609375 218.888916015625,-29.042602539062493 244.86967812915816, -32.57795824885207 241.3343224193686, -32.57812499999999 241.33427773853316, -36.11348070978957 237.7989220287436,-36.11364746093749 237.79887734790816, -39.64895848989165 234.26356631895402,0.9573974609375036 192.90815390209184,15.9573974609375 218.888916015625)))"#;
    let wkt2 = r#"POLYGON((19.492919921875 222.424560546875,-25.507080078124993 248.40532266040816,-36.48784219165816 237.424560546875, 4.4929199218750036 196.44379843334184,19.492919921875 222.424560546875))"#;
    check_sweep(wkt1, wkt2, OpType::Difference)?;
    Ok(())
}

mod simplify;
