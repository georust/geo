use std::collections::BTreeSet;

use approx::relative_eq;
use include_dir::{include_dir, Dir, DirEntry};
use log::{debug, info};
use wkt::ToWkt;

use super::{check_buffer_test_case, input, Operation, Result};
use geo::algorithm::{BooleanOps, Contains, HasDimensions, Intersects, Relate, Within};
use geo::geometry::*;
use geo::GeoNum;

const GENERAL_TEST_XML: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/testxml/general");
const VALIDATE_TEST_XML: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/testxml/validate");
const MISC_TEST_XML: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/testxml/misc");

#[derive(Debug, Default, Clone)]
pub struct TestRunner {
    filename_filter: Option<String>,
    desc_filter: Option<String>,
    cases: Option<Vec<TestCase>>,
    unexpected_failures: Vec<TestFailure>,
    expected_failures: Vec<TestId>,
    unsupported: Vec<TestCase>,
    successes: Vec<TestCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestId {
    file_name: String,
    case_idx: usize,
    test_idx: usize,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub(crate) test_id: TestId,
    description: String,
    operation: Operation,
}

#[derive(Debug, Clone)]
pub struct TestFailure {
    error_description: String,
    test_case: TestCase,
}

impl std::fmt::Display for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "failed {:?}: \"{}\" with error: {}",
            &self.test_case.test_id, &self.test_case.description, &self.error_description
        )
    }
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            // Please document any expected failures as you add them.
            expected_failures: vec![
                // Degenerate case we don't yet handle: buffering a collapsed (flat) polygon
                TestId {
                    file_name: "TestBuffer.xml".to_string(),
                    case_idx: 6,
                    test_idx: 1,
                },
            ],
            ..Self::default()
        }
    }

    pub fn successes(&self) -> &[TestCase] {
        &self.successes
    }

    pub fn add_failure(&mut self, failure: TestFailure) {
        if self.expected_failures.contains(&failure.test_case.test_id) {
            return;
        }
        self.unexpected_failures.push(failure);
    }

    pub fn unexpected_failures(&self) -> &[TestFailure] {
        &self.unexpected_failures
    }

    pub fn expected_failures(&self) -> &[TestId] {
        &self.expected_failures
    }

    /// `desc`: when specified runs just the test described by `desc`, otherwise all tests are run
    pub fn matching_desc(mut self, desc: &str) -> Self {
        self.desc_filter = Some(desc.to_string());
        self
    }

    pub fn matching_filename_glob(mut self, filename: &str) -> Self {
        self.filename_filter = Some(filename.to_string());
        self
    }

    pub fn prepare_cases(&mut self) -> Result<()> {
        self.cases = Some(self.parse_cases()?);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        let cases = if let Some(cases) = self.cases.take() {
            cases
        } else {
            self.parse_cases()?
        };

        debug!("cases.len(): {}", cases.len());

        for test_case in cases {
            match &test_case.operation {
                Operation::Buffer {
                    subject,
                    distance,
                    expected,
                } => {
                    use geo::algorithm::Buffer;
                    let actual = Geometry::from(subject.buffer(*distance));
                    if let Err(error_description) = check_buffer_test_case(&actual, expected) {
                        debug!(
                            "Buffer failure {:?}. {error_description}",
                            test_case.test_id
                        );
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else {
                        debug!("Buffer success (xor area close enough)");
                        self.successes.push(test_case);
                    }
                }
                Operation::Centroid { subject, expected } => {
                    use geo::prelude::Centroid;
                    match (subject.centroid(), expected) {
                        (None, None) => {
                            debug!("Centroid success: None == None");
                            self.successes.push(test_case);
                        }
                        (Some(actual), Some(expected)) if relative_eq!(actual, expected) => {
                            debug!("Centroid success: actual == expected");
                            self.successes.push(test_case);
                        }
                        (actual, expected) => {
                            debug!("Centroid failure: actual != expected");
                            let error_description =
                                format!("expected {expected:?}, actual: {actual:?}");
                            self.add_failure(TestFailure {
                                test_case,
                                error_description,
                            });
                        }
                    }
                }
                Operation::Contains {
                    subject,
                    target,
                    expected,
                } => {
                    let relate_actual = subject.relate(target).is_contains();
                    let direct_actual = subject.contains(target);

                    if relate_actual != *expected {
                        debug!("Contains failure: Relate doesn't match expected");
                        let error_description = format!(
                            "Contains failure: expected {expected:?}, relate: {relate_actual:?}"
                        );
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else if relate_actual != direct_actual {
                        debug!(
                            "Contains failure: Relate doesn't match Contains trait implementation"
                        );
                        let error_description = format!(
                            "Contains failure - Relate.is_contains: {expected:?} doesn't match Contains trait: {direct_actual:?}"
                        );
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else {
                        debug!("Contains success: actual == expected");
                        self.successes.push(test_case);
                    }
                }
                Operation::EqualsTopo { a, b, expected } => {
                    let im = a.relate(b);
                    let actual = im.is_equal_topo();
                    if actual == *expected {
                        debug!("Passed: EqualsTopo was {actual}");
                        self.successes.push(test_case);
                    } else {
                        debug!("is_equal_topo was {actual}, but expected {expected}");
                        let error_description =
                            format!("is_equal_topo was {actual}, but expected {expected}");
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
                Operation::IsValidOp { subject, expected } => {
                    use geo::algorithm::Validation;
                    let actual = subject.is_valid();
                    if actual == *expected {
                        debug!("IsValidOp success: actual == expected");
                        self.successes.push(test_case);
                    } else {
                        debug!("IsValidOp failure: actual != expected");
                        let error_description =
                            format!("expected {expected:?}, actual: {actual:?}",);
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
                Operation::Within {
                    subject,
                    target,
                    expected,
                } => {
                    let relate_within_result = subject.relate(target).is_within();
                    let within_trait_result = subject.is_within(target);

                    if relate_within_result != *expected {
                        debug!("Within failure: Relate doesn't match expected");
                        let error_description = format!(
                            "Within failure: expected {expected:?}, relate: {relate_within_result:?}"
                        );
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else if relate_within_result != within_trait_result {
                        debug!("Within failure: Relate doesn't match Within trait implementation");
                        let error_description = format!(
                            "Within failure: Relate: {expected:?}, Within trait: {within_trait_result:?}"
                        );
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else {
                        debug!("Within success: actual == expected");
                        self.successes.push(test_case);
                    }
                }
                Operation::ConvexHull { subject, expected } => {
                    use geo::prelude::ConvexHull;

                    let actual_polygon = match subject {
                        Geometry::MultiPoint(g) => g.convex_hull(),
                        Geometry::Point(_g) => {
                            debug!("ConvexHull not implemented for this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                        Geometry::Line(_g) => {
                            debug!("ConvexHull not implemented for this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                        Geometry::LineString(_g) => {
                            debug!("ConvexHull not implemented for this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                        Geometry::Polygon(g) => g.convex_hull(),
                        Geometry::MultiLineString(_g) => {
                            debug!("ConvexHull not implemented for this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                        Geometry::MultiPolygon(_g) => {
                            debug!("ConvexHull not implemented for this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                        Geometry::GeometryCollection(_g) => {
                            debug!("ConvexHull not implemented for this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                        Geometry::Rect(_g) => {
                            debug!("ConvexHull not implemented for this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                        Geometry::Triangle(_g) => {
                            debug!("ConvexHull doesn't support this geometry (yet?)");
                            self.unsupported.push(test_case);
                            continue;
                        }
                    };

                    // JTS returns a variety of Geometry types depending on the convex hull
                    // whereas geo *always* returns a polygon.
                    let expected = match expected {
                        Geometry::LineString(ext) => Polygon::new(ext.clone(), vec![]),
                        Geometry::Polygon(p) => p.clone(),
                        _ => {
                            let error_description = format!("expected result for convex hull is not a polygon or a linestring: {expected:?}" );
                            self.add_failure(TestFailure {
                                test_case,
                                error_description,
                            });
                            continue;
                        }
                    };
                    if actual_polygon.is_rotated_eq(&expected, |c1, c2| relative_eq!(c1, c2)) {
                        debug!("ConvexHull success: actual == expected");
                        self.successes.push(test_case);
                    } else {
                        debug!("ConvexHull failure: actual != expected");
                        let error_description =
                            format!("expected {expected:?}, actual: {actual_polygon:?}");
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
                Operation::Intersects {
                    subject,
                    clip,
                    expected,
                } => {
                    let direct_actual = subject.intersects(clip);
                    let relate_actual = subject.relate(clip).is_intersects();

                    if direct_actual != *expected {
                        debug!("Intersects failure: direct_actual != expected");
                        let error_description =
                            format!("expected {expected:?}, direct_actual: {direct_actual:?}",);
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else if relate_actual != *expected {
                        debug!("Intersects failure: relate_actual != expected");
                        let error_description =
                            format!("expected {expected:?}, relate_actual: {relate_actual:?}",);
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else {
                        debug!("Intersects success: actual == expected");
                        self.successes.push(test_case);
                    }
                }
                Operation::Relate { a, b, expected } => {
                    let actual = a.relate(b);
                    if actual == *expected {
                        debug!("Relate success: actual == expected");
                        self.successes.push(test_case);
                    } else {
                        debug!("Relate failure: actual != expected");
                        let error_description =
                            format!("expected {expected:?}, actual: {actual:?}");
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
                Operation::BooleanOp { a, b, op, expected } => {
                    match expected {
                        Geometry::MultiPolygon(_) | Geometry::Polygon(_) => {}
                        _ => {
                            info!("skipping unsupported Union expectation: {expected:?}");
                            self.unsupported.push(test_case);
                            continue;
                        }
                    };

                    let actual = match (a, b) {
                        (Geometry::Polygon(a), Geometry::Polygon(b)) => a.boolean_op(b, *op),
                        (Geometry::Polygon(a), Geometry::MultiPolygon(b)) => a.boolean_op(b, *op),
                        (Geometry::MultiPolygon(a), Geometry::MultiPolygon(b)) => {
                            a.boolean_op(b, *op)
                        }
                        (Geometry::MultiPolygon(a), Geometry::Polygon(b)) => a.boolean_op(b, *op),
                        _ => {
                            info!("skipping unsupported Union combination: {a:?}, {b:?}");
                            self.unsupported.push(test_case);
                            continue;
                        }
                    };

                    if actual.relate(expected).is_equal_topo() {
                        debug!(
                            "BooleanOp success (topo eq) - expected: {:?}",
                            expected.wkt_string()
                        );
                        self.successes.push(test_case);
                    } else {
                        let error_description = format!(
                            "op: {:?}, expected {:?}, actual: {:?}",
                            op,
                            expected.wkt_string(),
                            actual.wkt_string()
                        );
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
                Operation::ClipOp {
                    a,
                    b,
                    invert,
                    expected,
                } => {
                    match expected {
                        Geometry::MultiLineString(_) | Geometry::LineString(_) => {}
                        other => {
                            info!("skipping unsupported ClipOp output: {other:?}");
                            self.unsupported.push(test_case);
                            continue;
                        }
                    };

                    let actual = match (a, b) {
                        (Geometry::Polygon(polygon), Geometry::LineString(line_string))
                        | (Geometry::LineString(line_string), Geometry::Polygon(polygon)) => {
                            // REVIEW: add a line_string flavor
                            polygon.clip(&MultiLineString(vec![line_string.clone()]), *invert)
                        }
                        (
                            Geometry::Polygon(polygon),
                            Geometry::MultiLineString(multi_line_string),
                        )
                        | (
                            Geometry::MultiLineString(multi_line_string),
                            Geometry::Polygon(polygon),
                        ) => polygon.clip(multi_line_string, *invert),
                        (
                            Geometry::LineString(line_string),
                            Geometry::MultiPolygon(multi_polygon),
                        )
                        | (
                            Geometry::MultiPolygon(multi_polygon),
                            Geometry::LineString(line_string),
                        ) => {
                            multi_polygon.clip(&MultiLineString(vec![line_string.clone()]), *invert)
                        }
                        (
                            Geometry::MultiLineString(multi_line_string),
                            Geometry::MultiPolygon(multi_polygon),
                        )
                        | (
                            Geometry::MultiPolygon(multi_polygon),
                            Geometry::MultiLineString(multi_line_string),
                        ) => multi_polygon.clip(multi_line_string, *invert),

                        // We should be filtering the input test cases in such a way that we don't get here.
                        _ => todo!("Handle {:?} and {:?}", a, b),
                    };

                    if actual.relate(expected).is_equal_topo() {
                        debug!(
                            "ClipOp success (topo eq) - expected: {:?}",
                            expected.wkt_string()
                        );
                        self.successes.push(test_case);
                    } else {
                        let error_description = format!(
                            "expected {:?}, actual: {:?}",
                            expected.wkt_string(),
                            actual.wkt_string()
                        );
                        self.add_failure(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
                Operation::Unsupported { reason: _ } => self.unsupported.push(test_case),
            }
        }
        debug!("unsupported: {:?}", self.unsupported);
        info!(
            "run summary: successes: {}, unexpected failures: {}, expected_failures: {}, unsupported: {}",
            self.successes.len(),
            self.unexpected_failures.len(),
            self.expected_failures.len(),
            self.unsupported.len(),
        );

        Ok(())
    }

    fn parse_cases(&self) -> Result<Vec<TestCase>> {
        let mut cases = vec![];

        let filename_filter = if let Some(filter) = &self.filename_filter {
            filter.to_string()
        } else {
            "**/*.xml".to_string()
        };

        for entry in GENERAL_TEST_XML
            .find(&filename_filter)?
            .chain(VALIDATE_TEST_XML.find(&filename_filter)?)
            .chain(MISC_TEST_XML.find(&filename_filter)?)
        {
            let file = match entry {
                DirEntry::Dir(_) => {
                    debug_assert!(false, "unexpectedly found dir.xml");
                    continue;
                }
                DirEntry::File(file) => file,
            };
            debug!("deserializing from {:?}", file.path());
            let file_reader = std::io::BufReader::new(file.contents());
            let run: input::Run = match serde_xml_rs::from_reader(file_reader) {
                Ok(r) => r,
                Err(err) => {
                    debug!(
                        "skipping invalid test input: {:?}. error: {:?}",
                        file.path(),
                        err
                    );
                    continue;
                }
            };

            for (case_idx, mut case) in run.cases.into_iter().enumerate() {
                if let Some(desc_filter) = &self.desc_filter {
                    if case.desc.as_str().contains(desc_filter) {
                        debug!("filter matched case: {}", &case.desc);
                    } else {
                        debug!("filter skipped case: {}", &case.desc);
                        continue;
                    }
                } else {
                    debug!("parsing case {}:", &case.desc);
                }
                let tests = std::mem::take(&mut case.tests);
                for (test_idx, test) in tests.into_iter().enumerate() {
                    let description = case.desc.clone();

                    let test_file_name = file
                        .path()
                        .file_name()
                        .expect("file from include_dir unexpectedly missing name")
                        .to_string_lossy()
                        .to_string();

                    let test_id = TestId {
                        file_name: test_file_name.clone(),
                        case_idx,
                        test_idx,
                    };
                    match test.operation_input.into_operation(&case) {
                        Ok(operation) => {
                            if matches!(
                                operation,
                                Operation::BooleanOp { .. } | Operation::ClipOp { .. }
                            ) && run.precision_model.is_some()
                                && &run.precision_model.as_ref().unwrap().ty != "FLOATING"
                            {
                                cases.push(TestCase {
                                    description,
                                    test_id,
                                    operation: Operation::Unsupported {
                                        reason: "unsupported BooleanOp precision model".to_string(),
                                    },
                                });
                            } else {
                                cases.push(TestCase {
                                    description,
                                    test_id,
                                    operation,
                                });
                            }
                        }
                        Err(e) => {
                            debug!("skipping unsupported operation: {e}");
                            continue;
                        }
                    }
                }
            }
        }
        Ok(cases)
    }
}

trait RotatedEq<T: GeoNum> {
    fn is_rotated_eq<F>(&self, other: &Self, coord_matcher: F) -> bool
    where
        F: Fn(&Coord<T>, &Coord<T>) -> bool;
}

impl<T: GeoNum> RotatedEq<T> for MultiPolygon<T> {
    fn is_rotated_eq<F>(&self, other: &Self, coord_matcher: F) -> bool
    where
        F: Fn(&Coord<T>, &Coord<T>) -> bool,
    {
        if self.0.len() != other.0.len() {
            // We have some discrepancies about having a multipolygon with nothing in it vs a multipolygon with an empty polygon.
            return self.is_empty() && other.is_empty();
        }
        let mut matched_in_other: BTreeSet<usize> = BTreeSet::new();

        for self_poly in self {
            let did_match = other.iter().enumerate().find(|(j, other_poly)| {
                !matched_in_other.contains(j) && self_poly.is_rotated_eq(other_poly, &coord_matcher)
            });
            if let Some((j, _)) = did_match {
                matched_in_other.insert(j);
            } else {
                return false;
            }
        }
        true
    }
}

/// Test if two polygons are equal upto rotation, and
/// permutation of interiors.
impl<T: GeoNum> RotatedEq<T> for Polygon<T> {
    fn is_rotated_eq<F>(&self, other: &Self, coord_matcher: F) -> bool
    where
        F: Fn(&Coord<T>, &Coord<T>) -> bool,
    {
        if self.interiors().len() != other.interiors().len() {
            return false;
        }
        if !self
            .exterior()
            .is_rotated_eq(other.exterior(), &coord_matcher)
        {
            return false;
        }

        let mut matched_in_other: BTreeSet<usize> = BTreeSet::new();
        for r1 in self.interiors().iter() {
            let did_match = other.interiors().iter().enumerate().find(|(j, other)| {
                !matched_in_other.contains(j) && r1.is_rotated_eq(other, &coord_matcher)
            });
            if let Some((j, _)) = did_match {
                matched_in_other.insert(j);
            } else {
                return false;
            }
        }
        true
    }
}

/// Test if two rings are equal upto rotation / reversal
impl<T: GeoNum> RotatedEq<T> for LineString<T> {
    fn is_rotated_eq<F>(&self, other: &Self, coord_matcher: F) -> bool
    where
        F: Fn(&Coord<T>, &Coord<T>) -> bool,
    {
        assert!(self.is_closed(), "self is not closed");
        assert!(other.is_closed(), "other is not closed");
        if self.0.len() != other.0.len() {
            return false;
        }
        let len = self.0.len() - 1;
        (0..len).any(|shift| {
            (0..len).all(|i| coord_matcher(&self.0[i], &other.0[(i + shift) % len]))
                || (0..len).all(|i| coord_matcher(&self.0[len - i], &other.0[(i + shift) % len]))
        })
    }
}
