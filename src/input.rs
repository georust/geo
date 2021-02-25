use geo::{Point, Geometry};
use serde::Deserialize;

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
    #[serde(rename="case")]
    pub cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Case {
    pub(crate) desc: String,
    // `a` seems to always be a WKT geometry, but until we can handle `POINT EMPTY` this
    // will error.
    // see https://github.com/georust/wkt/issues/61
    //
    // I also spent some time trying to have serde "try" to deserialize, skipping any
    // cases that were unparseable, without throwing away the whole thing but eventually ran out of time.
    // See https://github.com/serde-rs/serde/issues/1583 for a related approach
    //#[serde(deserialize_with = "wkt::deserialize_geometry")]
    pub(crate) a: String,
    #[serde(rename = "test")]
    pub(crate) tests: Vec<Test>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Test {
    #[serde(rename="op")]
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
#[serde(tag = "name")]
pub(crate) enum OperationInput {
    #[serde(rename = "getCentroid")]
    CentroidInput(CentroidInput),

    #[serde(rename = "convexhull")]
    ConvexHullInput(ConvexHullInput),

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
    }
}

impl OperationInput {
    pub(crate) fn into_operation(self, geometry: Geometry<f64>) -> Result<Operation>
    {
        match self {
            Self::CentroidInput(centroid_input) => {
                assert_eq!("A", centroid_input.arg1);
                Ok(Operation::Centroid { subject: geometry.clone(), expected: centroid_input.expected })
            }
            Self::ConvexHullInput(convex_hull_input) => {
                assert_eq!("A", convex_hull_input.arg1);
                Ok(Operation::ConvexHull { subject: geometry.clone(), expected: convex_hull_input.expected })
            }
            Self::Unsupported => Err("This OperationInput not supported".into())
        }
    }
}
