use anyhow::{Context, Result, bail};
use geo::prelude::*;
use geo_booleanop::boolean::BooleanOp as OtherBOp;
use geo_types::*;
use geojson::{Feature, GeoJson};
use glob::glob;
use log::{error, info};

use serde_derive::Serialize;
use std::{
    convert::TryInto,
    error::Error,
    fs::{File, read_to_string},
    io::BufWriter,
    panic::{catch_unwind, resume_unwind},
    path::Path,
};

#[cfg(test)]
use geo_benches::utils::bops::*;

pub(super) fn init_log() {
    use pretty_env_logger::env_logger;
    use std::io::Write;
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "{} - {}", record.level(), record.args()))
        .try_init();
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

            debug!("p1: {p1:?}");
            debug!("p2: {p2:?}");
            fc.features
                .into_iter()
                .skip(2)
                .map(|feat| -> Result<_> {
                    let p = feature_as_geom(&feat)?;
                    let props = feat.properties.unwrap();
                    let ty = props["operation"]
                        .as_str()
                        .context("operation was not a string")?;
                    debug!("op: {ty} {p:?}");

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
                        debug!("ours: {geoms:?}");
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
                        debug!("theirs: {geoms:?}");
                        format!("{geoms:?}")
                    });
                    let theirs = their_result.unwrap_or_else(|_e| {
                        error!("theirs panicked");
                        "pannik".to_string()
                    });

                    let (comment, our_geom) = match result {
                        Ok(our_geom) => {
                            let diff = catch_unwind(|| p.difference(&our_geom));
                            let comment = match diff {
                                Ok(diff) => {
                                    debug!("difference: {diff:?}");
                                    if !diff.is_empty() {
                                        debug!("output was not identical:");
                                        debug!("\tours: {our_geom:?}");
                                        debug!("op: {ty} {p:?}");
                                        let area = diff.unsigned_area();
                                        let err = area / p.unsigned_area();
                                        debug!("\trel. error = {err}");
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
                        p1: format!("{p1:?}"),
                        p2: format!("{p2:?}"),
                        op: ty.to_string(),
                        ours: our_geom.map(|g| format!("{g:?}")).unwrap_or_default(),
                        expected: format!("{p:?}"),
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
