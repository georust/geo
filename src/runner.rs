use std::collections::BTreeSet;

use approx::relative_eq;
use include_dir::{include_dir, Dir, DirEntry};

use super::{input, Operation, Result};
use geo::{Coordinate, Geometry, LineString, Polygon};

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
                    // whereas geo *alway* returns a polygon
                    //
                    // This is currently the cause of some test failures.
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
                        },
                    };
                    if is_polygon_rotated_eq(&actual_polygon, &expected, |c1, c2| relative_eq!(c1, c2)) {
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
            format!("{}", filter)
        } else {
            format!("**/*.xml")
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
                    error!(
                        "skipping invalid test input: {:?}. error: {:?}",
                        file.path(),
                        err
                    );
                    continue;
                }
            };
            for case in run.cases {
                if let Some(desc_filter) = &self.desc_filter {
                    if case.desc.as_str() != desc_filter {
                        debug!("filter skipped case: {}", &case.desc);
                        continue;
                    } else {
                        debug!("filter matched case: {}", &case.desc);
                    }
                } else {
                    debug!("parsing case {}:", &case.desc);
                }

                let geometry = match geometry_try_from_wkt_str(&case.a) {
                    Ok(g) => g,
                    Err(e) => {
                        warn!(
                            "skipping case after failing to parse wkt into geometry: {:?}",
                            e
                        );
                        continue;
                    }
                };

                for test in case.tests {
                    let description = case.desc.clone();

                    let test_file_name = file
                        .path()
                        .file_name()
                        .expect("file from include_dir unexpectedly missing name")
                        .to_string_lossy()
                        .to_string();

                    match test.operation_input.into_operation(geometry.clone()) {
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

fn geometry_try_from_wkt_str<T>(wkt_str: &str) -> Result<Geometry<T>>
where
    T: wkt::WktFloat + std::str::FromStr + std::default::Default,
{
    use std::convert::TryInto;
    Ok(wkt::Wkt::from_str(&wkt_str)?.try_into()?)
}

/// Test if two polygons are equal upto rotation, and permutation of iteriors
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
            !matched_in_p2.contains(&j) && is_ring_rotated_eq(r1, r2, &coord_matcher)
        });
        if let Some((j, _)) = did_match {
            matched_in_p2.insert(j);
        } else {
            return false;
        }
    }
    return true;
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
