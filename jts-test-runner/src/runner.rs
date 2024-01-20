use std::collections::BTreeSet;

use approx::relative_eq;
use include_dir::{include_dir, Dir, DirEntry};
use log::{debug, info};
use wkt::ToWkt;

use super::{input, Operation, Result};
use geo::algorithm::{BooleanOps, Contains, HasDimensions, Intersects, Within};
use geo::geometry::*;
use geo::GeoNum;

const GENERAL_TEST_XML: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/testxml/general");
const VALIDATE_TEST_XML: Dir = include_dir!("$CARGO_MANIFEST_DIR/resources/testxml/validate");

#[derive(Debug, Default, Clone)]
pub struct TestRunner {
    filename_filter: Option<String>,
    desc_filter: Option<String>,
    cases: Option<Vec<TestCase>>,
    failures: Vec<TestFailure>,
    unsupported: Vec<TestCase>,
    successes: Vec<TestCase>,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    test_file_name: String,
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
            "failed {} case \"{}\" with error: {}",
            &self.test_case.test_file_name, &self.test_case.description, &self.error_description
        )
    }
}

impl TestRunner {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn successes(&self) -> &Vec<TestCase> {
        &self.successes
    }

    pub fn failures(&self) -> &Vec<TestFailure> {
        &self.failures
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
                            self.failures.push(TestFailure {
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
                    use geo::Relate;
                    let relate_actual = subject.relate(target).is_contains();
                    let direct_actual = subject.contains(target);

                    if relate_actual != *expected {
                        debug!("Contains failure: Relate doesn't match expected");
                        let error_description = format!(
                            "Contains failure: expected {expected:?}, relate: {relate_actual:?}"
                        );
                        self.failures.push(TestFailure {
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
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else {
                        debug!("Contains success: actual == expected");
                        self.successes.push(test_case);
                    }
                }
                Operation::Within {
                    subject,
                    target,
                    expected,
                } => {
                    use geo::Relate;
                    let relate_within_result = subject.relate(target).is_within();
                    let within_trait_result = subject.is_within(target);

                    if relate_within_result != *expected {
                        debug!("Within failure: Relate doesn't match expected");
                        let error_description = format!(
                            "Within failure: expected {expected:?}, relate: {relate_within_result:?}"
                        );
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else if relate_within_result != within_trait_result {
                        debug!("Within failure: Relate doesn't match Within trait implementation");
                        let error_description = format!(
                            "Within failure: Relate: {expected:?}, Within trait: {within_trait_result:?}"
                        );
                        self.failures.push(TestFailure {
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
                            self.failures.push(TestFailure {
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
                        self.failures.push(TestFailure {
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
                    use geo::Relate;
                    let direct_actual = subject.intersects(clip);
                    let relate_actual = subject.relate(clip).is_intersects();

                    if direct_actual != *expected {
                        debug!("Intersects failure: direct_actual != expected");
                        let error_description =
                            format!("expected {expected:?}, direct_actual: {direct_actual:?}",);
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else if relate_actual != *expected {
                        debug!("Intersects failure: relate_actual != expected");
                        let error_description =
                            format!("expected {expected:?}, relate_actual: {relate_actual:?}",);
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else {
                        debug!("Intersects success: actual == expected");
                        self.successes.push(test_case);
                    }
                }
                Operation::Relate { a, b, expected } => {
                    use geo::Relate;
                    let actual = a.relate(b);
                    if actual == *expected {
                        debug!("Relate success: actual == expected");
                        self.successes.push(test_case);
                    } else {
                        debug!("Relate failure: actual != expected");
                        let error_description =
                            format!("expected {expected:?}, actual: {actual:?}");
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
                Operation::BooleanOp { a, b, op, expected } => {
                    let expected = match expected {
                        Geometry::MultiPolygon(multi) => multi.clone(),
                        Geometry::Polygon(poly) => MultiPolygon(vec![poly.clone()]),
                        _ => {
                            info!("skipping unsupported Union expectation: {:?}", expected);
                            self.unsupported.push(test_case);
                            continue;
                        }
                    };

                    let actual = match (a, b) {
                        (Geometry::Polygon(a), Geometry::Polygon(b)) => a.boolean_op(b, *op),
                        (Geometry::MultiPolygon(a), Geometry::MultiPolygon(b)) => {
                            a.boolean_op(b, *op)
                        }
                        _ => {
                            info!("skipping unsupported Union combination: {:?}, {:?}", a, b);
                            self.unsupported.push(test_case);
                            continue;
                        }
                    };

                    if actual.is_rotated_eq(&expected, |c1, c2| relative_eq!(c1, c2)) {
                        debug!("Union success - expected: {:?}", expected.wkt_string());
                        self.successes.push(test_case);
                    } else {
                        let error_description = format!(
                            "op: {:?}, expected {:?}, actual: {:?}",
                            op,
                            expected.wkt_string(),
                            actual.wkt_string()
                        );
                        self.failures.push(TestFailure {
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
            "run summary: successes: {}, failures: {}, unsupported: {}",
            self.successes.len(),
            self.failures.len(),
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

            for mut case in run.cases {
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
                for test in tests {
                    let description = case.desc.clone();

                    let test_file_name = file
                        .path()
                        .file_name()
                        .expect("file from include_dir unexpectedly missing name")
                        .to_string_lossy()
                        .to_string();

                    match test.operation_input.into_operation(&case) {
                        Ok(operation) => {
                            if matches!(operation, Operation::BooleanOp { .. })
                                && run.precision_model.is_some()
                                && &run.precision_model.as_ref().unwrap().ty != "FLOATING"
                            {
                                cases.push(TestCase {
                                    description,
                                    test_file_name,
                                    operation: Operation::Unsupported {
                                        reason: "unsupported BooleanOp precision model".to_string(),
                                    },
                                });
                            } else {
                                cases.push(TestCase {
                                    description,
                                    test_file_name,
                                    operation,
                                });
                            }
                        }
                        Err(e) => {
                            debug!("skipping unsupported operation: {}", e);
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
