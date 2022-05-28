use crate::{dimensions::HasDimensions, prelude::Area, Geometry, MultiPolygon, Polygon, Rect};
use anyhow::{bail, Context, Result};
use geo_booleanop::boolean::BooleanOp as OtherBOp;
use geojson::{Feature, GeoJson};
use glob::glob;
use log::{error, info};
use rand::thread_rng;
use serde_derive::Serialize;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fs::{read_to_string, File},
    io::{stdout, BufWriter},
    panic::{catch_unwind, resume_unwind},
    path::{Path, PathBuf},
};
use wkt::{ToWkt, TryFromWkt};

#[cfg(test)]
#[path = "../../../benches/utils/bops.rs"]
pub mod bops_utils;

use bops_utils::*;

pub(super) fn init_log() {
    use pretty_env_logger::env_logger;
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
        .try_init();
}

use super::*;

#[test]
fn test_rect_overlapping() -> Result<()> {
    // Two rects that overlap
    let wkt1 = "POLYGON((0 0,1 0,1 1,0 1,0 0))";
    let wkt2 = "POLYGON((0.5 1,2 1,2 2,0.5 2,0.5 1))";

    let wkt_union = "MULTIPOLYGON(((2 1,1 1,1 0,0 0,0 1,0.5 1,0.5 2,2 2,2 1)))";
    let output = check_sweep(wkt1, wkt2, OpType::Union)?;
    assert_eq!(output, MultiPolygon::try_from_wkt_str(wkt_union).unwrap());
    Ok(())
}

