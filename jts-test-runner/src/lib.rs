mod input;
use input::Operation;

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
        runner.failures().len() + runner.successes().len() > 0,
        "No tests were run."
    );

    if !runner.failures().is_empty() {
        let failure_text = runner
            .failures()
            .iter()
            .map(|failure| format!("{}", failure))
            .collect::<Vec<String>>()
            .join("\n");
        panic!(
            "{} failures / {} successes in JTS test suite:\n{}",
            runner.failures().len(),
            runner.successes().len(),
            failure_text
        );
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

        if !runner.failures().is_empty() {
            let failure_text = runner
                .failures()
                .iter()
                .map(|failure| format!("{}", failure))
                .collect::<Vec<String>>()
                .join("\n");
            panic!(
                "{} failures / {} successes in JTS test suite:\n{}",
                runner.failures().len(),
                runner.successes().len(),
                failure_text
            );
        }

        // sanity check that the expected number of tests were run.
        //
        // We'll need to increase this number as more tests are added, but it should never be
        // decreased.
        let expected_test_count: usize = 3775;
        let actual_test_count = runner.failures().len() + runner.successes().len();
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
