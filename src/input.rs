use geo::algorithm::relate::IntersectionMatrix;
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
    #[serde(rename = "case")]
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Case {
    #[serde(default)]
    pub(crate) desc: String,

    #[serde(deserialize_with = "wkt::deserialize_geometry")]
    pub(crate) a: Geometry<f64>,

    #[serde(deserialize_with = "deserialize_opt_geometry", default)]
    pub(crate) b: Option<Geometry<f64>>,

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

    #[serde(rename = "$value")]
    #[serde(deserialize_with = "wkt::deserialize_point")]
    pub(crate) expected: Option<geo::Point<f64>>,
}

#[derive(Debug, Deserialize)]
pub struct ConvexHullInput {
    pub(crate) arg1: String,

    #[serde(rename = "$value")]
    #[serde(deserialize_with = "wkt::deserialize_geometry")]
    pub(crate) expected: geo::Geometry<f64>,
}

#[derive(Debug, Deserialize)]
pub struct IntersectsInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "$value")]
    pub(crate) expected: String,
}

#[derive(Debug, Deserialize)]
pub struct RelateInput {
    pub(crate) arg1: String,
    pub(crate) arg2: String,

    #[serde(rename = "arg3")]
    #[serde(deserialize_with = "deserialize_intersection_matrix")]
    pub(crate) expected: IntersectionMatrix,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name")]
pub(crate) enum OperationInput {
    #[serde(rename = "getCentroid")]
    CentroidInput(CentroidInput),

    #[serde(rename = "convexhull")]
    ConvexHullInput(ConvexHullInput),

    #[serde(rename = "intersects")]
    IntersectsInput(IntersectsInput),

    #[serde(rename = "relate")]
    RelateInput(RelateInput),

    #[serde(other)]
    Unsupported,
}

#[derive(Debug)]
pub(crate) enum Operation {
    Centroid {
        subject: Geometry<f64>,
        expected: Option<Point<f64>>,
    },
    ConvexHull {
        subject: Geometry<f64>,
        expected: Geometry<f64>,
    },
    Intersects {
        subject: Geometry<f64>,
        clip: Geometry<f64>,
        expected: bool,
    },
    Relate {
        a: Geometry<f64>,
        b: Geometry<f64>,
        expected: IntersectionMatrix,
    },
}

impl OperationInput {
    pub(crate) fn into_operation(self, case: &Case) -> Result<Operation> {
        let geometry = &case.a;
        match self {
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
            Self::IntersectsInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);
                assert!(
                    case.b.is_some(),
                    "intersects test case must contain geometry b"
                );
                let expected = if input.expected.eq_ignore_ascii_case("true") {
                    true
                } else if input.expected.eq_ignore_ascii_case("false") {
                    false
                } else {
                    panic!("expected value was neither \"true\" nor \"false\"");
                };
                Ok(Operation::Intersects {
                    subject: geometry.clone(),
                    clip: case.b.clone().expect("no geometry b in case"),
                    expected,
                })
            }
            Self::RelateInput(input) => {
                assert_eq!("A", input.arg1);
                assert_eq!("B", input.arg2);
                assert!(
                    case.b.is_some(),
                    "intersects test case must contain geometry b"
                );
                Ok(Operation::Relate {
                    a: geometry.clone(),
                    b: case.b.clone().expect("no geometry b in case"),
                    expected: input.expected,
                })
            }
            Self::Unsupported => Err("This OperationInput not supported".into()),
        }
    }
}

pub fn deserialize_opt_geometry<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Geometry<f64>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "wkt::deserialize_geometry")] Geometry<f64>);

    Option::<Wrapper>::deserialize(deserializer).map(|opt_wrapped| opt_wrapped.map(|w| w.0))
}

pub fn deserialize_intersection_matrix<'de, D>(
    deserializer: D,
) -> std::result::Result<IntersectionMatrix, D::Error>
where
    D: Deserializer<'de>,
{
    use std::str::FromStr;
    String::deserialize(deserializer)
        .and_then(|str| IntersectionMatrix::from_str(&str).map_err(serde::de::Error::custom))
}
