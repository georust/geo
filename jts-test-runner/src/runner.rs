use std::collections::BTreeSet;

use approx::relative_eq;
use include_dir::{include_dir, Dir, DirEntry};
use log::{debug, info};

use super::{input, Operation, Result};
use geo::{intersects::Intersects, prelude::Contains, Coordinate, Geometry, LineString, Polygon};

const GENERAL_TEST_XML: Dir = include_dir!("resources/testxml/general");

#[derive(Debug, Default)]
pub struct TestRunner {
    filename_filter: Option<String>,
    desc_filter: Option<String>,
    failures: Vec<TestFailure>,
    unsupported: Vec<TestCase>,
    successes: Vec<TestCase>,
}

#[derive(Debug)]
pub struct TestCase {
    test_file_name: String,
    description: String,
    operation: Operation,
}

#[derive(Debug)]
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

    pub fn run(&mut self) -> Result<()> {
        let cases = self.parse_cases()?;
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
                                format!("expected {:?}, actual: {:?}", expected, actual);
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

                    // TODO: impl `Contains` for `Geometry` in geo and check that result here too
                    // let direct_actual = subject.contains(target);
                    let verify_contains_trait = match (subject, target) {
                        (Geometry::Point(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::Point(subject), Geometry::Line(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::LineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Point(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::Line(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::Line(subject), Geometry::Line(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::Line(subject), Geometry::LineString(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::Line(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Line(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Line(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Line(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Line(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Line(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Line(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::LineString(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::LineString(subject), Geometry::Line(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::LineString(subject), Geometry::LineString(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::LineString(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::LineString(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::LineString(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::LineString(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::LineString(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::LineString(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::LineString(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::Polygon(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::Polygon(subject), Geometry::Line(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::Polygon(subject), Geometry::LineString(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::Polygon(subject), Geometry::Polygon(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::Polygon(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Polygon(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Polygon(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Polygon(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Polygon(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Polygon(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::MultiPoint(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::MultiPoint(subject), Geometry::Line(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::LineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiPoint(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::MultiLineString(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiLineString(subject), Geometry::Line(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiLineString(subject), Geometry::LineString(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::MultiLineString(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiLineString(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiLineString(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiLineString(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiLineString(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiLineString(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::MultiLineString(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::MultiPolygon(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::Line(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::LineString(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::Polygon(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::MultiPoint(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::MultiLineString(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::MultiPolygon(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::GeometryCollection(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::Rect(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::MultiPolygon(subject), Geometry::Triangle(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        (Geometry::GeometryCollection(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::GeometryCollection(subject), Geometry::Line(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::LineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::GeometryCollection(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::Rect(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::Rect(subject), Geometry::Line(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Rect(subject), Geometry::LineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Rect(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Rect(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Rect(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Rect(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Rect(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::Rect(subject), Geometry::Rect(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::Rect(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        (Geometry::Triangle(subject), Geometry::Point(target)) => {
                            subject.contains(target) == relate_actual
                        }
                        // (Geometry::Triangle(subject), Geometry::Line(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::LineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::Polygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::MultiPoint(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::MultiLineString(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::MultiPolygon(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::GeometryCollection(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::Rect(target)) => { subject.contains(target) == relate_actual }
                        // (Geometry::Triangle(subject), Geometry::Triangle(target)) => { subject.contains(target) == relate_actual }
                        _ => true,
                    };

                    if relate_actual != *expected {
                        debug!("Contains failure: relate_actual != expected");
                        let error_description = format!(
                            "expected {:?}, relate_actual: {:?}",
                            expected, relate_actual
                        );
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else if !verify_contains_trait {
                        debug!("Contains failure: relate_actual != contains_trait_impl");
                        let error_description = format!(
                            "expected {:?}, contains_trait_impl: {:?}",
                            expected, !relate_actual
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
                            let error_description = format!("expected result for convex hull is not a polygon or a linestring: {:?}", expected);
                            self.failures.push(TestFailure {
                                test_case,
                                error_description,
                            });
                            continue;
                        }
                    };
                    if is_polygon_rotated_eq(&actual_polygon, &expected, |c1, c2| {
                        relative_eq!(c1, c2)
                    }) {
                        debug!("ConvexHull success: actual == expected");
                        self.successes.push(test_case);
                    } else {
                        debug!("ConvexHull failure: actual != expected");
                        let error_description =
                            format!("expected {:?}, actual: {:?}", expected, actual_polygon);
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
                        let error_description = format!(
                            "expected {:?}, direct_actual: {:?}",
                            expected, direct_actual
                        );
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    } else if relate_actual != *expected {
                        debug!("Intersects failure: relate_actual != expected");
                        let error_description = format!(
                            "expected {:?}, relate_actual: {:?}",
                            expected, relate_actual
                        );
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
                            format!("expected {:?}, actual: {:?}", expected, actual);
                        self.failures.push(TestFailure {
                            test_case,
                            error_description,
                        });
                    }
                }
            }
        }
        info!(
            "successes: {}, failures: {}",
            self.successes.len(),
            self.failures.len()
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

        for entry in GENERAL_TEST_XML.find(&filename_filter)? {
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
                            cases.push(TestCase {
                                description,
                                test_file_name,
                                operation,
                            });
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

/// Test if two polygons are equal upto rotation, and
/// permutation of interiors.
pub fn is_polygon_rotated_eq<T, F>(p1: &Polygon<T>, p2: &Polygon<T>, coord_matcher: F) -> bool
where
    T: geo::GeoNum,
    F: Fn(&Coordinate<T>, &Coordinate<T>) -> bool,
{
    if p1.interiors().len() != p2.interiors().len() {
        return false;
    }
    if !is_ring_rotated_eq(p1.exterior(), p2.exterior(), &coord_matcher) {
        return false;
    }

    let mut matched_in_p2: BTreeSet<usize> = BTreeSet::new();
    for r1 in p1.interiors().iter() {
        let did_match = p2.interiors().iter().enumerate().find(|(j, r2)| {
            !matched_in_p2.contains(j) && is_ring_rotated_eq(r1, r2, &coord_matcher)
        });
        if let Some((j, _)) = did_match {
            matched_in_p2.insert(j);
        } else {
            return false;
        }
    }
    true
}

/// Test if two rings are equal upto rotation / reversal
pub fn is_ring_rotated_eq<T, F>(r1: &LineString<T>, r2: &LineString<T>, coord_matcher: F) -> bool
where
    T: geo::GeoNum,
    F: Fn(&Coordinate<T>, &Coordinate<T>) -> bool,
{
    assert!(r1.is_closed(), "r1 is not closed");
    assert!(r2.is_closed(), "r2 is not closed");
    if r1.0.len() != r2.0.len() {
        return false;
    }
    let len = r1.0.len() - 1;
    (0..len).any(|shift| {
        (0..len).all(|i| coord_matcher(&r1.0[i], &r2.0[(i + shift) % len]))
            || (0..len).all(|i| coord_matcher(&r1.0[len - i], &r2.0[(i + shift) % len]))
    })
}
