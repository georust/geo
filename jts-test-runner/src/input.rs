use geo::bool_ops::OpType as BoolOp;
use geo::relate::IntersectionMatrix;
use geo::{Geometry, Point};
use serde::{Deserialize, Deserializer};

use super::Result;

/// Example of the XML that these structures represent
///
/// ```xml
/// <run>
/// <precisionModel scale="1.0" offsetx="0.0" offsety="0.0"/>
///
/// <case>
/// <desc>AA disjoint</desc>
/// <a>
/// POLYGON(
/// (0 0, 80 0, 80 80, 0 80, 0 0))
/// </a>
/// <b>
/// POLYGON(
/// (100 200, 100 140, 180 140, 180 200, 100 200))
/// </b>
/// <test><op name="relate" arg3="FF2FF1212" arg1="A" arg2="B"> true </op>
/// </test>
/// <test>  <op name="intersects" arg1="A" arg2="B">   false   </op></test>
/// <test>  <op name="contains" arg1="A" arg2="B">   false   </op></test>
/// </case>
/// </run>
/// ```
#[derive(Debug, Deserialize)]
pub(crate) struct Run {
    #[serde(rename = "precisionModel", default)]
    pub precision_model: Option<PrecisionModel>,

    #[serde(rename = "case")]
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PrecisionModel {
    #[serde(rename = "type", default)]
    pub ty: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Case {
    #[serde(default)]
    pub(crate) desc: String,

    #[serde(deserialize_with = "deserialize_lenient_geometry", default)]
    pub(crate) a: Option<Geometry>,

    #[serde(deserialize_with = "deserialize_lenient_geometry", default)]
    pub(crate) b: Option<Geometry>,

    #[serde(rename = "test", default)]
    pub(crate) tests: Vec<Test>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Test {
    #[serde(rename = "op")]
    pub(crate) operation_input: OperationInput,
}

#[derive(Debug, Deserialize)]
pub struct CentroidInput {
    pub(crate) arg1: String,

    #[serde(rename = "$value", deserialize_with = "wkt::deserialize_point")]
    pub(crate) expected: Option<geo::Point>,
}

#[derive(Debug, Deserialize)]
pub struct ConvexHullInput {
    pub(crate) arg1: String,

    #[serde(rename = "$value", deserialize_with = "wkt::deserialize_wkt")]
    pub(crate) expected: geo::Geometry,
}

#[derive(Debug, Deserialize)]
pub struct EqualsTopoInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool,
}

#[derive(Debug, Deserialize)]
pub struct IntersectsInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool,
}

#[derive(Debug, Deserialize)]
pub struct IsValidInput {
    pub(crate) arg1: String,

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool,
}

#[derive(Debug, Deserialize)]
pub struct RelateInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "arg3", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: IntersectionMatrix,
}

#[derive(Debug, Deserialize)]
pub struct BufferInput {
    pub(crate) arg1: String,
    #[serde(rename = "arg2", deserialize_with = "deserialize_from_str")]
    pub(crate) distance: f64,

    #[serde(rename = "$value", deserialize_with = "wkt::deserialize_wkt")]
    pub(crate) expected: geo::Geometry,
}

#[derive(Debug, Deserialize)]
pub struct ContainsInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool,
}

#[derive(Debug, Deserialize)]
pub struct CoversInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool,
}

#[derive(Debug, Deserialize)]
pub struct WithinInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "$value", deserialize_with = "deserialize_from_str")]
    pub(crate) expected: bool,
}

#[derive(Debug, Deserialize)]
pub struct OverlayInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "$value", deserialize_with = "wkt::deserialize_wkt")]
    pub(crate) expected: geo::Geometry<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name")]
pub(crate) enum OperationInput {
    #[serde(rename = "buffer")]
    BufferInput(BufferInput),

    #[serde(rename = "contains")]
    ContainsInput(ContainsInput),

    #[serde(rename = "covers")]
    CoversInput(CoversInput),

    #[serde(rename = "getCentroid")]
    CentroidInput(CentroidInput),

    #[serde(rename = "convexhull")]
    ConvexHullInput(ConvexHullInput),

    #[serde(rename = "equalsTopo")]
    EqualsTopoInput(EqualsTopoInput),

    #[serde(rename = "intersects")]
    IntersectsInput(IntersectsInput),

    #[serde(rename = "isValid")]
    IsValidInput(IsValidInput),

    #[serde(rename = "relate")]
    RelateInput(RelateInput),

    #[serde(rename = "union")]
    UnionInput(OverlayInput),

    #[serde(rename = "intersection")]
    IntersectionInput(OverlayInput),

    #[serde(rename = "difference")]
    DifferenceInput(OverlayInput),

    #[serde(rename = "symdifference")]
    SymDifferenceInput(OverlayInput),