#[test]
fn test_ext_in_hole() -> Result<(), Box<dyn Error>> {
    // A union which outputs a ring inside a hole inside a ext.
    let wkt1 = "POLYGON((0 0, 40 0, 40 40, 0 40, 0 0), (10 10, 30 10, 30 30, 10 30, 10 10))";
    let wkt2 = "POLYGON((11 11, 29 11, 29 29, 11 29, 11 11), (15 15, 25 15, 25 25, 15 25, 15 15))";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn test_invalid_simple() -> Result<(), Box<dyn Error>> {
    // Polygon with holes and invalid
    let wkt1 = "POLYGON((0 0, 2 2, 2 0, 0 0), (1 1, 2 1, 1 0))";
    let wkt2 = "POLYGON EMPTY";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn test_invalid_loops() -> Result<(), Box<dyn Error>> {
    let wkt1 = "POLYGON((0 0, 2 2, 0 4, -2 2, 0 0, 1 2, 0 3, -1 2, 0 0))";
    let wkt2 = "POLYGON EMPTY";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

fn check_sweep(wkt1: &str, wkt2: &str, ty: OpType) -> Result<MultiPolygon<f64>> {
    init_log();
    let poly1 = MultiPolygon::<f64>::try_from_wkt_str(wkt1)
        .or_else(|_| Polygon::<f64>::try_from_wkt_str(wkt1).map(|p| MultiPolygon::from(p)))
        .unwrap();
    let poly2 = MultiPolygon::try_from_wkt_str(wkt2)
        .or_else(|_| Polygon::<f64>::try_from_wkt_str(wkt2).map(|p| MultiPolygon::from(p)))
        .unwrap();
    let mut bop = Op::new(ty, 0);
    bop.add_multi_polygon(&poly1, true);
    bop.add_multi_polygon(&poly2, false);

    let rings = bop.sweep();
    info!("Got {n} rings", n = rings.len());
    for ring in rings.iter() {
        info!(
            "\t{hole}: {wkt}",
            wkt = ring.coords().to_wkt(),
            hole = if ring.is_hole() { "HOLE" } else { "EXTR" }
        );
    }

    let polygons = assemble(rings);
    info!("got {n} output polygons", n = polygons.len());
    for p in polygons.iter() {
        info!("\t{wkt}", wkt = p.to_wkt());
    }
    Ok(MultiPolygon::new(polygons))
}

#[test]
fn test_complex_rects() -> Result<(), Box<dyn Error>> {
    let wkt1 = "MULTIPOLYGON(((-1 -2,-1.0000000000000002 2,-0.8823529411764707 2,-0.8823529411764706 -2,-1 -2)),((-0.7647058823529411 -2,-0.7647058823529412 2,-0.6470588235294118 2,-0.6470588235294118 -2,-0.7647058823529411 -2)),((-0.5294117647058824 -2,-0.5294117647058825 2,-0.41176470588235287 2,-0.4117647058823529 -2,-0.5294117647058824 -2)),((-0.2941176470588236 -2,-0.2941176470588236 2,-0.17647058823529418 2,-0.17647058823529416 -2,-0.2941176470588236 -2)),((-0.05882352941176472 -2,-0.05882352941176472 2,0.05882352941176472 2,0.05882352941176472 -2,-0.05882352941176472 -2)),((0.17647058823529416 -2,0.17647058823529416 2,0.29411764705882365 2,0.2941176470588236 -2,0.17647058823529416 -2)),((0.4117647058823528 -2,0.41176470588235287 2,0.5294117647058821 2,0.5294117647058822 -2,0.4117647058823528 -2)),((0.6470588235294117 -2,0.6470588235294118 2,0.7647058823529411 2,0.7647058823529411 -2,0.6470588235294117 -2)),((0.8823529411764706 -2,0.8823529411764707 2,1.0000000000000002 2,1 -2,0.8823529411764706 -2)))";
    let wkt2 = "MULTIPOLYGON(((-2 -1,2 -1.0000000000000002,2 -0.8823529411764707,-2 -0.8823529411764706,-2 -1)),((-2 -0.7647058823529411,2 -0.7647058823529412,2 -0.6470588235294118,-2 -0.6470588235294118,-2 -0.7647058823529411)),((-2 -0.5294117647058824,2 -0.5294117647058825,2 -0.41176470588235287,-2 -0.4117647058823529,-2 -0.5294117647058824)),((-2 -0.2941176470588236,2 -0.2941176470588236,2 -0.17647058823529418,-2 -0.17647058823529416,-2 -0.2941176470588236)),((-2 -0.05882352941176472,2 -0.05882352941176472,2 0.05882352941176472,-2 0.05882352941176472,-2 -0.05882352941176472)),((-2 0.17647058823529416,2 0.17647058823529416,2 0.29411764705882365,-2 0.2941176470588236,-2 0.17647058823529416)),((-2 0.4117647058823528,2 0.41176470588235287,2 0.5294117647058821,-2 0.5294117647058822,-2 0.4117647058823528)),((-2 0.6470588235294117,2 0.6470588235294118,2 0.7647058823529411,-2 0.7647058823529411,-2 0.6470588235294117)),((-2 0.8823529411764706,2 0.8823529411764707,2 1.0000000000000002,-2 1,-2 0.8823529411764706)))";

    let mp1 = MultiPolygon::<f64>::try_from_wkt_str(&wkt1)?;
    let mp2 = MultiPolygon::<f64>::try_from_wkt_str(&wkt2)?;

    for p1 in mp1.0.iter() {
        let p1 = MultiPolygon::from(p1.clone());
        for p2 in mp2.0.iter() {
            let p2 = MultiPolygon::from(p2.clone());
            let result = catch_unwind(|| -> Result<()> {
                check_sweep(&p1.wkt_string(), &p2.wkt_string(), OpType::Union)?;
                Ok(())
            });
            if result.is_err() {
                error!("p1: {wkt}", wkt = p1.wkt_string());
                error!("p2: {wkt}", wkt = p2.wkt_string());
                resume_unwind(result.unwrap_err());
            }
        }
    }
    Ok(())
}
#[test]
fn test_complex_rects1() -> Result<(), Box<dyn Error>> {
    let wkt1 = "MULTIPOLYGON(((-1 -2,-1.0000000000000002 2,-0.8823529411764707 2,-0.8823529411764706 -2,-1 -2)))";
    let wkt2 = "MULTIPOLYGON(((-2 -1,2 -1.0000000000000002,2 -0.8823529411764707,-2 -0.8823529411764706,-2 -1)))";
    check_sweep(wkt1, wkt2, OpType::Union)?;
    Ok(())
}

#[test]
fn generate_ds() -> Result<(), Box<dyn Error>> {
    init_log();

    let proj_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut cases = vec![];
    for fix in glob(&format!(
        "{proj_path}/fixtures/rust-geo-booleanop-fixtures/**/*.geojson"
    ))? {
        let fix = fix?;
        info!("Running fixture {fix}...", fix = fix.display());
        match try_run_fixture(&fix) {
            Ok(c) => {
                info!("\tfixture succeeded.  Got {n} cases", n = c.len());
                cases.extend(c)
            }
            Err(e) => {
                error!("error running fixture: {fix:?}");
                error!("\t{e}");
            }
        }
    }
    #[derive(Serialize)]
    struct TestCase {
        p1: String,
        p2: String,
        op: String,
        expected: String,
        ours: String,
        theirs: String,
        comment: String,
    }

    fn panic_message(e: Box<dyn std::any::Any + Send + 'static>) -> String {
        e.downcast::<String>().map(|b| *b).unwrap_or_else(|e| {
            e.downcast::<&str>()
                .map(|b| b.to_string())
                .unwrap_or_else(|e| resume_unwind(e))
        })
    }

    fn try_run_fixture(fix: &Path) -> Result<Vec<TestCase>> {
        let data = read_to_string(fix)?;
        let gjson: GeoJson = data.parse()?;
        let cases = if let GeoJson::FeatureCollection(fc) = gjson {
            if fc.features.len() <= 2 {
                return Ok(vec![]);
            }
            let p1 = feature_as_geom(&fc.features[0]).context("geom 1 not readable")?;
            let p2 = feature_as_geom(&fc.features[1]).context("geom 2 not readable")?;

            let prev_p1 = convert_mpoly(&p1);
            let prev_p2 = convert_mpoly(&p2);

            info!("p1: {wkt}", wkt = p1.to_wkt());
            info!("p2: {wkt}", wkt = p2.to_wkt());
            fc.features
                .into_iter()
                .skip(2)
                .map(|feat| -> Result<_> {
                    let p = feature_as_geom(&feat)?;
                    let props = feat.properties.unwrap();
                    let ty = props["operation"]
                        .as_str()
                        .context("operation was not a string")?;
                    info!("op: {ty} {wkt}", wkt = p.to_wkt(),);

                    let result = catch_unwind(|| {
                        let geoms = if ty == "intersection" {
                            p1.intersection(&p2)
                        } else if ty == "union" {
                            p1.union(&p2)
                        } else if ty == "xor" {
                            p1.xor(&p2)
                        } else if ty == "diff" {
                            p1.difference(&p2)
                        } else if ty == "diff_ba" {
                            p2.difference(&p1)
                        } else {
                            error!("unexpected op: {ty}");
                            unreachable!()
                        };
                        info!("ours: {wkt}", wkt = geoms.to_wkt());
                        geoms
                    });

                    let their_result = catch_unwind(|| {
                        let geoms = if ty == "intersection" {
                            prev_p1.intersection(&prev_p2)
                        } else if ty == "union" {
                            prev_p1.union(&prev_p2)
                        } else if ty == "xor" {
                            prev_p1.xor(&prev_p2)
                        } else if ty == "diff" {
                            prev_p1.difference(&prev_p2)
                        } else if ty == "diff_ba" {
                            prev_p2.difference(&prev_p1)
                        } else {
                            error!("unexpected op: {ty}");
                            unreachable!()
                        };
                        let geoms = convert_back_mpoly(&geoms);
                        let wkt = geoms.wkt_string();
                        info!("theirs: {wkt}");
                        wkt
                    });
                    let theirs = their_result.unwrap_or_else(|e| {
                        error!("theirs panicked");
                        "pannik".to_string()
                    });

                    let (comment, our_geom) = match result {
                        Ok(our_geom) => {
                            let diff = catch_unwind(|| p.difference(&our_geom));
                            let comment = match diff {
                                Ok(diff) => {
                                    info!("difference: {wkt}", wkt = diff.to_wkt());
                                    if !diff.is_empty() {
                                        info!("output was not identical:");
                                        info!("\tours: {wkt}", wkt = our_geom.wkt_string());
                                        info!("op: {ty} {wkt}", wkt = p.to_wkt(),);
                                        let area = diff.unsigned_area();
                                        let err = area / p.unsigned_area();
                                        info!("\trel. error = {err}");
                                        format!("relerr: {err:.2}")
                                    } else {
                                        "identical".to_string()
                                    }
                                }
                                Err(e) => {
                                    let msg = panic_message(e);
                                    error!("diff-compute panicked: {msg}!");
                                    format!("diff-panic: {msg}")
                                }
                            };
                            (comment, Some(our_geom))
                        }
                        Err(e) => {
                            let msg = panic_message(e);
                            error!("compute panicked: {msg}!");
                            (format!("panic: {msg}"), None)
                        }
                    };

                    Ok(TestCase {
                        p1: p1.wkt_string(),
                        p2: p2.wkt_string(),
                        op: ty.to_string(),
                        ours: our_geom.map(|g| g.wkt_string()).unwrap_or_default(),
                        expected: p.wkt_string(),
                        comment,
                        theirs,
                    })
                })
                .collect::<Result<_>>()?
        } else {
            unreachable!()
        };
        Ok(cases)
    }

    let file = File::create("rust-geo-booleanop-fixtures.json")?;
    serde_json::to_writer(BufWriter::new(file), &cases)?;
    Ok(())
}

fn feature_as_geom(feat: &Feature) -> Result<MultiPolygon<f64>> {
    let p: Geometry<f64> = feat
        .geometry
        .clone()
        .context("missing geometry in feature")?
        .try_into()
        .context("could not parse feature as geometry")?;
    Ok(match p {
        Geometry::Polygon(p) => p.into(),
        Geometry::MultiPolygon(p) => p,
        _ => bail!("unexpected type of geometry"),
    })
}
