mod input;

use approx::relative_eq;
use geo::{Area, BooleanOps, Geometry, Relate};
use input::Operation;
use log::debug;

mod runner;
pub use runner::TestRunner;

type Error = Box<dyn std::error::Error>;
type Result<T> = std::result::Result<T, Error>;

/// ```
/// use jts_test_runner::assert_jts_tests_succeed;
/// assert_jts_tests_succeed("*Relate*.xml");
/// ```
pub fn assert_jts_tests_succeed(pattern: &str) {
    let mut runner = TestRunner::new().matching_filename_glob(pattern);
    runner.run().expect("test cases failed");

    // sanity check that *something* was run
    assert!(
        runner.unexpected_failures().len() + runner.successes().len() > 0,
        "No tests were run."
    );

    if !runner.unexpected_failures().is_empty() {
        let failure_text = runner
            .unexpected_failures()
            .iter()
            .map(|failure| format!("{}", failure))
            .collect::<Vec<String>>()
            .join("\n");

        panic!(
            "{} unexpected failures / {} successes in JTS test suite:\n{}",
            runner.unexpected_failures().len(),
            runner.successes().len(),
            failure_text
        );
    }

    for success in runner.successes() {
        if runner.expected_failures().contains(&success.test_id) {
            panic!("Test {:?} was expected to fail, but it passed. Did you fix something? Good job! Update `expected_failures`.", success.test_id);
        }
    }
}

pub fn check_buffer_test_case(
    actual: &Geometry,
    expected: &Geometry,
) -> std::result::Result<(), String> {
    // This error threshold is arbitrary. I ratcheted it down until tests started failing.
    // Manually inspecting the output of the borderline cases appeared subjectively "reasonable".
    //
    // In particular, we seem to diverge the most when doing large subtractive (negative buffers)
    // from complex geometries (e.g., polygons with narrow arms that get wholly erased)
    check_buffer_test_case_with_error_ratio(actual, expected, 0.0015)
}

pub fn check_buffer_test_case_with_error_ratio(
    actual: &Geometry,
    expected: &Geometry,
    max_error_ratio: f64,
) -> std::result::Result<(), String> {
    let im = actual.relate(expected);
    if im.is_equal_topo() {
        debug!("Buffer success (equal_topo)");
        Ok(())
    } else if relative_eq!(actual, expected) {
        debug!("Buffer success (relative eq)");
        Ok(())
    } else {
        let diff = match (expected, &actual) {
            (Geometry::MultiPolygon(expected), Geometry::MultiPolygon(actual)) => {
                expected.xor(actual)
            }
            (Geometry::Polygon(expected), Geometry::MultiPolygon(actual)) => expected.xor(actual),
            (Geometry::MultiPolygon(expected), Geometry::Polygon(actual)) => expected.xor(actual),
            (Geometry::Polygon(expected), Geometry::Polygon(actual)) => expected.xor(actual),
            _ => unreachable!("unexpected comparison"),
        };

        let diff_area = diff.unsigned_area();
        let area_error_ratio = diff_area / expected.unsigned_area();

        if area_error_ratio > max_error_ratio {
            let error_description = format!("diff_area: {diff_area}, area_error_ratio: {area_error_ratio}\nactual:\n  {actual:?}\nexpected:\n  {expected:?}");
            Err(error_description)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    fn init_logging() {
        use std::sync::Once;
        static LOG_SETUP: Once = Once::new();
        LOG_SETUP.call_once(|| {
            pretty_env_logger::init();
        });
    }

    #[test]
    // several of the ConvexHull tests are currently failing
    fn test_all_general() {
        init_logging();
        let mut runner = TestRunner::new();
        runner.run().expect("test cases failed");

        if !runner.unexpected_failures().is_empty() {
            let failure_text = runner
                .unexpected_failures()
                .iter()
                .map(|failure| format!("{}", failure))
                .collect::<Vec<String>>()
                .join("\n");
            panic!(
                "{} unexpected_failures / {} successes in JTS test suite:\n{}",
                runner.unexpected_failures().len(),
                runner.successes().len(),
                failure_text
            );
        }

        // sanity check that the expected number of tests were run.
        //
        // We'll need to increase this number as more tests are added, but it should never be
        // decreased.
        let expected_test_count: usize = 3911;
        let actual_test_count = runner.unexpected_failures().len() + runner.successes().len();
        match actual_test_count.cmp(&expected_test_count) {
            Ordering::Less => {
                panic!(
                    "We're running {} less test cases than before. What happened to them?",
                    expected_test_count - actual_test_count
                );
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                panic!(
                    "Great, looks like we're running new tests. Just increase `expected_test_count` to {}",
                    actual_test_count
                );
            }
        }
    }
}