    #[serde(rename = "within")]
    WithinInput(WithinInput),

    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Buffer {
        subject: Geometry,
        distance: f64,
        expected: Geometry,
    },
    Centroid {
        subject: Geometry,
        expected: Option<Point>,
    },
    Contains {
        subject: Geometry,
        target: Geometry,
        expected: bool,
    },
    Covers {
        subject: Geometry,
        target: Geometry,
        expected: bool,
    },
    IsValidOp {
        subject: Geometry,
        expected: bool,
    },
    Within {
        subject: Geometry,
        target: Geometry,
        expected: bool,
    },
    ConvexHull {
        subject: Geometry,
        expected: Geometry,
    },
    EqualsTopo {
        a: Geometry,
        b: Geometry,
        expected: bool,
    },
    Intersects {
        subject: Geometry,
        clip: Geometry,
        expected: bool,
    },
    Relate {
        a: Geometry,
        b: Geometry,
        expected: IntersectionMatrix,
    },
    BooleanOp {
        a: Geometry<f64>,
        b: Geometry<f64>,
        op: BoolOp,
        expected: Geometry<f64>,
    },
    ClipOp {
        a: Geometry<f64>,
        b: Geometry<f64>,
        invert: bool,
        expected: Geometry<f64>,
    },
    Unsupported {
        #[allow(dead_code)]
        reason: String,
    },
}

/// Resolves the (arg1, arg2) references of a binary predicate op to the
/// case geometries, supporting both (A, B) and the swapped (B, A) order
/// used by some fixtures.
fn binary_args<'c>(case: &'c Case, arg1: &str, arg2: &str) -> Result<(&'c Geometry, &'c Geometry)> {
    let a = case
        .a
        .as_ref()
        .ok_or("geometry A is not representable in geo")?;
    let b = case.b.as_ref().ok_or("no geometry b in case")?;
    match (arg1.to_uppercase().as_str(), arg2.to_uppercase().as_str()) {
        ("A", "B") => Ok((a, b)),
        ("B", "A") => Ok((b, a)),
        (arg1, arg2) => Err(format!("unsupported op arguments: {arg1}, {arg2}").into()),
    }
}

impl OperationInput {
    pub(crate) fn into_operation(self, case: &Case) -> Result<Operation> {
        let geometry = case
            .a
            .as_ref()
            .ok_or("geometry A is not representable in geo")?;
        match self {
            Self::BufferInput(BufferInput {
                arg1,
                distance,
                expected,
            }) => {
                assert_eq!("A", arg1.to_uppercase());
                Ok(Operation::Buffer {
                    subject: geometry.clone(),
                    distance,
                    expected,
                })
            }
            Self::CentroidInput(centroid_input) => {
                assert_eq!("A", centroid_input.arg1);
                Ok(Operation::Centroid {
                    subject: geometry.clone(),
                    expected: centroid_input.expected,
                })
            }
            Self::ConvexHullInput(convex_hull_input) => {
                assert_eq!("A", convex_hull_input.arg1);
                Ok(Operation::ConvexHull {
                    subject: geometry.clone(),
                    expected: convex_hull_input.expected,
                })
            }
            Self::EqualsTopoInput(input) => {
                let (first, second) = binary_args(case, &input.arg1, &input.arg2)?;
                Ok(Operation::EqualsTopo {
                    a: first.clone(),
                    b: second.clone(),
                    expected: input.expected,
                })
            }
            Self::IntersectsInput(input) => {
                let (first, second) = binary_args(case, &input.arg1, &input.arg2)?;
                Ok(Operation::Intersects {
                    subject: first.clone(),
                    clip: second.clone(),
                    expected: input.expected,
                })
            }
            Self::RelateInput(input) => {
                let (first, second) = binary_args(case, &input.arg1, &input.arg2)?;
                Ok(Operation::Relate {
                    a: first.clone(),
                    b: second.clone(),
                    expected: input.expected,
                })
            }
            Self::ContainsInput(input) => {
                let (first, second) = binary_args(case, &input.arg1, &input.arg2)?;
                Ok(Operation::Contains {
                    subject: first.clone(),
                    target: second.clone(),
                    expected: input.expected,
                })
            }
            Self::CoversInput(input) => {
                let (first, second) = binary_args(case, &input.arg1, &input.arg2)?;
                Ok(Operation::Covers {
                    subject: first.clone(),
                    target: second.clone(),
                    expected: input.expected,
                })
            }
            Self::WithinInput(input) => {
                let (first, second) = binary_args(case, &input.arg1, &input.arg2)?;
                Ok(Operation::Within {
                    subject: first.clone(),
                    target: second.clone(),
                    expected: input.expected,
                })
            }
            Self::UnionInput(input) => {
                validate_boolean_op(
                    &input.arg1,
                    &input.arg2,
                    geometry,
                    case.b.as_ref().ok_or("no geometry b in case")?,
                )?;
                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().ok_or("no geometry b in case")?,
                    op: BoolOp::Union,
                    expected: input.expected,
                })
            }
            Self::IntersectionInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);

                // Clipping a line string in geo is like a Line x Poly Intersection in JTS
                match (geometry, case.b.as_ref().ok_or("no geometry b in case")?) {
                    (
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                    )
                    | (
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                    ) => {
                        return Ok(Operation::ClipOp {
                            a: geometry.clone(),
                            b: case.b.clone().ok_or("no geometry b in case")?,
                            invert: false,
                            expected: input.expected,
                        });
                    }
                    _ => {
                        validate_boolean_op(
                            &input.arg1,
                            &input.arg2,
                            geometry,
                            case.b.as_ref().ok_or("no geometry b in case")?,
                        )?;
                    }
                };

                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().ok_or("no geometry b in case")?,
                    op: BoolOp::Intersection,
                    expected: input.expected,
                })
            }
            Self::DifferenceInput(input) => {
                // Clipping a line string in geo is like a Line x Poly Intersection in JTS
                match (geometry, case.b.as_ref().ok_or("no geometry b in case")?) {
                    (
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                    )
                    | (
                        Geometry::Polygon(_) | Geometry::MultiPolygon(_),
                        Geometry::LineString(_) | Geometry::MultiLineString(_),
                    ) => {
                        return Ok(Operation::ClipOp {
                            a: geometry.clone(),
                            b: case.b.clone().ok_or("no geometry b in case")?,
                            invert: true,
                            expected: input.expected,
                        });
                    }
                    _ => {
                        validate_boolean_op(
                            &input.arg1,
                            &input.arg2,
                            geometry,
                            case.b.as_ref().ok_or("no geometry b in case")?,
                        )?;
                    }
                };
                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().ok_or("no geometry b in case")?,
                    op: BoolOp::Difference,
                    expected: input.expected,
                })
            }
            Self::SymDifferenceInput(input) => {
                validate_boolean_op(
                    &input.arg1,
                    &input.arg2,
                    geometry,
                    case.b.as_ref().ok_or("no geometry b in case")?,
                )?;
                Ok(Operation::BooleanOp {
                    a: geometry.clone(),
                    b: case.b.clone().ok_or("no geometry b in case")?,
                    op: BoolOp::Xor,
                    expected: input.expected,
                })
            }
            Self::Unsupported => Err("This OperationInput not supported".into()),
            OperationInput::IsValidInput(input) => match input.arg1.as_str() {
                "A" => Ok(Operation::IsValidOp {
                    subject: geometry.clone(),
                    expected: input.expected,
                }),
                "B" => Ok(Operation::IsValidOp {
                    subject: case.b.clone().ok_or("no geometry b in case")?,
                    expected: input.expected,
                }),
                _ => todo!("Handle {}", input.arg1),
            },
        }
    }
}

fn validate_boolean_op(arg1: &str, arg2: &str, a: &Geometry<f64>, b: &Geometry<f64>) -> Result<()> {
    assert_eq!("A", arg1);
    assert_eq!("B", arg2);
    for arg in &[a, b] {
        if matches!(arg, Geometry::LineString(_)) {
            log::warn!("skipping `line_string.union` we don't support");
            return Err("`line_string.union` is not supported".into());
        }
        if matches!(arg, Geometry::MultiPoint(_)) {
            log::warn!("skipping `line_string.union` we don't support");
            return Err("`line_string.union` is not supported".into());
        }
    }
    Ok(())
}

/// Deserialises an optional WKT geometry, treating WKT that cannot be
/// represented as a geo geometry (for example a MultiPoint with an EMPTY
/// point element) as absent instead of failing the whole file. Cases with
/// an absent geometry are skipped per test, with a log message, rather
/// than silently dropping every case in the file.
pub fn deserialize_lenient_geometry<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Geometry>, D::Error>
where
    D: Deserializer<'de>,
{
    use wkt::TryFromWkt;

    let raw = Option::<String>::deserialize(deserializer)?;
    Ok(
        raw.and_then(|wkt_str| match Geometry::try_from_wkt_str(&wkt_str) {
            Ok(geometry) => Some(geometry),
            Err(e) => {
                log::warn!("geometry not representable in geo, skipping its cases: {e}: {wkt_str}");
                None
            }
        }),
    )
}

pub fn deserialize_from_str<'de, T, D>(deserializer: D) -> std::result::Result<T, D::Error>
where
    T: std::str::FromStr,
    D: Deserializer<'de>,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    String::deserialize(deserializer)
        .and_then(|str| T::from_str(&str).map_err(serde::de::Error::custom))
}
